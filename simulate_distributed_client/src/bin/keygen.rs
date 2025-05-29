use ed25519_dalek::SigningKey;
use rand::rngs::OsRng;
use simulate_distributed_client::keys::KeyData;
use std::fs::File;
use std::io::Write;

fn main() {
    let mut clients = vec![];
    for i in 1..=5 {
        let client_id = format!("client{}", i);
        let signing_key = SigningKey::generate(&mut OsRng);
        let verifying_key = signing_key.verifying_key();

        let private_hex = hex::encode(signing_key.to_bytes());
        let public_hex = hex::encode(verifying_key.to_bytes());

        let key_data = KeyData {
            client_id,
            public: public_hex,
            private: private_hex,
        };
        clients.push(key_data);
    }

    let json = serde_json::to_string_pretty(&clients).unwrap();
    let mut file = File::create("keys.json").unwrap();
    file.write_all(json.as_bytes()).unwrap();
    println!("✅ keys.json generated");
}
