mod file;

use clap::{Arg, Command};
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};

const WEB_SOCKET_URL_BTC: &str = "wss://stream.binance.com:443/ws/btcusdt@miniTicker";
const CONNECTION_TIMEOUT: u64 = 5;

#[tokio::main(flavor = "current_thread")]
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
                   let result = get_btc_price(times).await;
                   if result.is_err() {
                       println!("Failed to get btc price: {}", result.err().unwrap());
                   }
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

async fn get_btc_price(times: usize) -> Result<(), Box<dyn std::error::Error>> {
    let connection = tokio_tungstenite::connect_async(WEB_SOCKET_URL_BTC);
    let timeout_duration = tokio::time::Duration::from_secs(CONNECTION_TIMEOUT);
    let (stream, _response) = tokio::time::timeout(timeout_duration, connection).await??;
    let (_write, mut read) = stream.split();
    let fetch_times = tokio::time::Duration::from_secs(times as u64);
    let start_time = tokio::time::Instant::now();
    let mut prices = Vec::new();

    while start_time.elapsed() < fetch_times {
        if let Some(msg) = read.next().await{
            let message = msg?.to_string();
            let btc_ticker = serde_json::from_str::<BtcTicker>(message.as_str());
            if btc_ticker.is_ok() {
                let price = btc_ticker.unwrap().c;
                prices.push(price);
                println!("Received btc Price: {}", price);
            } else {
                println!("Failed to get btc price: {}", btc_ticker.err().unwrap());
            }
        }
    }
    let average = prices.iter().sum::<f64>() / prices.len() as f64;
    let price_data = file::PriceData::new(prices, average);
    file::write_data_to_file(&price_data);
    println!("Cache complete. The average USD price of BTC is: {}", average);
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