use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Serialize, Deserialize, Debug)]
pub struct SignedMessage {
    pub client_id: String,
    pub average: String,
    pub signature: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct KeyData {
    pub client_id: String,
    pub public: String,
    pub private: String,
}

pub fn load_keys() -> Vec<KeyData> {
    let data = fs::read_to_string("keys.json").unwrap();
    serde_json::from_str(&data).unwrap()
}