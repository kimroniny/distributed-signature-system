use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use std::sync::Arc;
use tokio::sync::Mutex;
use clap::Parser;
use rand_core::OsRng;
use hex;
use std::collections::HashMap;
use uuid::Uuid;
use serde_json::json;
use bls_signatures::{aggregate, PrivateKey as KeyPair, Serialize, Signature}; // 更新为 Keypair
use reqwest::Client; // 导入 reqwest 用于发送 HTTP 请求

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    node_addr: String,

    #[arg(short, long)]
    web_addr: String,

    #[arg(short, long)]
    other_nodes: Vec<String>,

    #[arg(short, long)]
    key_collector: String, // 新增公钥收集服务的地址
}

// 共享状态，用于存储其他节点的地址
struct AppState {
    other_nodes: Vec<String>,
    pending_requests: Mutex<HashMap<String, Vec<u8>>>,
}

async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello, world!")
}

async fn send_public_key(key_collector: &str, public_key: &str) {
    let client = Client::new();
    let response = client
        .post(format!("{}/receive_key", key_collector))
        .json(&public_key)
        .send()
        .await;

    match response {
        Ok(res) if res.status().is_success() => {
            println!("Public key sent successfully.");
        },
        Ok(res) => {
            eprintln!("Failed to send public key. Server responded with status: {}", res.status());
        },
        Err(e) => {
            eprintln!("Error sending public key: {}", e);
        }
    }
}

async fn receive_message(
    msg: web::Json<serde_json::Value>,
    state: web::Data<Arc<Mutex<AppState>>>,
) -> impl Responder {
    let request_id = Uuid::new_v4().to_string();
    let request_id_clone = request_id.clone();
    
    let state_clone = state.clone();
    tokio::spawn(async move {
        let mut signatures = Vec::new();        
        let state = state_clone.lock().await;
        
        
        for node in &state.other_nodes {
            match send_to_node(node, &msg).await {
                Ok(signature) => {
                    println!("Received signature from node: {}", node);
                    signatures.push(signature);
                },
                Err(e) => eprintln!("Failed to send message to {} or receive signature: {}", node, e),
            }
        }

        
        // 聚合所有签名
        if !signatures.is_empty() {
            let aggregated_signature = aggregate_signatures(&signatures).await;
            println!("Aggregated Signature: {:?}", aggregated_signature);
            let mut pending_requests = state.pending_requests.lock().await;
            pending_requests.insert(request_id_clone, aggregated_signature.as_bytes());
        }
        
    });

    HttpResponse::Ok().json(json!({ "request_id": request_id }))
}

async fn send_to_node(addr: &str, msg: &serde_json::Value) -> std::io::Result<Signature> {
    let mut stream = TcpStream::connect(addr).await?;
    let msg_str = serde_json::to_string(msg)?;
    stream.write_all(msg_str.as_bytes()).await?;

    // 读取签名结果
    let mut buf = [0; 96]; // BLS签名大小
    stream.read_exact(&mut buf).await?;
    let signature = Signature::from_bytes(&buf)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

    Ok(signature)
}

async fn run_node_service(addr: &str, key_collector: &str) -> std::io::Result<()> {
    let listener = TcpListener::bind(addr).await?;
    println!("Node service listening on {}", addr);

    // 生成BLS签名密钥
    let keypair = KeyPair::generate(&mut OsRng);
    let public_key = keypair.public_key().as_bytes(); // 获取公钥

    // 将公钥发送到公钥收集服务
    let public_key_hex = hex::encode(public_key);
    send_public_key(key_collector, &public_key_hex).await;

    loop {
        let (mut socket, _) = listener.accept().await?;
        let keypair_clone = keypair.clone();
        tokio::spawn(async move {
            let mut buf = [0; 1024];
            match socket.read(&mut buf).await {
                Ok(n) if n > 0 => {
                    let received = String::from_utf8_lossy(&buf[..n]);
                    println!("Node service received: {}", received);

                    // 对消息进行BLS签名
                    let public_key_bytes = keypair_clone.public_key().as_bytes();
                    let public_key_hex = hex::encode(public_key_bytes);
                    let message_with_key = format!("{}:{}", public_key_hex, received);
                    let signature: Signature = keypair_clone.sign(message_with_key.as_bytes());

                    // 发送签名结果回主节点
                    if let Err(e) = socket.write_all(signature.as_bytes().as_slice()).await {
                        eprintln!("Failed to send signature: {}", e);
                    }
                }
                _ => return,
            }
        });
    }
}

// 新增聚合签名的逻辑
async fn aggregate_signatures(signatures: &[Signature]) -> Signature {
    aggregate(signatures).unwrap() // 聚合签名
}

async fn check_status(
    request_id: web::Path<String>,
    state: web::Data<Arc<Mutex<AppState>>>,
) -> impl Responder {
    let request_id = request_id.into_inner();
    let state = state.lock().await;
    let pending_requests = state.pending_requests.lock().await;
    
    match pending_requests.get(&request_id) {
        Some(signature) => HttpResponse::Ok().json(hex::encode(signature.as_slice())),
        None => HttpResponse::Ok().body("processing"),
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let args = Args::parse();

    let state = web::Data::new(Arc::new(Mutex::new(AppState {
        other_nodes: args.other_nodes,
        pending_requests: Mutex::new(HashMap::new()),
    })));

    // 启动节点服务
    let node_addr = args.node_addr.clone();
    tokio::spawn(async move {
        run_node_service(&node_addr, &args.key_collector).await.unwrap();
    });

    // 启动 Web 服务
    HttpServer::new(move || {
        App::new()
            .app_data(state.clone())
            .route("/", web::get().to(hello))
            .route("/recv_msg", web::post().to(receive_message))
            .route("/check_status/{request_id}", web::get().to(check_status))
    })
    .bind(args.web_addr)?
    .run()
    .await
}