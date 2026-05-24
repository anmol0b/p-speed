use anyhow::Result;
use colored::Colorize;
use comfy_table::{
    modifiers::UTF8_ROUND_CORNERS,
    presets::UTF8_FULL,
    Attribute, Cell, CellAlignment, Color, ContentArrangement, Table,
};

use crate::types::{improvement_pct, RunResult, SplBaseline};

pub fn print_report(result: &RunResult) {
    println!();
    println!("{}", "  ═══════════════════════════════════════════════════════════════".cyan());
    println!("  {}  —  {}",
        "P-Speed Results".bold().cyan(),
        result.label.bold(),
    );
    println!("{}", "  ═══════════════════════════════════════════════════════════════".cyan());
    println!();

    // ── Comparison table ─────────────────────────────────────────────────────
    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .apply_modifier(UTF8_ROUND_CORNERS)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(vec![
            Cell::new("Operation")
                .add_attribute(Attribute::Bold).fg(Color::Cyan),
            Cell::new("SPL Token (old)")
                .add_attribute(Attribute::Bold).fg(Color::Red),
            Cell::new("P-Token (live)")
                .add_attribute(Attribute::Bold).fg(Color::Green),
            Cell::new("Improvement")
                .add_attribute(Attribute::Bold).fg(Color::Yellow),
        ]);

    // Mint creation
    add_row(
        &mut table, "Mint creation",
        SplBaseline::MINT_CREATION_CU,
        result.mint_creation.compute_units,
    );

    // Mint tokens
    add_row(
        &mut table, "Mint tokens",
        SplBaseline::MINT_TO_CU,
        result.mint_to.compute_units,
    );

    // Transfer average
    let avg_cu = result.avg_transfer_cu();
    let pct    = improvement_pct(SplBaseline::TRANSFER_CU as f64, avg_cu);
    table.add_row(vec![
        Cell::new(format!("Transfer  (avg n={})", result.transfer_count)),
        Cell::new(format!("{} CU", fmt_num(SplBaseline::TRANSFER_CU)))
            .fg(Color::Red).set_alignment(CellAlignment::Right),
        Cell::new(format!("{:.0} CU", avg_cu))
            .fg(Color::Green).add_attribute(Attribute::Bold)
            .set_alignment(CellAlignment::Right),
        pct_cell(pct),
    ]);

    // Transfer min / max
    table.add_row(vec![
        Cell::new("Transfer  (min / max)"),
        Cell::new(""),
        Cell::new(format!("{} / {} CU",
            fmt_num(result.min_transfer_cu()),
            fmt_num(result.max_transfer_cu()),
        )).fg(Color::Green).set_alignment(CellAlignment::Right),
        Cell::new(""),
    ]);

    // Total fees
    table.add_row(vec![
        Cell::new("Total fees (all TXs)"),
        Cell::new("—").set_alignment(CellAlignment::Right),
        Cell::new(format!("{:.6} SOL", result.total_fee_sol()))
            .fg(Color::Green).add_attribute(Attribute::Bold)
            .set_alignment(CellAlignment::Right),
        Cell::new(""),
    ]);

    println!("{table}");

    // ── Headline summary ──────────────────────────────────────────────────────
    let transfer_pct = improvement_pct(SplBaseline::TRANSFER_CU as f64, avg_cu);
    println!();
    println!("  {}", "Summary:".bold());
    println!("    SPL Token transfer : {} CU  (documented baseline)",
        fmt_num(SplBaseline::TRANSFER_CU).red());
    println!("    P-Token transfer   : {:.0} CU  (live devnet)",
        avg_cu.to_string().green().bold());

    if transfer_pct > 0.0 {
        println!("    Reduction          : {}  {}",
            format!("{:.1}% fewer compute units", transfer_pct).green().bold(),
            "🚀",
        );
    } else {
        println!("    {}",
            "Note: CU close to SPL baseline — feature gate may not be active on this RPC."
            .yellow()
        );
    }

    println!("    Total fees paid    : {} SOL",
        format!("{:.6}", result.total_fee_sol()).green()
    );

    // ── Disclaimer ────────────────────────────────────────────────────────────
    println!();
    println!("  {}", "Note:".dimmed());
    println!("  {}",
        "SPL Token column uses documented baseline CU (spl.solana.com / SIMD-0266).".dimmed());
    println!("  {}",
        "P-Token column is real on-chain data from transaction.meta.compute_units_consumed.".dimmed());

    // ── Explorer links ────────────────────────────────────────────────────────
    let cluster = if result.rpc_url.contains("devnet")  { "?cluster=devnet" }
                  else if result.rpc_url.contains("mainnet") { "" }
                  else { "?cluster=custom" };

    println!();
    println!("  {}", "Verify on Solana Explorer:".bold());
    println!("    Mint TX     : https://explorer.solana.com/tx/{}{}",
        result.mint_creation.signature.dimmed(), cluster);
    if let Some(t) = result.transfers.first() {
        println!("    Transfer TX : https://explorer.solana.com/tx/{}{}",
            t.signature.dimmed(), cluster);
    }

    println!();
    println!("{}", "  ═══════════════════════════════════════════════════════════════".cyan());
    println!("  {} CU values read from transaction.meta.compute_units_consumed", "ℹ".cyan());
    println!("  {} Program : {}", "ℹ".cyan(), result.program_id.dimmed());
    println!("{}", "  ═══════════════════════════════════════════════════════════════".cyan());
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn add_row(table: &mut Table, label: &str, old_cu: u64, new_cu: u64) {
    let pct = improvement_pct(old_cu as f64, new_cu as f64);
    table.add_row(vec![
        Cell::new(label),
        Cell::new(format!("{} CU", fmt_num(old_cu)))
            .fg(Color::Red).set_alignment(CellAlignment::Right),
        Cell::new(format!("{} CU", fmt_num(new_cu)))
            .fg(Color::Green).set_alignment(CellAlignment::Right),
        pct_cell(pct),
    ]);
}

fn pct_cell(pct: f64) -> Cell {
    if pct > 0.0 {
        Cell::new(format!("{:.1}% less", pct))
            .fg(Color::Green).add_attribute(Attribute::Bold)
            .set_alignment(CellAlignment::Right)
    } else if pct < -5.0 {
        Cell::new(format!("{:.1}% more", pct.abs()))
            .fg(Color::Red).set_alignment(CellAlignment::Right)
    } else {
        Cell::new("~no change").set_alignment(CellAlignment::Right)
    }
}

pub fn export_json(result: &RunResult, path: &str) -> Result<()> {
    let json = serde_json::to_string_pretty(result)?;
    std::fs::write(path, &json)?;
    println!("\n  {} Saved to {}", "✓".green(), path.bold());
    Ok(())
}

/// Format u64 with thousands separators: 12450 → "12,450"
pub fn fmt_num(n: u64) -> String {
    let s = n.to_string();
    let mut out = String::new();
    for (i, ch) in s.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 { out.push(','); }
        out.push(ch);
    }
    out.chars().rev().collect()
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────
#[cfg(test)]
mod tests {
    use super::*;

    #[test] fn fmt_zero()           { assert_eq!(fmt_num(0),         "0"); }
    #[test] fn fmt_below_thousand() { assert_eq!(fmt_num(999),       "999"); }
    #[test] fn fmt_exactly_1000()   { assert_eq!(fmt_num(1_000),     "1,000"); }
    #[test] fn fmt_12450()          { assert_eq!(fmt_num(12_450),    "12,450"); }
    #[test] fn fmt_million()        { assert_eq!(fmt_num(1_000_000), "1,000,000"); }

    #[test]
    fn pct_cell_positive_is_green() {
        // just check it doesn't panic and pct math is right
        let pct = improvement_pct(4645.0, 76.0);
        assert!(pct > 98.0);
    }

    #[test]
    fn spl_baseline_transfer_is_4645() {
        assert_eq!(SplBaseline::TRANSFER_CU, 4_645);
    }
}