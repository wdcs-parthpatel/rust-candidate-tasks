pub mod keys;

use clap::Parser;
use ed25519_dalek::{Signer, SigningKey};
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;

const NUM_OF_CLIENT: usize = 5;
const WEB_SOCKET_STREAM: &str = "wss://stream.binance.com:443/ws/btcusdt@miniTicker";

#[derive(Debug, Parser)]
struct Cli {
    #[arg(short, long, help = "Value should only be cache")]
    mode: String,
    #[arg(short, long, default_value = "10", help = "Number of seconds to fetch the price for")]
    times: u64,
}

#[derive(Debug, Deserialize, Serialize)]
struct BTC {
    s: String,
    #[serde(deserialize_with = "de_string_to_f64")]
    c: f64,
}

fn de_string_to_f64<'de, D>(deserializer: D) -> Result<f64, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s: &str = Deserialize::deserialize(deserializer)?;
    s.parse::<f64>().map_err(serde::de::Error::custom)
}


#[tokio::main(flavor = "multi_thread")]
async fn main() {
    let cli = Cli::parse();
    match cli.mode.as_str() {
        "cache" => {
            client_process(cli.times).await;
        }
        _ => {
            println!("Invalid mode");
        }
    }
}

async fn client_process(times: u64) {
    let (tx, rx) = tokio::sync::mpsc::channel::<(usize, f64)>(NUM_OF_CLIENT);
    for id in 1..=NUM_OF_CLIENT {
        let tx_clone = tx.clone();
        tokio::spawn(async move {
            let result = get_price_from_socket_stream(times, id, tx_clone).await;
            if result.is_err() {
                println!("Error:: {}", result.err().unwrap());
            }
        });
    }
    send_to_server(rx).await;
}

async fn get_price_from_socket_stream(times: u64, id: usize, tx: tokio::sync::mpsc::Sender<(usize, f64)>) -> Result<(), Box<dyn std::error::Error>> {
    let (stream, _response) = tokio_tungstenite::connect_async(WEB_SOCKET_STREAM).await?;
    let (_write, mut read) = stream.split();
    let start_time = tokio::time::Instant::now();
    let fetch_time = tokio::time::Duration::from_secs(times);
    let mut prices = Vec::new();
    println!("Client {id} connected to WebSocket.");
    while start_time.elapsed() < fetch_time {
        if let Some(message) = read.next().await {
            let message = message?.to_string();
            let btc = serde_json::from_str::<BTC>(&message);
            if btc.is_ok() {
                let price = btc.unwrap().c;
                prices.push(price);
            }
        }
    }
    let avg = prices.iter().sum::<f64>() / prices.len() as f64;
    println!("The average price is: {} for client id: {}", avg, id);
    tx.send((id, avg)).await?;
    Ok(())
}

async fn send_to_server(mut rx: tokio::sync::mpsc::Receiver<(usize, f64)>) {
    let mut count = 0;
    let keys = keys::load_keys();
    while let Some(value) = rx.recv().await {
        let client_id = format!("client{}", value.0);
        let average_price = value.1;
        let key_data = keys.iter().find(|data| { data.client_id == client_id });
        if key_data.is_some() {
            let key_data = key_data.unwrap();
            let secret_key = hex::decode(key_data.private.clone()).unwrap().try_into().unwrap();
            let signing_key = SigningKey::from_bytes(&secret_key);
            let message = format!("{}:{}", client_id, average_price);
            let signature = signing_key.sign(message.as_bytes());
            let signature_hex = hex::encode(signature.to_bytes());
            let signed_message = keys::SignedMessage {
                client_id,
                average: average_price.to_string(),
                signature: signature_hex,
            };
            let encoded = serde_json::to_vec(&signed_message).unwrap();
            let mut stream = TcpStream::connect("127.0.0.1:8080").await.expect("Failed to connect");
            stream.write_all(&encoded).await.expect("Failed to send");

            count += 1;
            if count == 5 {
                break;
            }
        }
    };
}