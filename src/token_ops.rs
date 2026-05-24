use anyhow::{anyhow, Result};
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    commitment_config::CommitmentConfig,
    compute_budget::ComputeBudgetInstruction,
    instruction::Instruction,
    pubkey::Pubkey,
    signature::{Keypair, Signature, Signer},
    system_instruction,
    transaction::Transaction,
};
use spl_associated_token_account::{
    get_associated_token_address_with_program_id,
    instruction::create_associated_token_account,
};
use solana_sdk::program_pack::Pack;
use spl_token::{instruction as token_ix, state::Mint};
use std::time::{Duration, Instant};

use crate::{config::CU_LIMIT, cu_parser::fetch_tx_data, types::TxMetrics};

// ── Core send helper ────────────────────────────────────────────────────────

/// Build, sign, send, and confirm a transaction.
/// Prepends a ComputeBudget instruction so TXs never fail on CU limit.
/// Returns (signature, elapsed_ms).
pub fn send_and_confirm(
    client:       &RpcClient,
    instructions: &[Instruction],
    signers:      &[&Keypair],
    payer:        &Keypair,
) -> Result<(Signature, u128)> {
    let blockhash = client.get_latest_blockhash()
        .map_err(|e| anyhow!("get_latest_blockhash: {}", e))?;

    let mut ixs = vec![ComputeBudgetInstruction::set_compute_unit_limit(CU_LIMIT)];
    ixs.extend_from_slice(instructions);

    let tx = Transaction::new_signed_with_payer(
        &ixs,
        Some(&payer.pubkey()),
        signers,
        blockhash,
    );

    let start = Instant::now();
    let sig = client
        .send_and_confirm_transaction_with_spinner_and_commitment(
            &tx,
            CommitmentConfig::confirmed(),
        )
        .map_err(|e| anyhow!("Transaction failed: {}", e))?;

    Ok((sig, start.elapsed().as_millis()))
}

/// Confirm a signature and build a TxMetrics from on-chain metadata.
pub fn collect_metrics(
    client:     &RpcClient,
    signature:  Signature,
    elapsed_ms: u128,
) -> Result<TxMetrics> {
    let data = fetch_tx_data(client, &signature)?;
    Ok(TxMetrics {
        signature:     signature.to_string(),
        compute_units: data.compute_units_consumed,
        elapsed_ms,
        fee_lamports:  data.fee_lamports,
    })
}

// ── Token operations ─────────────────────────────────────────────────────────

/// Create and initialize a new mint account.
pub fn create_mint(
    client:           &RpcClient,
    payer:            &Keypair,
    mint_keypair:     &Keypair,
    decimals:         u8,
    token_program_id: &Pubkey,
) -> Result<TxMetrics> {
    let rent = client
        .get_minimum_balance_for_rent_exemption(Mint::LEN)
        .map_err(|e| anyhow!("get rent: {}", e))?;

    let ixs = vec![
        system_instruction::create_account(
            &payer.pubkey(),
            &mint_keypair.pubkey(),
            rent,
            Mint::LEN as u64,
            token_program_id,
        ),
        token_ix::initialize_mint(
            token_program_id,
            &mint_keypair.pubkey(),
            &payer.pubkey(), // mint authority
            None,            // freeze authority
            decimals,
        ).map_err(|e| anyhow!("initialize_mint ix: {}", e))?,
    ];

    let (sig, ms) = send_and_confirm(client, &ixs, &[payer, mint_keypair], payer)?;
    collect_metrics(client, sig, ms)
}

/// Create an Associated Token Account for `owner`.
pub fn create_ata(
    client:           &RpcClient,
    payer:            &Keypair,
    owner:            &Pubkey,
    mint:             &Pubkey,
    token_program_id: &Pubkey,
) -> Result<(Pubkey, TxMetrics)> {
    let ata = get_associated_token_address_with_program_id(owner, mint, token_program_id);

    let ix = create_associated_token_account(
        &payer.pubkey(), // funding account (pays rent)
        owner,           // wallet that will own the ATA
        mint,
        token_program_id,
    );

    let (sig, ms) = send_and_confirm(client, &[ix], &[payer], payer)?;
    let metrics = collect_metrics(client, sig, ms)?;
    Ok((ata, metrics))
}

/// Mint tokens into a destination ATA.
pub fn mint_tokens(
    client:           &RpcClient,
    payer:            &Keypair,
    mint:             &Pubkey,
    destination:      &Pubkey,
    amount:           u64,
    token_program_id: &Pubkey,
) -> Result<TxMetrics> {
    let ix = token_ix::mint_to(
        token_program_id,
        mint,
        destination,
        &payer.pubkey(), // mint authority
        &[],
        amount,
    ).map_err(|e| anyhow!("mint_to ix: {}", e))?;

    let (sig, ms) = send_and_confirm(client, &[ix], &[payer], payer)?;
    collect_metrics(client, sig, ms)
}

/// Transfer tokens between two ATAs using transfer_checked (spl-token 4.x).
pub fn transfer_tokens(
    client:           &RpcClient,
    payer:            &Keypair,
    source:           &Pubkey,
    destination:      &Pubkey,
    mint:             &Pubkey,
    amount:           u64,
    decimals:         u8,
    token_program_id: &Pubkey,
) -> Result<TxMetrics> {
    let ix = token_ix::transfer_checked(
        token_program_id,
        source,
        mint,
        destination,
        &payer.pubkey(), // authority
        &[],
        amount,
        decimals,
    ).map_err(|e| anyhow!("transfer_checked ix: {}", e))?;

    let (sig, ms) = send_and_confirm(client, &[ix], &[payer], payer)?;
    collect_metrics(client, sig, ms)
}

pub fn sleep_ms(ms: u64) {
    std::thread::sleep(Duration::from_millis(ms));
}