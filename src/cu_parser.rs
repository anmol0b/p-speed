use anyhow::{anyhow, Result};
use solana_client::rpc_client::RpcClient;
use solana_sdk::signature::Signature;
use solana_transaction_status::UiTransactionEncoding;

/// Data pulled from a confirmed transaction's on-chain metadata.
pub struct ConfirmedTxData {
    /// Actual compute units consumed — NOT the limit we requested.
    /// This is what Solana Explorer shows as "Compute units consumed".
    pub compute_units_consumed: u64,
    /// Transaction fee in lamports.
    pub fee_lamports: u64,
}

/// Fetch a confirmed transaction and extract real CU + fee.
///
/// This reads `transaction.meta.compute_units_consumed` — the actual
/// on-chain value, not the ComputeBudget limit we set.
/// Available on devnet and mainnet for all confirmed transactions.
pub fn fetch_tx_data(client: &RpcClient, signature: &Signature) -> Result<ConfirmedTxData> {
    let tx = client
        .get_transaction(signature, UiTransactionEncoding::Json)
        .map_err(|e| anyhow!("get_transaction failed for {}: {}", signature, e))?;

    let meta = tx.transaction.meta
        .ok_or_else(|| anyhow!("No metadata for TX {}", signature))?;

    let compute_units_consumed = match meta.compute_units_consumed {
        solana_transaction_status::option_serializer::OptionSerializer::Some(cu) => cu,
        _ => return Err(anyhow!(
            "compute_units_consumed missing for TX {}.\n\
             Make sure you are using devnet or mainnet — local validators \
             may not return this field unless started with --limit-ledger-size.",
            signature
        )),
    };

    Ok(ConfirmedTxData {
        compute_units_consumed,
        fee_lamports: meta.fee,
    })
}