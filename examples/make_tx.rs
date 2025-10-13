/// utility helper to get a fresh signed txn.
/// args: recipient publey, sol amount to transfer, keypair path on your local, and rpc url
/// fetches a recent blockhash, sign txns, serialize and encodes in base64, and outputs that.
use std::{env, path::Path, str::FromStr};

use anyhow::{Context, anyhow};
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64_STANDARD};
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_commitment_config::CommitmentConfig;
use solana_sdk::{
    native_token::LAMPORTS_PER_SOL,
    pubkey::Pubkey,
    signature::{Signer, read_keypair_file},
    transaction::{Transaction, VersionedTransaction},
};
use solana_system_interface::instruction;

const DEFAULT_KEYPAIR: &str = "~/.config/solana/id.json";
const DEFAULT_RPC_URL: &str = "https://api.testnet.solana.com";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        eprintln!(
            "Usage: {} <RECIPIENT_PUBKEY> <AMOUNT_SOL|LAMPORTS> [KEYPAIR_PATH] [RPC_URL]",
            args[0]
        );
        return Ok(());
    }

    let recipient = Pubkey::from_str(&args[1])
        .with_context(|| format!("invalid recipient pubkey: {}", args[1]))?;
    let lamports = parse_amount_to_lamports(&args[2])?;

    let keypair_path = args
        .get(3)
        .cloned()
        .unwrap_or_else(|| DEFAULT_KEYPAIR.to_string());
    let rpc_url = args
        .get(4)
        .cloned()
        .unwrap_or_else(|| env::var("RPC_URL").unwrap_or_else(|_| DEFAULT_RPC_URL.to_string()));

    let expanded_keypair_path = expand_tilde(&keypair_path);
    let payer = read_keypair_file(&expanded_keypair_path).map_err(|err| {
        anyhow!(
            "failed to read keypair from {}: {}",
            Path::new(&expanded_keypair_path).display(),
            err
        )
    })?;

    let rpc_client = RpcClient::new(rpc_url.clone());
    let (blockhash, last_valid_block_height) = rpc_client
        .get_latest_blockhash_with_commitment(CommitmentConfig::processed())
        .await?;

    let transfer_ix = instruction::transfer(&payer.pubkey(), &recipient, lamports);
    let legacy_tx = Transaction::new_signed_with_payer(
        &[transfer_ix],
        Some(&payer.pubkey()),
        &[&payer],
        blockhash,
    );
    let versioned_tx = VersionedTransaction::from(legacy_tx);
    let serialized = bincode::serialize(&versioned_tx)?;
    let base64_tx = BASE64_STANDARD.encode(serialized);

    println!("RPC URL: {}", rpc_url);
    println!("Recent blockhash: {}", blockhash);
    println!("Last valid block height: {}", last_valid_block_height);
    println!(
        "Transaction signature: {}",
        versioned_tx
            .signatures
            .get(0)
            .map(|s| s.to_string())
            .unwrap_or_default()
    );
    println!("Amount (lamports): {}", lamports);
    println!("Fee payer: {}", payer.pubkey());
    println!("Recipient: {}", recipient);
    println!();
    println!("base64 txn: {}", base64_tx);

    Ok(())
}

fn parse_amount_to_lamports(input: &str) -> anyhow::Result<u64> {
    if let Ok(lamports) = input.parse::<u64>() {
        return Ok(lamports);
    }

    let sol: f64 = input
        .parse()
        .with_context(|| format!("invalid amount '{}': expected lamports or SOL", input))?;
    if sol < 0.0 {
        return Err(anyhow!("amount must be non-negative"));
    }

    let lamports_f64 = sol * LAMPORTS_PER_SOL as f64;
    if lamports_f64 > u64::MAX as f64 {
        return Err(anyhow!("amount too large"));
    }

    Ok(lamports_f64.round() as u64)
}

fn expand_tilde(path: &str) -> String {
    if let Some(stripped) = path.strip_prefix("~/") {
        if let Ok(home) = env::var("HOME") {
            return format!("{}/{}", home, stripped);
        }
    }
    path.to_string()
}
