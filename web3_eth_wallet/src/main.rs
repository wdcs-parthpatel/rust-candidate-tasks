use std::fs;
use std::str::FromStr;
use secp256k1::{Secp256k1, SecretKey, PublicKey};
use web3::types::{Address, TransactionParameters, U256};
use web3::{Transport};
use web3::transports::{Http};
use tiny_keccak::{Hasher, Keccak};

const WALLET_FILE: &str = "wallet.json";
const RPC_URL: &str = "https://sepolia.infura.io/v3/ddf3c68b62f849feb5aa8e01c80c0caa";
const TO_WALLET_ADDRESS: &str = "0xE689CE731843944B64F687a442422a7f850aB9Ac";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let transport_http = Http::new(RPC_URL)?;
    let web3 = web3::Web3::new(transport_http);
    println!("Connected to Ethereum node");

    let secret_key = create_or_load_wallet().await?;
    println!("Private key: {}", secret_key.display_secret());

    let wallet_address = get_wallet_address(&secret_key);
    println!("Wallet Address: 0x{:x}", wallet_address);
    
    let balance = get_balance(&web3, wallet_address).await?;
    println!("Balance: {} Wei", balance);
    println!("Balance: {} ETH", wei_to_eth(balance));

    let to_address = Address::from_str(TO_WALLET_ADDRESS)?;
    let amount = U256::from(1_000_000_000_000_000u128); // 0.001 ETH
    let tx_hash = send_eth(&web3, &secret_key, wallet_address, to_address, amount).await?;
    println!("Transaction sent: {:?}", tx_hash);
    Ok(())
}

#[derive(serde::Serialize, serde::Deserialize)]
struct Wallet {
    private_key: String,
}

async fn create_or_load_wallet() -> Result<SecretKey, Box<dyn std::error::Error>> {
    if let Ok(content) = fs::read_to_string(WALLET_FILE) {
        let wallet: Wallet = serde_json::from_str(&content)?;
        let secret_key = SecretKey::from_str(&wallet.private_key)?;
        Ok(secret_key)
    } else {
        let mut rng = rand::rngs::OsRng;
        let secp = Secp256k1::new();
        let (secret_key, _public_key) = secp.generate_keypair(&mut rng);
        let wallet = Wallet {
            private_key: secret_key.display_secret().to_string(),
        };
        let json = serde_json::to_string(&wallet)?;
        fs::write(WALLET_FILE, json)?;
        Ok(secret_key)
    }
}

fn get_wallet_address(secret_key: &SecretKey) -> Address {
    let public_key = PublicKey::from_secret_key(&Secp256k1::new(), &secret_key);
    let public_key = public_key.serialize_uncompressed();
    let public_key = &public_key[1..];
    let mut hasher = Keccak::v256();
    hasher.update(public_key);
    let mut output = [0u8; 32];
    hasher.finalize(&mut output);
    Address::from_slice(&output[12..])
}

async fn get_balance(web3: &web3::Web3<impl Transport>, address: Address) -> Result<U256, web3::Error> {
    web3.eth().balance(address, None).await
}

fn wei_to_eth(wei: U256) -> f64 {
    let base = 1e18;
    let wei_f64 = wei.as_u128() as f64;
    wei_f64 / base
}

async fn send_eth(
    web3: &web3::Web3<impl Transport>,
    from_secret_key: &SecretKey,
    from_address: Address,
    to_address: Address,
    amount: U256,
) -> Result<web3::types::H256, web3::Error> {
    let nonce = web3.eth().transaction_count(from_address, None).await?;
    println!("nonce: {:?}", nonce);
    let gas_price = web3.eth().gas_price().await?;
    println!("gas_price: {:?}", gas_price);
    let transaction = TransactionParameters {
        nonce: Some(nonce),
        to: Some(to_address),
        gas: U256::from(21000),
        gas_price: Some(gas_price),
        value: amount,
        ..Default::default()
    };
    let signed = web3.accounts().sign_transaction(transaction, from_secret_key).await?;
    web3.eth().send_raw_transaction(signed.raw_transaction).await
}