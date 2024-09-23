use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use std::sync::{Arc, Mutex};
use clap::{Parser};

#[derive(Parser)]
struct Args {
    #[arg(short, long, default_value = "127.0.0.1:8081")]
    addr: String, // 新增地址和端口参数
}

struct AppState {
    public_keys: Mutex<Vec<String>>, // 存储公钥
}

async fn receive_key(key: web::Json<String>, state: web::Data<Arc<AppState>>) -> impl Responder {
    let mut public_keys = state.public_keys.lock().unwrap();
    public_keys.push(key.clone());
    println!("Received public key: {}", key.clone());

    HttpResponse::Ok().body("Public key received.")
}

async fn get_public_keys(state: web::Data<Arc<AppState>>) -> impl Responder {
    let public_keys = state.public_keys.lock().unwrap();
    HttpResponse::Ok().json(public_keys.clone())
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let args = Args::parse(); // 解析命令行参数

    let state = web::Data::new(Arc::new(AppState {
        public_keys: Mutex::new(Vec::new()), // 初始化公钥存储
    }));

    HttpServer::new(move || {
        App::new()
            .app_data(state.clone())
            .route("/receive_key", web::post().to(receive_key)) // 接收公钥的路由
            .route("/public_keys", web::get().to(get_public_keys)) // 获取所有公钥的路由
    })
    .bind(&args.addr)? // 使用命令行参数中的地址和端口
    .run()
    .await
}