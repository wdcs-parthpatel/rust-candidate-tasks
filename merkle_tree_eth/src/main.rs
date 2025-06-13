use std::str::FromStr;
use secp256k1::{PublicKey, SecretKey};
use tiny_keccak::{Hasher, Keccak};
use web3::Transport;
use web3::transports::{Http};
use web3::types::{Address, BlockId, TransactionParameters, H256, U256};
use merkle_tree_eth::MerkleTree;

const RPC_URL: &str = "https://sepolia.infura.io/v3/ddf3c68b62f849feb5aa8e01c80c0caa";
const SECRET_KEY: &str = "c95241e341e5236b781553c96fc354d17d71efdd861defc7822df6db69d16e9b";
const TO_WALLET_ADDRESS: &str = "0xE689CE731843944B64F687a442422a7f850aB9Ac";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>>{
    let http = Http::new(RPC_URL)?;
    let web3 = web3::Web3::new(http);

    let secret_key: SecretKey = SecretKey::from_str(SECRET_KEY)?;
    let public_key = PublicKey::from_secret_key(&secp256k1::Secp256k1::new(), &secret_key);
    let public_key = public_key.serialize_uncompressed();
    let public_key = &public_key[1..];
    let mut hasher = Keccak::v256();
    hasher.update(public_key);
    let mut output = [0u8; 32];
    hasher.finalize(&mut output);
    let from_wallet_address = Address::from_slice(&output[12..]);

    let to_wallet_address = Address::from_str(TO_WALLET_ADDRESS)?;

    let balance = web3.eth().balance(from_wallet_address, None).await?;
    println!("balance: {} ETH", wei_to_eth(balance));

    let tx_hash = send_transaction(&web3, from_wallet_address, to_wallet_address).await?;
    println!("tx_hash: {:x}", tx_hash);
    println!("tx_hash 0: {:?}", tx_hash.0);
    
    tokio::time::sleep(tokio::time::Duration::from_secs(20)).await;
    
    let (tx_hashes, tx_root) = fetch_block_txs(&web3).await?;
    println!("tx_root: {:x}", tx_root);

    let tree = MerkleTree::new(tx_hashes.clone());
    let root = tree.root();
    println!("Block transactionRoot: {:?}", tx_root.0);
    println!("Computed Merkle root:  {:?}", root);

    let idx = tx_hashes.iter().position(|h| *h == tx_hash.0).expect("Transaction not found in block");
    println!("index: {}", idx);

    let proof = tree.proof(idx)?;
    let verified = MerkleTree::verify_proof(root, tx_hash.0, proof);
    println!("Proof Verified: {}", verified);
    Ok(())
}
fn wei_to_eth(wei: U256) -> f64 {
    let base = 1e18;
    let wei_f64 = wei.as_u128() as f64;
    wei_f64 / base
}

async fn send_transaction(web3: &web3::Web3<impl Transport>,
                         from_address: Address,
                         to_address: Address) -> Result<H256, web3::Error> {
    let nonce = web3.eth().transaction_count(from_address, None).await?;
    let gas_price = web3.eth().gas_price().await?;

    let secret_key = web3::signing::SecretKey::from_str(SECRET_KEY).expect("Parse error in secret key");
    let tx = TransactionParameters{
        nonce: Some(nonce),
        to: Some(to_address),
        gas: U256::from(21000),
        gas_price: Some(gas_price),
        value: U256::from(1_000_000_000_000_000u128),
        ..Default::default()
    };
    let signed = web3.accounts().sign_transaction(tx, &secret_key).await?;
    web3.eth().send_raw_transaction(signed.raw_transaction).await
}

async fn fetch_block_txs(web3: &web3::Web3<impl Transport>) -> web3::Result<(Vec<[u8; 32]>, H256)> {
    let block_number = web3.eth().block_number().await?;
    let block_id = BlockId::from(block_number);
    let block_tx = web3.eth().block_with_txs(block_id).await?.unwrap();
    let tx_hashes = block_tx.transactions.iter().map(|tx| {
        println!("hash: {:x}", tx.hash);
        tx.hash.0
    }).collect::<Vec<_>>();
    let tx_root = block_tx.transactions_root;
    Ok((tx_hashes, tx_root))
}