use serde::{Deserialize, Serialize};
use std::fs;

const DIR_NAME: &str = "simple_client";
const FILE_NAME: &str = "btc_data.json";

#[derive(Serialize, Deserialize, Debug)]
pub struct PriceData {
    prices: Vec<f64>,
    pub average: f64,
}

impl PriceData {
    pub fn new(prices: Vec<f64>, average: f64) -> Self {
        Self {
            prices,
            average,
        }
    }
}

pub fn write_data_to_file(price_data: &PriceData) {
    let path_exists = fs::exists(DIR_NAME).unwrap();
    if !path_exists {
        fs::create_dir(DIR_NAME).expect("Error in creating directory");
    }
    let file_path = format!("{}/{}", DIR_NAME, FILE_NAME);
    let json_string = serde_json::to_string(price_data).expect("Error in serializing");
    fs::write(file_path, json_string).expect("Error in writing file");
}
pub fn get_data_from_file() {
    let file_path = format!("{}/{}", DIR_NAME, FILE_NAME);
    let path_exists = fs::exists(&file_path).unwrap();
    if path_exists {
        if let Ok(json_string) = fs::read_to_string(file_path) {
            let price_data: PriceData = serde_json::from_str(json_string.as_str()).expect("Error in deserializing");
            println!("Read complete: {:#?}", price_data);
        }else {
            println!("No data found, please run the cache mode first.");
        }   
    } else {
        println!("No data found, please run the cache mode first.");
    }
}