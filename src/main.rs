use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use std::sync::Arc;
use tokio::sync::Mutex;
use clap::Parser;
use k256::ecdsa::{SigningKey, Signature, signature::Signer};
use rand_core::OsRng;
use hex;
use std::collections::HashMap;
use uuid::Uuid;
use serde_json::json;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    node_addr: String,

    #[arg(short, long)]
    web_addr: String,

    #[arg(short, long)]
    other_nodes: Vec<String>,
}

// 共享状态，用于存储其他节点的地址
struct AppState {
    other_nodes: Vec<String>,
    pending_requests: Mutex<HashMap<String, Vec<Vec<u8>>>>,
}

async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello, world!")
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
                Ok(signature) => signatures.push(signature.to_bytes()),
                Err(e) => eprintln!("Failed to send message to {} or receive signature: {}", node, e),
            }
        }
        
        let mut pending_requests = state.pending_requests.lock().await;
        pending_requests.insert(request_id_clone, signatures.iter().map(|s| s.to_vec()).collect());
    });

    HttpResponse::Ok().json(json!({ "request_id": request_id }))
}

async fn send_to_node(addr: &str, msg: &serde_json::Value) -> std::io::Result<Signature> {
    let mut stream = TcpStream::connect(addr).await?;
    let msg_str = serde_json::to_string(msg)?;
    stream.write_all(msg_str.as_bytes()).await?;

    // 读取签名结果
    let mut buf = [0; 64];
    stream.read_exact(&mut buf).await?;
    let signature = Signature::from_slice(&buf)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

    Ok(signature)
}

async fn run_node_service(addr: &str) -> std::io::Result<()> {
    let listener = TcpListener::bind(addr).await?;
    println!("Node service listening on {}", addr);

    // 生成签名密钥
    let signing_key = SigningKey::random(&mut OsRng);

    loop {
        let (mut socket, _) = listener.accept().await?;
        let key = signing_key.clone();
        tokio::spawn(async move {
            let mut buf = [0; 1024];
            match socket.read(&mut buf).await {
                Ok(n) if n > 0 => {
                    let received = String::from_utf8_lossy(&buf[..n]);
                    println!("Node service received: {}", received);

                    // 对消息进行签名
                    let signature: Signature = key.sign(received.as_bytes());

                    // 发送签名结果回主节点
                    if let Err(e) = socket.write_all(signature.to_bytes().as_slice()).await {
                        eprintln!("Failed to send signature: {}", e);
                    }
                }
                _ => return,
            }
        });
    }
}

async fn check_status(
    request_id: web::Path<String>,
    state: web::Data<Arc<Mutex<AppState>>>,
) -> impl Responder {
    let request_id = request_id.into_inner();
    let state = state.lock().await;
    let pending_requests = state.pending_requests.lock().await;
    
    match pending_requests.get(&request_id) {
        Some(signatures) => HttpResponse::Ok().json(signatures.iter().map(|s| hex::encode(s)).collect::<Vec<String>>()),
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
        run_node_service(&node_addr).await.unwrap();
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