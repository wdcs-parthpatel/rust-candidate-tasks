use std::str::FromStr;
use web3::futures::StreamExt;
use web3::transports::{WebSocket};
use web3::types::{Address, BlockId, BlockNumber};

// const RPC_URL: &str = "https://sepolia.infura.io/v3/ddf3c68b62f849feb5aa8e01c80c0caa";
const WS_URL: &str = "wss://sepolia.infura.io/ws/v3/ddf3c68b62f849feb5aa8e01c80c0caa";
const WALLET_ADDRESS: &str = "0xb4ebd8ada94410820dfaf2f9c33b405e1df9747c";

#[tokio::main(flavor = "current_thread")]
async fn main() -> web3::Result<()> {
    // socket_block_pooling().await?;
    socket_block_subscribing().await?;
    Ok(())
}
#[allow(unused)]
async fn socket_block_pooling() -> web3::Result<()> {
    let transport_ws = WebSocket::new(WS_URL).await?;
    let web3 = web3::Web3::new(transport_ws);

    let wallet_address = Address::from_str(WALLET_ADDRESS).expect("invalid address");
    println!("wallet_address {:?}", wallet_address);
    
    let mut block_number = web3.eth().block_number().await?.as_u64();
    loop {
        println!("Block number: {}", block_number);
        if let Ok(Some(block)) = web3.eth().block_with_txs(BlockId::Number(BlockNumber::Number(block_number.into()))).await {
            for tx in block.transactions {
                if tx.to == Some(wallet_address) {
                    println!("✅ Confirmed tx:\n  Hash: {:?}\n  From: {:?}\n  Value: {} Wei", tx.hash, tx.from, tx.value);
                }
            }
            block_number += 1;
        } else {
            tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;   
        }
    }
}

#[allow(unused)]
async fn socket_block_subscribing() -> web3::Result<()> {
    let transport_ws = WebSocket::new(WS_URL).await?;
    let web3 = web3::Web3::new(transport_ws);

    let wallet_address = Address::from_str(WALLET_ADDRESS).expect("invalid address");
    println!("wallet_address {:?}", wallet_address);
    let mut stream = web3.eth_subscribe().subscribe_new_heads().await?;

    while let Some(header) = stream.next().await{
        match header {
            Ok(block) => {
                if let Some(block_number) = block.number {
                    println!("block_number: {}", block_number);
                    if let Ok(Some(block)) = web3.eth().block_with_txs(BlockId::Number(BlockNumber::Number(block_number.into()))).await {
                        for tx in block.transactions {
                            if tx.to == Some(wallet_address) {
                                println!("✅ Confirmed tx:\n  Hash: {:?}\n  From: {:?}\n  Value: {} Wei", tx.hash, tx.from, tx.value);
                            }
                        }
                    }
                }
                else {
                    println!("New block received without block number");
                }
            }
            Err(e) => {
                println!("Error in receiving block: {:?}", e)
            }
        }
    }
    Ok(())
}