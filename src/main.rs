mod benchmark;
mod config;
mod cu_parser;
mod reporter;
mod token_ops;
mod types;

use anyhow::Result;
use clap::{Parser, Subcommand};
use colored::Colorize;

use crate::{
    benchmark::run_benchmark,
    config::{load_or_generate_keypair, resolve_rpc, label_from_rpc, Config, DEFAULT_RPC, LOCALNET_RPC},
    reporter::{export_json, export_json_compare, print_report, print_compare_report},
    types::CompareResult,
};

#[derive(Parser)]
#[command(
    name    = "p-speed",
    version = "0.1.4",
    author  = "anmol0b",
    about   = "Benchmark real P-Token (SIMD-0266) compute units on Solana",
    long_about = "Benchmark real P-Token (SIMD-0266) compute units on Solana.

P-Token is a drop-in replacement for SPL Token, rebuilt with Pinocchio.
Same program address. Same instructions. 94.5% fewer compute units.

Commands:
  run      — measure P-Token CU live on devnet or mainnet
  compare  — measure BOTH sides live (local validator vs devnet)

Examples:
  p-speed run
  p-speed run --rpc devnet --transfers 20
  p-speed run --rpc mainnet --transfers 10
  p-speed run --rpc https://your-rpc.com --transfers 20
  p-speed compare --transfers 20
  p-speed compare --ptoken-rpc mainnet --transfers 20

For compare, start local validator first:
  solana-test-validator --deactivate-feature ptokFjwyJtrwCa9Kgo9xoDS59V4QccBGEaRFnRPnSdP"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Measure P-Token CU live on devnet or mainnet
    Run {
        /// RPC endpoint: devnet | mainnet | local | or a full URL
        #[arg(long, default_value = DEFAULT_RPC,
            help = "RPC endpoint: devnet | mainnet | local | or a full URL")]
        rpc: String,

        /// Path to keypair file (default: ~/.config/solana/id.json)
        #[arg(long)]
        keypair: Option<String>,

        /// Number of transfers to benchmark
        #[arg(long, short = 'n', default_value = "20",
            help = "Number of transfers to run (default: 20)")]
        transfers: usize,

        /// Save results to a JSON file
        #[arg(long, help = "Save results to JSON (e.g. --output results.json)")]
        output: Option<String>,
    },

    /// Measure BOTH sides live — local validator (SPL) vs devnet (P-Token)
    ///
    /// Both columns are real on-chain numbers. Nothing hardcoded.
    ///
    /// Setup (run once in a separate terminal, keep it open):
    ///   solana-test-validator \\
    ///     --deactivate-feature ptokFjwyJtrwCa9Kgo9xoDS59V4QccBGEaRFnRPnSdP
    ///
    /// Setup: solana-test-validator --deactivate-feature ptokFjwyJtrwCa9Kgo9xoDS59V4QccBGEaRFnRPnSdP
    Compare {
        /// RPC for old SPL Token (local validator with feature gate OFF)
        #[arg(long, default_value = LOCALNET_RPC,
            help = "SPL Token RPC — local validator with feature gate OFF (default: http://127.0.0.1:8899)")]
        spl_rpc: String,

        /// RPC for P-Token (devnet or mainnet where feature gate is ON)
        #[arg(long, default_value = DEFAULT_RPC,
            help = "P-Token RPC — devnet or mainnet (default: https://api.devnet.solana.com)")]
        ptoken_rpc: String,

        /// Path to keypair file (default: ~/.config/solana/id.json)
        #[arg(long)]
        keypair: Option<String>,

        /// Number of transfers to benchmark per side
        #[arg(long, short = 'n', default_value = "20",
            help = "Number of transfers per side (default: 20)")]
        transfers: usize,

        /// Save results to a JSON file
        #[arg(long)]
        output: Option<String>,
    },
}

fn banner() {
    println!();
    println!("{}", "  ██████╗       ███████╗██████╗ ███████╗███████╗██████╗".cyan());
    println!("{}", "  ██╔══██╗      ██╔════╝██╔══██╗██╔════╝██╔════╝██╔══██╗".cyan());
    println!("{}", "  ██████╔╝█████╗███████╗██████╔╝█████╗  █████╗  ██║  ██║".cyan());
    println!("{}", "  ██╔═══╝ ╚════╝╚════██║██╔═══╝ ██╔══╝  ██╔══╝  ██║  ██║".cyan());
    println!("{}", "  ██║           ███████║██║     ███████╗███████╗██████╔╝".cyan());
    println!("{}", "  ╚═╝           ╚══════╝╚═╝     ╚══════╝╚══════╝╚═════╝".cyan());
    println!();
    println!("  {}", "P-Token Benchmark Tool — SIMD-0266 in action".bold().white());
    println!();
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    banner();

    match cli.command {
        // ── run ──────────────────────────────────────────────────────────────
        Commands::Run { rpc, keypair, transfers, output } => {
            let rpc_url = resolve_rpc(&rpc);
            let label   = label_from_rpc(&rpc_url);
            let payer   = load_or_generate_keypair(keypair.as_deref())?;

            let config = Config {
                rpc_url,
                payer,
                commitment:     solana_sdk::commitment_config::CommitmentConfig::confirmed(),
                transfer_count: transfers,
                output_json:    output.clone(),
                label,
            };

            let result = run_benchmark(&config)?;
            print_report(&result);

            if let Some(path) = &output {
                export_json(&result, path)?;
            }
        }

        // ── compare ───────────────────────────────────────────────────────────
        Commands::Compare { spl_rpc, ptoken_rpc, keypair, transfers, output } => {
            let spl_url    = resolve_rpc(&spl_rpc);
            let ptoken_url = resolve_rpc(&ptoken_rpc);
            let payer      = load_or_generate_keypair(keypair.as_deref())?;

            // ── SPL Token side (local validator, feature gate OFF) ────────────
            println!("  {} Running SPL Token side (local validator)...",
                "Step 1/2".bold().red());
            let spl_config = Config {
                rpc_url:        spl_url.clone(),
                payer:          solana_sdk::signature::Keypair::from_bytes(&payer.to_bytes())?,
                commitment:     solana_sdk::commitment_config::CommitmentConfig::confirmed(),
                transfer_count: transfers,
                output_json:    None,
                label:          "SPL Token (local validator)".to_string(),
            };
            let spl_result = run_benchmark(&spl_config)?;

            println!();
            println!("  {} Running P-Token side ({})...",
                "Step 2/2".bold().green(),
                label_from_rpc(&ptoken_url).cyan());

            // ── P-Token side (devnet/mainnet, feature gate ON) ────────────────
            let ptoken_config = Config {
                rpc_url:        ptoken_url,
                payer,
                commitment:     solana_sdk::commitment_config::CommitmentConfig::confirmed(),
                transfer_count: transfers,
                output_json:    None,
                label:          label_from_rpc(&resolve_rpc(&ptoken_rpc)),
            };
            let ptoken_result = run_benchmark(&ptoken_config)?;

            let compare = CompareResult {
                spl:    spl_result,
                ptoken: ptoken_result,
            };

            print_compare_report(&compare);

            if let Some(path) = &output {
                export_json_compare(&compare, path)?;
            }
        }
    }

    Ok(())
}