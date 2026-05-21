use anyhow::{Context, Result};
use solana_sdk::{
    commitment_config::CommitmentConfig,
    pubkey::Pubkey,
    signature::{Keypair, read_keypair_file},
};
use std::str::FromStr;

/// SPL Token (legacy) program ID
pub const SPL_TOKEN_PROGRAM_ID: &str = "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA";

/// Token-2022 (Token Extensions) program ID
pub const TOKEN_2022_PROGRAM_ID: &str = "TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb";

/// Default devnet RPC
pub const DEFAULT_RPC: &str = "https://api.devnet.solana.com";

/// Compute unit limit we request per transaction.
/// High enough to never fail; we read the *actual* usage from metadata.
pub const CU_LIMIT: u32 = 400_000;

pub struct Config {
    pub rpc_url: String,
    pub payer: Keypair,
    pub commitment: CommitmentConfig,
    pub transfer_count: usize,
    pub output_json: Option<String>,
}

impl Config {
    pub fn spl_token_program_id(&self) -> Pubkey {
        Pubkey::from_str(SPL_TOKEN_PROGRAM_ID).unwrap()
    }

    pub fn token_2022_program_id(&self) -> Pubkey {
        Pubkey::from_str(TOKEN_2022_PROGRAM_ID).unwrap()
    }
}

/// Load keypair from a file path, or generate a fresh one for devnet use.
pub fn load_or_generate_keypair(path: Option<&str>) -> Result<Keypair> {
    match path {
        Some(p) => {
            read_keypair_file(p)
                .map_err(|e| anyhow::anyhow!("Failed to read keypair from {}: {}", p, e))
        }
        None => {
            // Try the default Solana CLI keypair location
            let default_path = shellexpand::tilde("~/.config/solana/id.json").to_string();
            if std::path::Path::new(&default_path).exists() {
                read_keypair_file(&default_path)
                    .map_err(|e| anyhow::anyhow!("Failed to read default keypair: {}", e))
            } else {
                println!(
                    "  No keypair found — generating a fresh one for this run.\n  \
                     Remember to airdrop SOL before benchmarking on devnet!"
                );
                Ok(Keypair::new())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;
    use solana_sdk::pubkey::Pubkey;

    #[test]
    fn spl_token_program_id_is_valid_pubkey() {
        assert!(
            Pubkey::from_str(SPL_TOKEN_PROGRAM_ID).is_ok(),
            "SPL Token program ID must be a valid base58 pubkey"
        );
    }

    #[test]
    fn token_2022_program_id_is_valid_pubkey() {
        assert!(
            Pubkey::from_str(TOKEN_2022_PROGRAM_ID).is_ok(),
            "Token-2022 program ID must be a valid base58 pubkey"
        );
    }

    #[test]
    fn program_ids_are_distinct() {
        assert_ne!(
            SPL_TOKEN_PROGRAM_ID,
            TOKEN_2022_PROGRAM_ID,
            "SPL Token and Token-2022 must be different programs"
        );
    }

    #[test]
    fn default_rpc_is_devnet() {
        assert!(
            DEFAULT_RPC.contains("devnet"),
            "Default RPC must point to devnet for safe testing"
        );
    }
}