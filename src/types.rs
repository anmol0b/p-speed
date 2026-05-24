use serde::{Deserialize, Serialize};

/// Metrics captured for a single confirmed transaction.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TxMetrics {
    /// Transaction signature (base58) — paste into Explorer to verify
    pub signature: String,
    /// Compute units actually consumed — read from transaction.meta.compute_units_consumed
    pub compute_units: u64,
    /// Wall-clock time from send → confirmation in milliseconds
    pub elapsed_ms: u128,
    /// Fee paid in lamports
    pub fee_lamports: u64,
}

impl TxMetrics {
    #[allow(dead_code)]

    pub fn fee_sol(&self) -> f64 {
        self.fee_lamports as f64 / 1_000_000_000.0
    }
}

/// All metrics for one full benchmark run (mint + ATA + mint_to + N transfers).
/// On devnet this is P-Token (SIMD-0266 feature gate is active).
/// On a local validator with the gate disabled this is old SPL Token.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunResult {
    /// Human-readable label, e.g. "P-Token (devnet)" or "SPL Token (local)"
    pub label: String,
    /// The RPC endpoint used
    pub rpc_url: String,
    /// The token program ID used (same address on both — runtime swaps the bytecode)
    pub program_id: String,
    /// Number of transfers run
    pub transfer_count: usize,

    pub mint_creation: TxMetrics,
    pub ata_creation:  TxMetrics,
    pub mint_to:       TxMetrics,
    pub transfers:     Vec<TxMetrics>,
}

impl RunResult {
    #[allow(dead_code)]
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

/// Utility: compute improvement percentage between old and new value.
/// Positive = new is better (fewer CU). Negative = regression.
pub fn improvement_pct(old: f64, new: f64) -> f64 {
    if old == 0.0 { return 0.0; }
    ((old - new) / old) * 100.0
}

// ─────────────────────────────────────────────────────────────────────────────
// Unit tests — pure logic, no RPC, runs offline instantly
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

    // ── TxMetrics ──────────────────────────────────────────────────────────

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

    // ── RunResult aggregation ──────────────────────────────────────────────

    #[test]
    fn avg_cu_single() {
        assert!((make_run(vec![300], 5000).avg_transfer_cu() - 300.0).abs() < f64::EPSILON);
    }

    #[test]
    fn avg_cu_multiple() {
        // (100+200+300)/3 = 200
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
        // mint + ata + mint_to + 2 transfers = 5 txs
        let r = make_run(vec![100, 200], 5_000);
        assert_eq!(r.total_fee_lamports(), 5 * 5_000);
    }

    #[test]
    fn total_fee_sol_consistent() {
        let r = make_run(vec![100], 5_000);
        let expected = r.total_fee_lamports() as f64 / 1e9;
        assert!((r.total_fee_sol() - expected).abs() < 1e-12);
    }

    // ── improvement_pct ────────────────────────────────────────────────────

    #[test]
    fn improvement_half()     { assert!((improvement_pct(100.0, 50.0)  - 50.0).abs() < f64::EPSILON); }

    #[test]
    fn improvement_full()     { assert!((improvement_pct(100.0, 0.0)   - 100.0).abs() < f64::EPSILON); }

    #[test]
    fn improvement_none()     { assert!((improvement_pct(100.0, 100.0) - 0.0).abs() < f64::EPSILON); }

    #[test]
    fn improvement_regression() {
        // new > old → negative %
        assert!((improvement_pct(100.0, 150.0) - (-50.0)).abs() < f64::EPSILON);
    }

    #[test]
    fn improvement_zero_old_no_panic() { assert_eq!(improvement_pct(0.0, 50.0), 0.0); }

    #[test]
    fn improvement_realistic_p_token() {
        // Old SPL ~4645 CU, P-Token ~76 CU → ~98.4% improvement
        let pct = improvement_pct(4645.0, 76.0);
        assert!(pct > 95.0 && pct < 99.5, "Expected ~98.4%, got {:.1}%", pct);
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Old SPL Token baseline — publicly documented CU numbers
// Source: https://spl.solana.com / Anza SIMD-0266 announcement
// These are the numbers P-Token replaces.
// ─────────────────────────────────────────────────────────────────────────────
pub struct SplBaseline;

impl SplBaseline {
    pub const MINT_CREATION_CU: u64 = 2_967;
    pub const MINT_TO_CU:       u64 = 2_921;
    pub const TRANSFER_CU:      u64 = 4_645; // per transfer, the headline number
}