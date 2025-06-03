use std::fs;
use std::str::FromStr;
use web3::types::{Address, TransactionParameters, U256};
use secp256k1::SecretKey;
use web3::Transport;
use tiny_keccak::{Hasher, Keccak};

const WALLET_FILE: &str = "wallet.json";

#[derive(serde::Serialize, serde::Deserialize)]
struct Wallet {
    private_key: String,
}

async fn create_or_load_wallet() -> Result<SecretKey, Box<dyn std::error::Error>> {
    if let Ok(content) = fs::read_to_string(WALLET_FILE) {
        let wallet: Wallet = serde_json::from_str(&content)?;
        let private_key = SecretKey::from_str(&wallet.private_key)?;
        Ok(private_key)
    } else {
        let mut rng = rand::thread_rng();
        let secp = secp256k1::Secp256k1::new();
        let private_key = secp.generate_keypair(&mut rng).0;

        // Save to file
        let wallet = Wallet {
            private_key: private_key.display_secret().to_string(),
        };
        let json = serde_json::to_string(&wallet)?;
        fs::write(WALLET_FILE, json)?;

        Ok(private_key)
    }
}

async fn get_balance(web3: &web3::Web3<impl Transport>, address: Address) -> Result<U256, web3::Error> {
    web3.eth().balance(address, None).await
}

fn public_key_to_address(public_key: &secp256k1::PublicKey) -> Address {
    let public_key = public_key.serialize_uncompressed();
    // Remove the first byte (0x04) which indicates an uncompressed public key
    let public_key = &public_key[1..];

    // Keccak-256 hash of the public key
    let mut hasher = Keccak::v256();
    hasher.update(public_key);
    let mut output = [0u8; 32];
    hasher.finalize(&mut output);

    // Take the last 20 bytes as ethereum address
    Address::from_slice(&output[12..])
}

async fn send_eth(
    web3: &web3::Web3<impl Transport>,
    from_private_key: &SecretKey,
    to_address: Address,
    amount: U256,
) -> Result<web3::types::H256, web3::Error> {
    let public_key = secp256k1::PublicKey::from_secret_key(
        &secp256k1::Secp256k1::new(),
        from_private_key,
    );
    let from_address = public_key_to_address(&public_key);

    let nonce = web3.eth().transaction_count(from_address, None).await?;

    let transaction = TransactionParameters {
        to: Some(to_address),
        value: amount,
        nonce: Some(nonce),
        gas: U256::from(21000),
        gas_price: Some(web3.eth().gas_price().await?),
        ..Default::default()
    };

    let signed = web3.accounts().sign_transaction(transaction, from_private_key).await?;
    web3.eth().send_raw_transaction(signed.raw_transaction).await
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Connect to an Ethereum node (replace URL with your node)
    let transport = web3::transports::Http::new(
        "https://sepolia.infura.io/v3/ddf3c68b62f849feb5aa8e01c80c0caa"
    )?;
    let web3 = web3::Web3::new(transport);
    println!("Connected to Ethereum node");

    let private_key = create_or_load_wallet().await?;
    println!("Private key: {}", private_key.display_secret());

    let public_key = secp256k1::PublicKey::from_secret_key(
        &secp256k1::Secp256k1::new(),
        &private_key
    );
    println!("Public key: {}", public_key);

    let address = public_key_to_address(&public_key);
    println!("Wallet Address: 0x{:x}", address);
    let address = Address::from_str("0xf11cF0AC883B42116006eF5AcDB5fDD34a766a75")?;
    
    // Get balance
    let balance = get_balance(&web3, address).await?;
    println!("Balance: {} Wei", balance);


    // Example: Send ETH (uncomment and modify as needed)
    let to_address = Address::from_str("0x742d35Cc6634C0532925a3b844Bc454e4438f44e")?;
    let amount = U256::from(1000000000000000000u64); // 1 ETH
    let tx_hash = send_eth(&web3, &private_key, to_address, amount).await?;
    println!("Transaction sent: {:?}", tx_hash);
    Ok(())
}