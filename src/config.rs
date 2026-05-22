use anyhow::{anyhow, Result};
use solana_sdk::{
    commitment_config::CommitmentConfig,
    pubkey::Pubkey,
    signature::{read_keypair_file, Keypair},
};
use std::str::FromStr;

// ── Program IDs ────────────────────────────────────────────────────────────
//
// P-Token (SIMD-0266) is a DROP-IN replacement for SPL Token.
// It lives at the SAME address as the old SPL Token program.
// The Solana runtime swaps the bytecode transparently when the feature gate
// (ptokFjwyJtrwCa9Kgo9xoDS59V4QccBGEaRFnRPnSdP) is active.
//
// So on devnet/mainnet (feature gate ON):
//   TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA  ← P-Token bytecode
//
// On a local validator with feature gate OFF:
//   TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA  ← old SPL Token bytecode
//
// Client code is IDENTICAL — only the RPC endpoint changes.
// That is the whole point of SIMD-0266: zero client-side changes.

/// The token program address (same for both old SPL and new P-Token)
pub const TOKEN_PROGRAM_ID: &str = "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA";

/// Devnet RPC — P-Token (SIMD-0266 feature gate is active here)
pub const DEVNET_RPC: &str = "https://api.devnet.solana.com";

/// Mainnet RPC — P-Token is live from epoch 971 onward
pub const MAINNET_RPC: &str = "https://api.mainnet-beta.solana.com";

/// Local validator — old SPL Token behavior (feature gate not active by default)
pub const LOCALNET_RPC: &str = "http://127.0.0.1:8899";

/// Default: devnet
pub const DEFAULT_RPC: &str = DEVNET_RPC;

/// How many CU to request per transaction.
/// We set this high so TXs never fail.
/// The actual usage is read from transaction.meta.compute_units_consumed.
pub const CU_LIMIT: u32 = 400_000;

pub struct Config {
    pub rpc_url:        String,
    pub payer:          Keypair,
    pub commitment:     CommitmentConfig,
    pub transfer_count: usize,
    pub output_json:    Option<String>,
    pub label:          String, // e.g. "P-Token (devnet)"
}

impl Config {
    pub fn token_program_id(&self) -> Pubkey {
        Pubkey::from_str(TOKEN_PROGRAM_ID).unwrap()
    }
}

/// Resolve a short alias ("devnet", "mainnet", "local") or a full URL.
pub fn resolve_rpc(input: &str) -> String {
    match input.to_lowercase().as_str() {
        "devnet"  | "dev"   => DEVNET_RPC.to_string(),
        "mainnet" | "main"  => MAINNET_RPC.to_string(),
        "local"   | "localhost" | "localnet" => LOCALNET_RPC.to_string(),
        other => other.to_string(), // treat as a full URL
    }
}

/// Build a human-readable label from the RPC URL.
pub fn label_from_rpc(rpc: &str) -> String {
    if rpc.contains("devnet")    { return "P-Token (devnet)".to_string(); }
    if rpc.contains("mainnet")   { return "P-Token (mainnet)".to_string(); }
    if rpc.contains("127.0.0.1") || rpc.contains("localhost") {
        return "SPL Token (local validator)".to_string();
    }
    format!("P-Token ({})", rpc)
}

/// Load keypair from a file, or fall back to ~/.config/solana/id.json,
/// or generate a fresh throwaway keypair.
pub fn load_or_generate_keypair(path: Option<&str>) -> Result<Keypair> {
    match path {
        Some(p) => read_keypair_file(p)
            .map_err(|e| anyhow!("Failed to read keypair from {}: {}", p, e)),
        None => {
            let default_path = shellexpand::tilde("~/.config/solana/id.json").to_string();
            if std::path::Path::new(&default_path).exists() {
                read_keypair_file(&default_path)
                    .map_err(|e| anyhow!("Failed to read default keypair: {}", e))
            } else {
                println!(
                    "  No keypair found — generating a fresh one.\n  \
                     Run `solana airdrop 2` before benchmarking."
                );
                Ok(Keypair::new())
            }
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────
#[cfg(test)]
mod tests {
    use super::*;
    use solana_sdk::pubkey::Pubkey;

    #[test]
    fn token_program_id_is_valid_pubkey() {
        assert!(Pubkey::from_str(TOKEN_PROGRAM_ID).is_ok());
    }

    #[test]
    fn default_rpc_is_devnet() {
        assert!(DEFAULT_RPC.contains("devnet"));
    }

    #[test]
    fn resolve_rpc_aliases() {
        assert_eq!(resolve_rpc("devnet"),  DEVNET_RPC);
        assert_eq!(resolve_rpc("mainnet"), MAINNET_RPC);
        assert_eq!(resolve_rpc("local"),   LOCALNET_RPC);
    }

    #[test]
    fn resolve_rpc_full_url_passthrough() {
        let url = "https://my-rpc.example.com";
        assert_eq!(resolve_rpc(url), url);
    }

    #[test]
    fn label_devnet() {
        assert!(label_from_rpc(DEVNET_RPC).contains("devnet"));
    }

    #[test]
    fn label_local() {
        assert!(label_from_rpc(LOCALNET_RPC).contains("local"));
    }
}