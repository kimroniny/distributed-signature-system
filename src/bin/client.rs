use clap::{Parser, Subcommand};
use reqwest::Client;
use serde_json::json;
use std::time::Duration;
use bn254::{PrivateKey, PublicKey, ECDSA, Signature};
use substrate_bn::{Group, G2};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[arg(short, long, default_value = "http://localhost:8080")]
    server: String,

    #[arg(short, long, default_value = "http://localhost:8081")]
    key_collector: String, // 新增公钥收集服务的地址

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Send {
        #[arg(short, long)]
        message: String,
    },
    Check {
        #[arg(short, long)]
        request_id: String,
    },
    Verify {
        #[arg(short, long)]
        request_id: String,
        #[arg(short, long)]
        message: String, // 添加消息参数
    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    let client = Client::new();

    match &cli.command {
        Commands::Send { message } => {
            let res = client
                .post(format!("{}/recv_msg", cli.server))
                .json(&json!({ "message": message }))
                .send()
                .await?
                .json::<serde_json::Value>()
                .await?;
            
            println!("Request sent. Request ID: {}", res["request_id"]);
        }
        Commands::Check { request_id } => {
            loop {
                let res = client
                    .get(format!("{}/check_status/{}", cli.server, request_id))
                    .send()
                    .await?
                    .text()
                    .await?;

                if res == "processing" {
                    println!("Still processing...");
                    tokio::time::sleep(Duration::from_secs(5)).await;
                } else {
                    println!("Processing complete. Result: {}", res);
                    break;
                }
            }
        }
        Commands::Verify { request_id, message } => {
            // 获取签名
            let signature_res = client
                .get(format!("{}/check_status/{}", cli.server, request_id))
                .send()
                .await?
                .json::<String>()
                .await?;

            println!("signature_res: {}", signature_res);

            // 解析签名
            let signature_bytes = hex::decode(signature_res)?;
            println!("signature_bytes: {:?}", signature_bytes);
            let signature = Signature::from_compressed(&signature_bytes)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

            println!("signature: {:?}", signature);

            // 获取所有公钥
            let public_keys_res = client
                .get(format!("{}/public_keys", cli.key_collector)) // 从公钥收集服务获取公钥
                .send()
                .await?
                .json::<Vec<String>>() // 获取公钥列表
                .await?;
            println!("public_keys_res: {:?}", public_keys_res);

            // 聚合公钥
            let public_keys = public_keys_res[1..].iter().map(|public_key_hex| {
                let public_key_bytes = hex::decode(public_key_hex).unwrap();
                PublicKey::from_compressed(&public_key_bytes).unwrap()
            }).collect::<Vec<PublicKey>>();
            let aggregated_public_key = public_keys.iter().fold(PublicKey(G2::zero()), |acc, x| acc + *x);

            // 验证签名
            if let Ok(_) = ECDSA::verify(&message, &signature, &aggregated_public_key) {
                println!("Signature is valid.");
            } else {
                println!("Signature is invalid.");
            }
        }
    }

    Ok(())
}
