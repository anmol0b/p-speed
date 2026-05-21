use serde::{Deserialize, Serialize};

/// Metrics captured for a single confirmed transaction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TxMetrics {
    /// Transaction signature (base58)
    pub signature: String,
    /// Compute units actually consumed (from transaction metadata)
    pub compute_units: u64,
    /// Wall-clock time from send to confirmation, in milliseconds
    pub elapsed_ms: u128,
    /// Transaction fee in lamports
    pub fee_lamports: u64,
}

impl TxMetrics {
    pub fn fee_sol(&self) -> f64 {
        self.fee_lamports as f64 / 1_000_000_000.0
    }
}

/// All metrics collected for one token program (SPL or Token-2022)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgramBenchmark {
    pub program_name: String,
    pub program_id: String,
    pub mint_creation: TxMetrics,
    pub ata_creation: TxMetrics,
    pub mint_to: TxMetrics,
    pub transfers: Vec<TxMetrics>,
}

impl ProgramBenchmark {
    pub fn avg_transfer_cu(&self) -> f64 {
        if self.transfers.is_empty() {
            return 0.0;
        }
        let sum: u64 = self.transfers.iter().map(|t| t.compute_units).sum();
        sum as f64 / self.transfers.len() as f64
    }

    pub fn avg_transfer_ms(&self) -> f64 {
        if self.transfers.is_empty() {
            return 0.0;
        }
        let sum: u128 = self.transfers.iter().map(|t| t.elapsed_ms).sum();
        sum as f64 / self.transfers.len() as f64
    }

    pub fn total_fee_lamports(&self) -> u64 {
        self.mint_creation.fee_lamports
            + self.ata_creation.fee_lamports
            + self.mint_to.fee_lamports
            + self.transfers.iter().map(|t| t.fee_lamports).sum::<u64>()
    }

    pub fn total_fee_sol(&self) -> f64 {
        self.total_fee_lamports() as f64 / 1_000_000_000.0
    }
}

/// Final comparison result — both programs side by side
#[derive(Debug, Serialize, Deserialize)]
pub struct BenchmarkResult {
    pub spl_token: ProgramBenchmark,
    pub token_2022: ProgramBenchmark,
    pub transfer_count: usize,
    pub rpc_url: String,
}

impl BenchmarkResult {
    /// Percentage improvement: how much less does token_2022 use vs spl_token?
    pub fn improvement_pct(old: f64, new: f64) -> f64 {
        if old == 0.0 {
            return 0.0;
        }
        ((old - new) / old) * 100.0
    }
}