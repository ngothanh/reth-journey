//! `etherscanlite <address> [--tx <hash>]`
//!
//! Prints an address's balance and nonce, and optionally one transaction by hash.
//! Everything crossing the RPC boundary is converted into `eth-primitives` types by
//! `boundary.rs`; no `alloy_*` type escapes that module (R4).

mod boundary;
mod client;
mod retry;

use anyhow::{bail, Context, Result};
use clap::Parser;
use eth_primitives::{Address, B256};

use client::Client;

/// Fetch balance, nonce, and (optionally) a transaction from an Ethereum RPC endpoint.
#[derive(Parser)]
#[command(name = "etherscanlite", version)]
struct Cli {
    /// Account address, 0x-prefixed. Mixed-case input is EIP-55 checksum validated.
    address: String,

    /// Also fetch this transaction by hash.
    #[arg(long, value_name = "HASH")]
    tx: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // R5 + D4: validate configuration at startup, not at first use. Reading this lazily
    // means a missing variable surfaces as a connection error 30s deep in the provider
    // stack, pointing at the network instead of at the operator's shell.
    let rpc_url = std::env::var("ETH_RPC_URL").context(
        "ETH_RPC_URL must be set to a valid RPC endpoint (e.g. an Alchemy or Infura https URL)",
    )?;
    // Parse arguments before connecting: a typo'd address should fail instantly, not
    // after a round-trip.
    let address = parse_address(&cli.address)?;
    let tx_hash = cli.tx.as_deref().map(parse_tx_hash).transpose()?;

    let client = Client::connect(&rpc_url)?;

    // Balance and nonce are independent; issue them together rather than serially.
    let (balance, nonce) = tokio::try_join!(client.balance(address), client.nonce(address))?;

    println!("address: {}", address.to_checksum(None));
    println!("balance: {balance} wei");
    println!("nonce:   {nonce}");

    if let Some(hash) = tx_hash {
        let tx = client
            .transaction(hash)
            .await?
            .with_context(|| format!("no transaction found for hash {hash}"))?;

        println!();
        println!("tx hash: {}", tx.hash);
        println!("value:   {} wei", tx.value);
        println!("gas:     {}", tx.gas);
    }

    Ok(())
}

/// D3: the input is self-declaring. All-lowercase and all-uppercase claim no checksum and
/// are accepted as-is; mixed case *is* an EIP-55 checksum claim, so it gets validated.
/// Accepting a mixed-case address without checking is how a typo'd address silently
/// becomes a burn address.
fn parse_address(input: &str) -> Result<Address> {
    let hex = input.strip_prefix("0x").unwrap_or(input);
    if hex.len() != 40 {
        bail!(
            "address must be 40 hex characters (20 bytes), got {}",
            hex.len()
        );
    }

    let address: Address = input
        .parse()
        .map_err(|e| anyhow::anyhow!("{e:?}"))
        .with_context(|| format!("`{input}` is not a valid address"))?;

    let has_lower = hex.chars().any(|c| c.is_ascii_lowercase());
    let has_upper = hex.chars().any(|c| c.is_ascii_uppercase());

    if has_lower && has_upper {
        let expected = address.to_checksum(None);
        let given = if input.starts_with("0x") {
            input.to_string()
        } else {
            format!("0x{input}")
        };
        if expected != given {
            bail!("invalid EIP-55 checksum for `{given}` — did you mean `{expected}`?");
        }
    }

    Ok(address)
}

fn parse_tx_hash(input: &str) -> Result<B256> {
    let hex = input.strip_prefix("0x").unwrap_or(input);
    if hex.len() != 64 {
        bail!(
            "transaction hash must be 64 hex characters (32 bytes), got {}",
            hex.len()
        );
    }
    input
        .parse()
        .map_err(|e| anyhow::anyhow!("{e:?}"))
        .with_context(|| format!("`{input}` is not a valid transaction hash"))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A real mainnet address in its canonical EIP-55 form.
    const CHECKSUMMED: &str = "0x5aAeb6053F3E94C9b9A09f33669435E7Ef1BeAed";

    #[test]
    fn accepts_all_lowercase_without_checking() {
        // No checksum is claimed, so none is validated. This is the industry default —
        // rejecting it would break every lowercase address in every config file.
        assert!(parse_address(&CHECKSUMMED.to_lowercase()).is_ok());
    }

    #[test]
    fn accepts_all_uppercase_without_checking() {
        let upper = format!("0x{}", CHECKSUMMED[2..].to_uppercase());
        assert!(parse_address(&upper).is_ok());
    }

    #[test]
    fn accepts_a_correct_mixed_case_checksum() {
        assert!(parse_address(CHECKSUMMED).is_ok());
    }

    #[test]
    fn rejects_a_wrong_mixed_case_checksum() {
        // Flip one character's case. Still valid hex, still the same 20 bytes, but the
        // checksum no longer matches — which is exactly the typo EIP-55 exists to catch.
        let mut chars: Vec<char> = CHECKSUMMED.chars().collect();
        chars[3] = chars[3].to_ascii_uppercase();
        let tampered: String = chars.into_iter().collect();

        let err = parse_address(&tampered).unwrap_err().to_string();
        assert!(err.contains("checksum"), "unhelpful error: {err}");
    }

    #[test]
    fn rejects_wrong_length_address() {
        assert!(parse_address("0xdeadbeef").is_err());
        assert!(parse_address(&format!("{CHECKSUMMED}00")).is_err());
    }

    #[test]
    fn parses_tx_hash_with_and_without_prefix() {
        let hex = "a".repeat(64);
        assert_eq!(
            parse_tx_hash(&format!("0x{hex}")).unwrap(),
            parse_tx_hash(&hex).unwrap()
        );
    }

    #[test]
    fn rejects_wrong_length_tx_hash() {
        assert!(parse_tx_hash("0xdeadbeef").is_err());
        assert!(parse_tx_hash(&"a".repeat(63)).is_err());
    }
}
