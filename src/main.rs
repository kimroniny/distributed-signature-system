#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(dead_code)]
use actix_web::web::BytesMut;
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
use reqwest::Client; // 导入 reqwest 用于发送 HTTP 请求
use core::slice;
use std::ops::Neg;
use num_bigint::BigUint;
use bn254::{PrivateKey, PublicKey, ECDSA, Signature};
use k256::{elliptic_curve::bigint::Encoding, U256};
use substrate_bn::{Group, G1};

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
            println!("Public key sent successfully. pk: {}", public_key);
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
        let message = msg.get("message").unwrap();
        let message_str = message.as_str().unwrap();    
        println!("message: {}", message_str);
        
        for node in &state.other_nodes {
            match send_to_node(node, message_str).await {
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
            pending_requests.insert(request_id_clone, aggregated_signature.to_compressed().unwrap());
        }
        
    });

    HttpResponse::Ok().json(json!({ "request_id": request_id }))
}

async fn send_to_node(addr: &str, msg: &str) -> std::io::Result<Signature> {
    let mut stream = TcpStream::connect(addr).await?;
    stream.write_all(msg.as_bytes()).await?;

    // 读取签名结果
    let mut buf = BytesMut::with_capacity(1024); // BLS签名大小
    stream.read_buf(&mut buf).await?;
    println!("Received signature: {:?}", buf);
    let signature = Signature::from_compressed(&buf)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

    Ok(signature)
}

async fn run_node_service(addr: &str, key_collector: &str) -> std::io::Result<()> {
    let listener = TcpListener::bind(addr).await?;
    println!("Node service listening on {}", addr);

    // 生成BLS签名密钥
    let private_key = PrivateKey::random(&mut OsRng);
    let public_key = PublicKey::from_private_key(&private_key);

    // 将公钥发送到公钥收集服务
    let public_key_hex = hex::encode(public_key.to_compressed().unwrap());
    send_public_key(key_collector, &public_key_hex).await;

    loop {
        let (mut socket, _) = listener.accept().await?;
        let keypair_clone = private_key.to_bytes().unwrap().clone();
        tokio::spawn(async move {
            let mut buf = [0; 1024];
            let private_key_clone = PrivateKey::try_from(keypair_clone.as_slice()).unwrap();
            match socket.read(&mut buf).await {
                Ok(n) if n > 0 => {
                    let received = String::from_utf8_lossy(&buf[..n]);
                    println!("Node service received: {}", received);

                    let message = received.as_bytes();
                    
                    // let hash_point = hash_to_try_and_increment(message).unwrap();
                    // TODO 使用合约中的hash_point来签名
                    let signature = ECDSA::sign(&message, &private_key_clone).unwrap();

                    // 发送签名结果回主节点
                    if let Err(e) = socket.write_all(signature.to_compressed().unwrap().as_slice()).await {
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
    signatures.iter().fold(Signature(G1::zero()), |acc, sig| acc + *sig)
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