use std::str::FromStr;
use std::fs;
use std::io::Write;
use futures::StreamExt;
use web3::transports::{WebSocket};
use web3::types::{Address, BlockId, BlockNumber, FilterBuilder, Log, H160, U256};
use web3::ethabi::{Contract};
use web3::signing::{keccak256};
use web3::contract;

const WS_URL: &str = "wss://sepolia.infura.io/ws/v3/ddf3c68b62f849feb5aa8e01c80c0caa";
const CONTRACT_ADDRESS: &str = "0x0A991e91aC9b8133A42e994d7aBBdCccB5008181";

#[tokio::main]
async fn main() -> web3::Result<()> {
    let transport_ws = WebSocket::new(WS_URL).await?;
    let web3 = web3::Web3::new(transport_ws);

    let contract_address = Address::from_str(CONTRACT_ADDRESS)
        .expect("invalid contract address");

    let abi_json = fs::read_to_string("storage.abi.json")?;
    let abi: Contract = serde_json::from_str(abi_json.as_str())?;
    
    let contract = contract::Contract::new(web3.eth(), contract_address, abi);

    let filter = FilterBuilder::default()
        .address(vec![contract_address])
        .topics(
            Some(vec![keccak256("NumberUpdatedEvent(address)".as_bytes()).into()]),
            None,
            None,
            None
        )
        .build();

    let mut sub_stream = web3.eth_subscribe().subscribe_logs(filter).await?;
    println!("Listening for NumberUpdatedEvent...");

    while let Some(log)= sub_stream.next().await {
        if let Ok(log) = log {
            if let Err(e) = handle_log(log, &contract).await {
                println!("Error handling log: {:?}", e);
            }
        }
    };
    Ok(())
}

async fn handle_log(log: Log, contract: &contract::Contract<WebSocket>) -> web3::Result<()> {
    println!("tx hash: {:?}", log.transaction_hash);
    let sender = H160::from_slice(&log.data.0[12..32]);
    println!("Sender address: 0x{:x}", sender);
    println!("block hash: {:?}", log.block_hash);
    println!("block number: {:?}", log.block_number);
    let block_number = BlockNumber::Number(log.block_number.unwrap());
    let block_id = BlockId::Number(block_number);
    let value: U256 = contract.query(
        "retrieve",
        (),
        None,
        contract::Options::default(),
        block_id,
    ).await.expect("retrieving value failed");
    println!("retrieved value: {}", value);
    log_to_file(value);
    Ok(())
}

fn log_to_file(value: U256) {
    let mut file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open("event_log.txt")
        .expect("opening file failed");
    writeln!(file, "Retrieved value: {}", value).expect("writing file failed");
}