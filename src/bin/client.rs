use clap::{Parser, Subcommand};
use reqwest::Client;
use serde_json::json;
use std::time::Duration;
use bls_signatures::{PublicKey, Serialize, Signature, verify_messages}; // 导入 BLS 签名和公钥

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
            let signature = Signature::from_bytes(&signature_bytes)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

            println!("signature: {:?}", signature.as_bytes());

            // 获取所有公钥
            let public_keys_res = client
                .get(format!("{}/public_keys", cli.key_collector)) // 从公钥收集服务获取公钥
                .send()
                .await?
                .json::<Vec<String>>() // 获取公钥列表
                .await?;
            println!("public_keys_res: {:?}", public_keys_res);

            // 验证签名
            let messages_with_key: Result<Vec<String>, std::io::Error> = public_keys_res.iter().map(|public_key_hex| {
                let message_with_key = format!("{}:{}", public_key_hex, message);
                println!("message_with_key: {}", message_with_key);
                Ok(message_with_key)
            }).collect();
            let messages_with_key = messages_with_key?;
            for (i, message) in messages_with_key.iter().enumerate() {
                println!("Message {}: {:?}", i + 1, message);
            }
            let valid = verify_messages(
                &signature, 
                &messages_with_key.iter().map(|m| m.as_bytes()).collect::<Vec<&[u8]>>(), 
                &public_keys_res.iter().map(|k| {
                    let public_key_bytes = hex::decode(k).unwrap();
                    PublicKey::from_bytes(&public_key_bytes).unwrap()
                }).collect::<Vec<PublicKey>>()
            );


            if valid {
                println!("Signature is valid.");
            } else {
                println!("Signature is invalid.");
            }
        }
    }

    Ok(())
}
