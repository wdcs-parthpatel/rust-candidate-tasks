mod file;

use std::fs;
use clap::{Arg, Command};
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;


const WEB_SOCKET_URL_BTC: &str = "wss://stream.binance.com:443/ws/btcusdt@miniTicker";
const CONNECTION_TIMEOUT: u64 = 5;

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    let matches = Command::new("simple")
        .about("simple client that can run the two following commands: 1) ./simple --mode=cache --times=10 2) ./simple --mode=read")
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
                   // let result = get_btc_price(times).await;
                   // if result.is_err() {
                   //     println!("Failed to get btc price: {}", result.err().unwrap());
                   // }
                   client_process(times).await;
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

async fn client_process(times: usize) {
    println!("Failed to get btc price:");
    let (tx, rx) = mpsc::channel::<file::PriceData>(5);

    for id in 0..5 {
        let tx_clone = tx.clone();
        tokio::spawn(async move {
            println!("Thread id: {:?}", std::thread::current().id());
            let result = get_btc_price(times, id, tx_clone).await;
            if result.is_err() {
                println!("Failed to get btc price: {}", result.err().unwrap());
            }
        });
    }
    aggregator(rx).await;
}

async fn aggregator(mut rx: mpsc::Receiver<file::PriceData>) {
    let mut results = Vec::new();
    while let Some(price_data) = rx.recv().await {
        println!("Aggregator received avg from client: {:?}", price_data);
        results.push(price_data.average);

        if results.len() == 5 {
            let final_avg = results.iter().sum::<f64>() / results.len() as f64;
            println!("\n✅ Final Aggregated Average of all clients: {:.2}", final_avg);
            break;
        }
    }
}

async fn get_btc_price(times: usize, id: i32, tx: mpsc::Sender<file::PriceData>) -> Result<(), Box<dyn std::error::Error>> {
    let connection = tokio_tungstenite::connect_async(WEB_SOCKET_URL_BTC);
    let timeout_duration = tokio::time::Duration::from_secs(CONNECTION_TIMEOUT);
    let (stream, _response) = tokio::time::timeout(timeout_duration, connection).await??;
    let (_write, mut read) = stream.split();
    let fetch_times = tokio::time::Duration::from_secs(times as u64);
    let start_time = tokio::time::Instant::now();
    let mut prices = Vec::new();
    println!("Client {id} connected to WebSocket.");
    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    
    while start_time.elapsed() < fetch_times {
        if let Some(msg) = read.next().await{
            let message = msg?.to_string();
            let btc_ticker = serde_json::from_str::<BtcTicker>(message.as_str());
            if btc_ticker.is_ok() {
                let price = btc_ticker.unwrap().c;
                prices.push(price);
                println!("Received btc Price: {} for id:{}", price, id);
            } else {
                println!("Failed to get btc price: {}", btc_ticker.err().unwrap());
            }
        }
    }
    let average = prices.iter().sum::<f64>() / prices.len() as f64;
    let price_data = file::PriceData::new(prices, average);

    let json_string = serde_json::to_string(&price_data).expect("Error in serializing");
    fs::write(format!("btc_price.{id}.json"), json_string).expect("Error in writing file");

    // file::write_data_to_file(&price_data);
    println!("Cache complete. The average USD price of BTC is: {} for id: {}", average, id);

    let _ = tx.send(price_data).await;
    Ok(())
}

#[derive(Serialize, Deserialize, Debug)]
struct BtcTicker {
    s: String,
    #[serde(deserialize_with = "de_string_to_f64")]
    c: f64,
}

// Custom deserializer function
fn de_string_to_f64<'de, D>(deserializer: D) -> Result<f64, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s: &str = Deserialize::deserialize(deserializer)?;
    s.parse::<f64>().map_err(serde::de::Error::custom)
}
