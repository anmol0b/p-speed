use serde::{Deserialize, Serialize};

/// Metrics captured for a single confirmed transaction.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TxMetrics {
    pub signature:     String,
    pub compute_units: u64,
    pub elapsed_ms:    u128,
    pub fee_lamports:  u64,
}

impl TxMetrics {
    #[allow(dead_code)]
    pub fn fee_sol(&self) -> f64 {
        self.fee_lamports as f64 / 1_000_000_000.0
    }
}

/// All metrics for one full benchmark run.
/// On devnet/mainnet  → P-Token bytecode  (feature gate ON)
/// On local validator → old SPL bytecode  (feature gate OFF)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunResult {
    pub label:          String,
    pub rpc_url:        String,
    pub program_id:     String,
    pub transfer_count: usize,
    pub mint_creation:  TxMetrics,
    pub ata_creation:   TxMetrics,
    pub mint_to:        TxMetrics,
    pub transfers:      Vec<TxMetrics>,
}

impl RunResult {
    pub fn avg_transfer_cu(&self) -> f64 {
        if self.transfers.is_empty() { return 0.0; }
        let sum: u64 = self.transfers.iter().map(|t| t.compute_units).sum();
        sum as f64 / self.transfers.len() as f64
    }

    pub fn min_transfer_cu(&self) -> u64 {
        self.transfers.iter().map(|t| t.compute_units).min().unwrap_or(0)
    }

    pub fn max_transfer_cu(&self) -> u64 {
        self.transfers.iter().map(|t| t.compute_units).max().unwrap_or(0)
    }

    #[allow(dead_code)]
    pub fn avg_transfer_ms(&self) -> f64 {
        if self.transfers.is_empty() { return 0.0; }
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

/// Two live RunResults side by side — both columns real, nothing hardcoded.
/// spl  = local validator run (feature gate OFF) → old SPL bytecode
/// ptoken = devnet/mainnet run (feature gate ON) → P-Token bytecode
#[derive(Debug, Serialize, Deserialize)]
pub struct CompareResult {
    pub spl:    RunResult,
    pub ptoken: RunResult,
}

/// Percentage improvement: positive = new uses fewer CU (better).
pub fn improvement_pct(old: f64, new: f64) -> f64 {
    if old == 0.0 { return 0.0; }
    ((old - new) / old) * 100.0
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────
#[cfg(test)]
mod tests {
    use super::*;

    fn make_metrics(cu: u64, elapsed_ms: u128, fee_lamports: u64) -> TxMetrics {
        TxMetrics { signature: "testsig".into(), compute_units: cu, elapsed_ms, fee_lamports }
    }

    fn make_run(transfer_cus: Vec<u64>, fee_each: u64) -> RunResult {
        RunResult {
            label:          "test".into(),
            rpc_url:        "http://localhost".into(),
            program_id:     "testprog".into(),
            transfer_count: transfer_cus.len(),
            mint_creation:  make_metrics(1000, 800, fee_each),
            ata_creation:   make_metrics(900,  750, fee_each),
            mint_to:        make_metrics(800,  700, fee_each),
            transfers: transfer_cus.into_iter()
                .map(|cu| make_metrics(cu, 600, fee_each))
                .collect(),
        }
    }

    #[test]
    fn fee_sol_zero() { assert_eq!(make_metrics(100, 200, 0).fee_sol(), 0.0); }

    #[test]
    fn fee_sol_one_sol() {
        let m = make_metrics(100, 200, 1_000_000_000);
        assert!((m.fee_sol() - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn fee_sol_typical() {
        let m = make_metrics(2000, 400, 5_000);
        assert!((m.fee_sol() - 5_000.0 / 1e9).abs() < 1e-12);
    }

    #[test]
    fn avg_cu_single() {
        assert!((make_run(vec![300], 5000).avg_transfer_cu() - 300.0).abs() < f64::EPSILON);
    }

    #[test]
    fn avg_cu_multiple() {
        assert!((make_run(vec![100, 200, 300], 5000).avg_transfer_cu() - 200.0).abs() < f64::EPSILON);
    }

    #[test]
    fn avg_cu_empty() { assert_eq!(make_run(vec![], 5000).avg_transfer_cu(), 0.0); }

    #[test]
    fn min_max_cu() {
        let r = make_run(vec![50, 200, 130], 5000);
        assert_eq!(r.min_transfer_cu(), 50);
        assert_eq!(r.max_transfer_cu(), 200);
    }

    #[test]
    fn total_fee_counts_all_txs() {
        let r = make_run(vec![100, 200], 5_000);
        assert_eq!(r.total_fee_lamports(), 5 * 5_000);
    }

    #[test]
    fn total_fee_sol_consistent() {
        let r = make_run(vec![100], 5_000);
        let expected = r.total_fee_lamports() as f64 / 1e9;
        assert!((r.total_fee_sol() - expected).abs() < 1e-12);
    }

    #[test]
    fn improvement_half() { assert!((improvement_pct(100.0, 50.0) - 50.0).abs() < f64::EPSILON); }

    #[test]
    fn improvement_full() { assert!((improvement_pct(100.0, 0.0) - 100.0).abs() < f64::EPSILON); }

    #[test]
    fn improvement_none() { assert!((improvement_pct(100.0, 100.0) - 0.0).abs() < f64::EPSILON); }

    #[test]
    fn improvement_regression() {
        assert!((improvement_pct(100.0, 150.0) - (-50.0)).abs() < f64::EPSILON);
    }

    #[test]
    fn improvement_zero_old_no_panic() { assert_eq!(improvement_pct(0.0, 50.0), 0.0); }

    #[test]
    fn improvement_realistic_p_token() {
        let pct = improvement_pct(4645.0, 255.0);
        assert!(pct > 93.0 && pct < 96.0, "Expected ~94.5%, got {:.1}%", pct);
    }
}