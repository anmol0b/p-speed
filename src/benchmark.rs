use anyhow::{anyhow, Result};
use colored::Colorize;
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    commitment_config::CommitmentConfig,
    native_token::LAMPORTS_PER_SOL,
    signature::{Keypair, Signer},
};

use crate::{
    config::Config,
    token_ops::{create_ata, create_mint, mint_tokens, sleep_ms, transfer_tokens},
    types::RunResult,
};

const MINT_DECIMALS:   u8  = 6;
const MINT_AMOUNT:     u64 = 1_000_000_000; // 1,000 tokens (6 decimals)
const TRANSFER_AMOUNT: u64 = 1_000;          // 0.001 tokens per transfer


pub fn ensure_funded(client: &RpcClient, config: &Config) -> Result<()> {
    let pubkey  = config.payer.pubkey();
    let balance = client.get_balance(&pubkey)
        .map_err(|e| anyhow!("get_balance: {}", e))?;

    let sol = balance as f64 / LAMPORTS_PER_SOL as f64;

    if balance < LAMPORTS_PER_SOL {
        println!("  Balance: {:.4} SOL — requesting airdrop...", sol);
        client.request_airdrop(&pubkey, 2 * LAMPORTS_PER_SOL)
            .map_err(|e| anyhow!(
                "Airdrop failed: {}\nTry manually: solana airdrop 2 --url devnet", e
            ))?;
        println!("  Waiting for airdrop confirmation...");
        sleep_ms(6_000);
        let new_bal = client.get_balance(&pubkey)? as f64 / LAMPORTS_PER_SOL as f64;
        println!("  Balance: {:.4} SOL  {}", new_bal, "✓".green());
    } else {
        println!("  Balance: {:.4} SOL  {}", sol, "✓".green());
    }
    Ok(())
}


/// Run one full benchmark pass:
///   create mint → create ATAs → mint tokens → N transfers
/// Returns a RunResult with real on-chain CU for every transaction.
pub fn run_benchmark(config: &Config) -> Result<RunResult> {
    let client = RpcClient::new_with_commitment(
        config.rpc_url.clone(),
        CommitmentConfig::confirmed(),
    );

    println!();
    println!("{}", "  P-Speed".bold().cyan());
    println!("{}", "  ═══════════════════════════════════════════════".cyan());
    println!("  Network  : {}", config.label.bold());
    println!("  RPC      : {}", config.rpc_url.dimmed());
    println!("  Wallet   : {}", config.payer.pubkey());
    println!("  Transfers: {}", config.transfer_count);
    println!("  Program  : {}", crate::config::TOKEN_PROGRAM_ID.dimmed());
    println!();

    print!("  Checking balance...  ");
    ensure_funded(&client, config)?;

    print!("  Warming up RPC...    ");
    let _ = client.get_slot();
    sleep_ms(800);
    println!("{}", "✓".green());

    let token_program_id = config.token_program_id();

    print!("  [1/4] Creating mint...        ");
    let mint_kp = Keypair::new();
    let mint_metrics = create_mint(
        &client, &config.payer, &mint_kp, MINT_DECIMALS, &token_program_id,
    )?;
    println!("{}  {} CU  ({} ms)",
        "✓".green(),
        format!("{:>7}", mint_metrics.compute_units).yellow().bold(),
        mint_metrics.elapsed_ms,
    );
    sleep_ms(400);

    print!("  [2/4] Creating source ATA...  ");
    let (source_ata, ata_metrics) = create_ata(
        &client, &config.payer, &config.payer.pubkey(),
        &mint_kp.pubkey(), &token_program_id,
    )?;
    println!("{}  {} CU  ({} ms)",
        "✓".green(),
        format!("{:>7}", ata_metrics.compute_units).yellow().bold(),
        ata_metrics.elapsed_ms,
    );
    sleep_ms(400);

    let recipient = Keypair::new();
    let (dest_ata, _) = create_ata(
        &client,
        &config.payer,       // payer covers rent + fee
        &recipient.pubkey(), // recipient owns the ATA
        &mint_kp.pubkey(),
        &token_program_id,
    )?;
    sleep_ms(400);

    print!("  [3/4] Minting tokens...       ");
    let mint_to_metrics = mint_tokens(
        &client, &config.payer, &mint_kp.pubkey(),
        &source_ata, MINT_AMOUNT, &token_program_id,
    )?;
    println!("{}  {} CU  ({} ms)",
        "✓".green(),
        format!("{:>7}", mint_to_metrics.compute_units).yellow().bold(),
        mint_to_metrics.elapsed_ms,
    );
    sleep_ms(400);

    println!("  [4/4] Running {} transfers...", config.transfer_count);
    let mut transfers = Vec::with_capacity(config.transfer_count);

    for i in 0..config.transfer_count {
        print!("        {:>2}/{} transfer...         ",
            i + 1, config.transfer_count);
        let m = transfer_tokens(
            &client,
            &config.payer,
            &source_ata,
            &dest_ata,
            &mint_kp.pubkey(),
            TRANSFER_AMOUNT,
            MINT_DECIMALS,
            &token_program_id,
        )?;
        println!("{}  {} CU  ({} ms)",
            "✓".green(),
            format!("{:>7}", m.compute_units).yellow().bold(),
            m.elapsed_ms,
        );
        transfers.push(m);
        sleep_ms(300);
    }

    Ok(RunResult {
        label:          config.label.clone(),
        rpc_url:        config.rpc_url.clone(),
        program_id:     token_program_id.to_string(),
        transfer_count: config.transfer_count,
        mint_creation:  mint_metrics,
        ata_creation:   ata_metrics,
        mint_to:        mint_to_metrics,
        transfers,
    })
}