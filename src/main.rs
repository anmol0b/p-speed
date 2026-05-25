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
    config::{load_or_generate_keypair, resolve_rpc, label_from_rpc, Config, DEFAULT_RPC},
    reporter::{export_json, print_report},
};

#[derive(Parser)]
#[command(
    name    = "p-speed",
    version = "0.1.1",
    author  = "anmol0b",
    about   = "Benchmark real P-Token (SIMD-0266) compute units on Solana",
    long_about = "Benchmark real P-Token (SIMD-0266) compute units on Solana.

P-Token is a drop-in replacement for SPL Token, rebuilt with Pinocchio.
Same program address. Same instructions. 94.5% fewer compute units.

Examples:
  p-speed run
  p-speed run --rpc devnet --transfers 20
  p-speed run --rpc mainnet --transfers 10
  p-speed run --rpc https://your-rpc.com --transfers 20
  p-speed run --rpc devnet --transfers 20 --output results.json

Verify any transaction on Solana Explorer:
  https://explorer.solana.com/tx/<SIGNATURE>?cluster=devnet"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run the P-Token benchmark
    Run {
        /// RPC endpoint — devnet | mainnet | local | or a full URL
        #[arg(
            long,
            default_value = DEFAULT_RPC,
            help = "RPC endpoint: devnet | mainnet | local | or a full URL"
        )]
        rpc: String,

        /// Path to payer keypair file
        #[arg(
            long,
            help = "Path to keypair file (default: ~/.config/solana/id.json)"
        )]
        keypair: Option<String>,

        /// Number of token transfers to benchmark
        #[arg(
            long, short = 'n',
            default_value = "20",
            help = "Number of transfers to run (default: 20)"
        )]
        transfers: usize,

        /// Save full results to a JSON file
        #[arg(
            long,
            help = "Save results to a JSON file (e.g. --output results.json)"
        )]
        output: Option<String>,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

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

    match cli.command {
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
    }

    Ok(())
}