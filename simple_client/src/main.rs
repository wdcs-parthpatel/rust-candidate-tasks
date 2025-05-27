mod file;

use clap::{Arg, Command};
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

const WEB_SOCKET_URL_BTC: &str = "wss://stream.binance.com:443/ws/btcusdt@miniTicker";
const CONNECTION_TIMEOUT: u64 = 5;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let matches = Command::new("simple")
        .about("simple client that can run the two following commands: 1) ./ =10 2) ./simple --mode=read")
        .arg(Arg::new("mode")
            .short('m')
            .long("mode")
            .required(true)
            .help("Mode should be 'cache' or 'read'")
        )
        .arg(Arg::new("times")
            .short('t')
            .long("times")
            .required(false)
            .help("Number of times to fetch the price")
            .default_value("10")
        ).get_matches();

    let mode = matches.get_one::<String>("mode").unwrap();
    match mode.as_str() {
       "cache" => {
           let times = matches.get_one::<String>("times").unwrap();
           match times.parse::<usize>() {
               Ok(times) => {
                   get_btc_price(times).await;
               },
               Err(e) => {
                   println!("Error in times argument: {}",e);
               }
           }
       },
       "read" => {
           file::get_data_from_file();
       },
       _ => {
           println!("Error: mode should be either 'cache' or 'read'");
       }
   }
}

async fn get_btc_price(times: usize) {
    let connection = tokio_tungstenite::connect_async(WEB_SOCKET_URL_BTC);
    let timeout_duration = tokio::time::Duration::from_secs(CONNECTION_TIMEOUT);
    let result = tokio::time::timeout(timeout_duration, connection).await;
    match result {
        Ok(connection) => {
            if connection.is_ok() {
                let (stream, _response) = connection.ok().unwrap();
                let (_write, read) = stream.split();
                let prices = Arc::new(Mutex::new(Vec::new()));
                read.take(times).for_each(|msg| {
                    let prices = prices.clone();
                    async move {
                        if msg.is_ok() {
                            let message = msg.ok().unwrap().to_string();
                            let btc_ticker = serde_json::from_str::<BtcTicker>(message.as_str());
                            if btc_ticker.is_ok() {
                                let price = btc_ticker.ok().unwrap().c.parse::<f64>().unwrap();
                                prices.lock().unwrap().push(price);
                                println!("Received btc Price: {}", price);
                            } else {
                                println!("Error in deserializing: {:?}", btc_ticker.err().unwrap());
                            }
                        } else {
                            println!("Error in message: {:?}", msg.err().unwrap());
                        }
                    }
                }).await;
                let data: Vec<f64> = prices.lock().unwrap().iter().map(|x| *x).collect();
                let average = data.iter().sum::<f64>() / data.len() as f64;
                let price_data = file::PriceData::new(data, average);
                file::write_data_to_file(&price_data);
                println!("Cache complete. The average USD price of BTC is: {}", average);
            } else {
                println!("Error connecting to the websocket {}", connection.err().unwrap());
            }
        },
        Err(_) => {
            println!("WebSocket connection timed out.");
        }       
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct BtcTicker {
    s: String,
    c: String,
}