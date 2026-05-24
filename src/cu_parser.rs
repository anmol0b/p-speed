use anyhow::{anyhow, Result};
use solana_client::rpc_client::RpcClient;
use solana_sdk::signature::Signature;
use solana_transaction_status::UiTransactionEncoding;
use std::thread::sleep;
use std::time::Duration;

pub struct ConfirmedTxData {
    pub compute_units_consumed: u64,
    pub fee_lamports: u64,
}

/// Fetch a confirmed transaction and extract real CU + fee.
/// Retries up to 5 times with increasing delays — devnet RPC nodes
/// sometimes lag a few seconds before indexing a confirmed transaction.
pub fn fetch_tx_data(client: &RpcClient, signature: &Signature) -> Result<ConfirmedTxData> {
    let max_retries = 5;
    let mut last_err = String::new();

    for attempt in 0..max_retries {
        let wait_ms = 1_000 * (attempt + 1);
        sleep(Duration::from_millis(wait_ms));

        let result = client.get_transaction(signature, UiTransactionEncoding::Json);

        match result {
            Err(e) => {
                last_err = e.to_string();
                // If it's a "not found" type error, retry
                continue;
            }
            Ok(tx) => {
                let meta = tx.transaction.meta
                    .ok_or_else(|| anyhow!("No metadata for TX {}", signature))?;

                let compute_units_consumed = match meta.compute_units_consumed {
                    solana_transaction_status::option_serializer::OptionSerializer::Some(cu) => cu,
                    _ => {
                        last_err = format!(
                            "compute_units_consumed missing for TX {}",
                            signature
                        );
                        continue;
                    }
                };

                return Ok(ConfirmedTxData {
                    compute_units_consumed,
                    fee_lamports: meta.fee,
                });
            }
        }
    }

    Err(anyhow!(
        "Failed to fetch TX {} after {} retries.\nLast error: {}\n\
         This usually means the RPC node is lagging. Try again in a few seconds.",
        signature, max_retries, last_err
    ))
}