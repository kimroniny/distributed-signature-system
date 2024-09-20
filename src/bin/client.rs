use clap::{Parser, Subcommand};
use reqwest::Client;
use serde_json::json;
use std::time::Duration;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[arg(short, long, default_value = "http://localhost:8080")]
    server: String,

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
    }

    Ok(())
}
