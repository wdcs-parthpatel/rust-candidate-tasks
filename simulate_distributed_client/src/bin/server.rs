use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use simulate_distributed_client::keys;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::io::AsyncReadExt;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
    let keys = keys::load_keys();
    let mut pubkeys = HashMap::new();
    for key in keys {
        pubkeys.insert(key.client_id.clone(), key.public);
    }
    let listener = TcpListener::bind("127.0.0.1:8080").await.expect("failed to bind");
    println!("Listening on {}", listener.local_addr().unwrap());
    let prices = Arc::new(Mutex::new(Vec::new()));
    loop {
        let (mut stream, _address) = listener.accept().await.expect("failed to accept");
        let prices = prices.clone();
        let pubkeys = pubkeys.clone();
        tokio::spawn(async move {
            let mut buffer = [0; 1024];
            let len = stream.read(&mut buffer).await.expect("failed to read");
            let signed: keys::SignedMessage = serde_json::from_slice(&buffer[..len]).unwrap();
            if let Some(pubkey_hex) = pubkeys.get(&signed.client_id) {
                let pubkey_bytes = hex::decode(pubkey_hex).unwrap();
                let verifying_key = VerifyingKey::from_bytes(&pubkey_bytes.try_into().unwrap()).unwrap();

                let message = format!("{}:{}", signed.client_id, signed.average);
                let sig_bytes = hex::decode(&signed.signature).unwrap();
                let signature = Signature::from_bytes(&sig_bytes.try_into().unwrap());

                if verifying_key.verify(message.as_bytes(), &signature).is_ok() {
                    println!("Received avg price of BTC: {} from {}", signed.average, signed.client_id);
                    let mut guard = prices.lock().unwrap();
                    guard.push(signed.average.parse::<f64>().unwrap());
                    if guard.len() == 5 {
                        let avg = guard.iter().sum::<f64>() / guard.len() as f64;
                        println!("Final Aggregated Average from 5 clients: {}", avg);
                        return;
                    }
                } else {
                    println!("❌ Invalid signature from {:?}", signed);
                }
            }
        });
    }
}