use anyhow::Result;
use colored::Colorize;
use comfy_table::{
    modifiers::UTF8_ROUND_CORNERS,
    presets::UTF8_FULL,
    Attribute, Cell, CellAlignment, Color, ContentArrangement, Table,
};

use crate::types::{improvement_pct, CompareResult, RunResult};

// ── Single run report (p-speed run) ──────────────────────────────────────────

pub fn print_report(result: &RunResult) {
    println!();
    println!("{}", "  ═══════════════════════════════════════════════════════════════".cyan());
    println!("  {}  —  {}",
        "P-Speed Results".bold().cyan(),
        result.label.bold(),
    );
    println!("{}", "  ═══════════════════════════════════════════════════════════════".cyan());
    println!();

    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .apply_modifier(UTF8_ROUND_CORNERS)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(vec![
            Cell::new("Operation").add_attribute(Attribute::Bold).fg(Color::Cyan),
            Cell::new("Compute Units").add_attribute(Attribute::Bold).fg(Color::Yellow),
            Cell::new("Time (ms)").add_attribute(Attribute::Bold).fg(Color::Yellow),
            Cell::new("Fee (SOL)").add_attribute(Attribute::Bold).fg(Color::Yellow),
        ]);

    table.add_row(vec![
        Cell::new("Mint creation"),
        cu_cell(result.mint_creation.compute_units),
        ms_cell(result.mint_creation.elapsed_ms),
        fee_cell(result.mint_creation.fee_lamports),
    ]);
    table.add_row(vec![
        Cell::new("Mint tokens"),
        cu_cell(result.mint_to.compute_units),
        ms_cell(result.mint_to.elapsed_ms),
        fee_cell(result.mint_to.fee_lamports),
    ]);
    table.add_row(vec![
        Cell::new(format!("Transfer avg  (n={})", result.transfer_count)),
        Cell::new(format!("{:.0} CU", result.avg_transfer_cu()))
            .fg(Color::Green).add_attribute(Attribute::Bold)
            .set_alignment(CellAlignment::Right),
        ms_cell(result.transfers.first().map(|t| t.elapsed_ms).unwrap_or(0)),
        fee_cell(result.transfers.first().map(|t| t.fee_lamports).unwrap_or(0)),
    ]);
    table.add_row(vec![
        Cell::new("Transfer  (min / max)"),
        Cell::new(format!("{} / {} CU",
            fmt_num(result.min_transfer_cu()),
            fmt_num(result.max_transfer_cu()),
        )).fg(Color::Green).set_alignment(CellAlignment::Right),
        Cell::new(""), Cell::new(""),
    ]);
    table.add_row(vec![
        Cell::new("Total fees"),
        Cell::new(""),
        Cell::new(""),
        Cell::new(format!("{:.6} SOL", result.total_fee_sol()))
            .fg(Color::Green).add_attribute(Attribute::Bold)
            .set_alignment(CellAlignment::Right),
    ]);

    println!("{table}");

    let avg_cu = result.avg_transfer_cu();
    println!();
    println!("  {}", "Summary:".bold());
    println!("    Transfer CU  : {}",
        format!("{:.0} CU (avg over {} txs)", avg_cu, result.transfer_count).green().bold());
    println!("    Total fees   : {}", format!("{:.6} SOL", result.total_fee_sol()).green());
    println!("    Network      : {}", result.label.cyan().bold());
    println!();
    println!("  {}",
        "Note: run `p-speed compare` to see live SPL Token vs P-Token side by side.".dimmed());

    print_explorer_links(result);
    print_footer(result);
}

// ── Compare report (p-speed compare) — both columns live ─────────────────────

pub fn print_compare_report(result: &CompareResult) {
    println!();
    println!("{}", "  ═══════════════════════════════════════════════════════════════".cyan());
    println!("  {}", "P-Speed Compare — SPL Token vs P-Token".bold().cyan());
    println!("  {}  {}",
        "Both columns are real on-chain measurements.".bold(),
        "Nothing hardcoded.".green().bold(),
    );
    println!("{}", "  ═══════════════════════════════════════════════════════════════".cyan());
    println!();
    println!("  {} : {} (feature gate OFF)",
        "SPL Token".red().bold(), result.spl.rpc_url.dimmed());
    println!("  {} : {} (feature gate ON)",
        "P-Token  ".green().bold(), result.ptoken.rpc_url.dimmed());
    println!();

    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .apply_modifier(UTF8_ROUND_CORNERS)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(vec![
            Cell::new("Operation").add_attribute(Attribute::Bold).fg(Color::Cyan),
            Cell::new("SPL Token (live)").add_attribute(Attribute::Bold).fg(Color::Red),
            Cell::new("P-Token (live)").add_attribute(Attribute::Bold).fg(Color::Green),
            Cell::new("Improvement").add_attribute(Attribute::Bold).fg(Color::Yellow),
        ]);

    // Mint creation
    add_compare_row(
        &mut table, "Mint creation",
        result.spl.mint_creation.compute_units,
        result.ptoken.mint_creation.compute_units,
    );

    // Mint tokens
    add_compare_row(
        &mut table, "Mint tokens",
        result.spl.mint_to.compute_units,
        result.ptoken.mint_to.compute_units,
    );

    // Transfer average
    let spl_avg = result.spl.avg_transfer_cu();
    let pt_avg  = result.ptoken.avg_transfer_cu();
    let pct     = improvement_pct(spl_avg, pt_avg);
    table.add_row(vec![
        Cell::new(format!("Transfer  (avg n={})", result.spl.transfer_count)),
        Cell::new(format!("{:.0} CU", spl_avg))
            .fg(Color::Red).set_alignment(CellAlignment::Right),
        Cell::new(format!("{:.0} CU", pt_avg))
            .fg(Color::Green).add_attribute(Attribute::Bold)
            .set_alignment(CellAlignment::Right),
        pct_cell(pct),
    ]);

    // Transfer min/max
    table.add_row(vec![
        Cell::new("Transfer  (min / max)"),
        Cell::new(format!("{} / {} CU",
            fmt_num(result.spl.min_transfer_cu()),
            fmt_num(result.spl.max_transfer_cu()),
        )).fg(Color::Red).set_alignment(CellAlignment::Right),
        Cell::new(format!("{} / {} CU",
            fmt_num(result.ptoken.min_transfer_cu()),
            fmt_num(result.ptoken.max_transfer_cu()),
        )).fg(Color::Green).set_alignment(CellAlignment::Right),
        Cell::new(""),
    ]);

    // Total fees
    let fee_pct = improvement_pct(
        result.spl.total_fee_sol(),
        result.ptoken.total_fee_sol(),
    );
    table.add_row(vec![
        Cell::new("Total fees (all TXs)"),
        Cell::new(format!("{:.6} SOL", result.spl.total_fee_sol()))
            .fg(Color::Red).set_alignment(CellAlignment::Right),
        Cell::new(format!("{:.6} SOL", result.ptoken.total_fee_sol()))
            .fg(Color::Green).add_attribute(Attribute::Bold)
            .set_alignment(CellAlignment::Right),
        pct_cell(fee_pct),
    ]);

    println!("{table}");

    // Headline summary
    println!();
    println!("  {}", "Summary:".bold());
    println!("    SPL Token transfer : {}  (live — local validator)",
        format!("{:.0} CU", spl_avg).red().bold());
    println!("    P-Token transfer   : {}  (live — {})",
        format!("{:.0} CU", pt_avg).green().bold(),
        result.ptoken.label.cyan());
    if pct > 0.0 {
        println!("    Reduction          : {}  🚀",
            format!("{:.1}% fewer compute units", pct).green().bold());
    }
    println!("    SPL fees paid      : {}", format!("{:.6} SOL", result.spl.total_fee_sol()).red());
    println!("    P-Token fees paid  : {}", format!("{:.6} SOL", result.ptoken.total_fee_sol()).green());

    println!();
    println!("  {}", "Both columns verified on-chain:".bold());
    println!("  {} SPL Token  : {}",
        "ℹ".cyan(), "local validator — feature gate OFF — old bytecode".dimmed());
    println!("  {} P-Token    : {}",
        "ℹ".cyan(), "devnet/mainnet — feature gate ON — Pinocchio bytecode".dimmed());

    // Explorer links for both
    println!();
    println!("  {}", "Verify on Solana Explorer:".bold());

    let pt_cluster = if result.ptoken.rpc_url.contains("devnet") { "?cluster=devnet" }
                     else if result.ptoken.rpc_url.contains("mainnet") { "" }
                     else { "?cluster=custom" };

    println!("  SPL mint TX  : https://explorer.solana.com/tx/{}?cluster=custom",
        result.spl.mint_creation.signature.dimmed());
    println!("  P-Token mint : https://explorer.solana.com/tx/{}{}",
        result.ptoken.mint_creation.signature.dimmed(), pt_cluster);
    if let (Some(s), Some(p)) = (result.spl.transfers.first(), result.ptoken.transfers.first()) {
        println!("  SPL xfer TX  : https://explorer.solana.com/tx/{}?cluster=custom",
            s.signature.dimmed());
        println!("  P-Token xfer : https://explorer.solana.com/tx/{}{}",
            p.signature.dimmed(), pt_cluster);
    }

    println!();
    println!("{}", "  ═══════════════════════════════════════════════════════════════".cyan());
    println!("  {} CU values from transaction.meta.compute_units_consumed", "ℹ".cyan());
    println!("  {} Program : {}", "ℹ".cyan(), result.spl.program_id.dimmed());
    println!("{}", "  ═══════════════════════════════════════════════════════════════".cyan());
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn add_compare_row(table: &mut Table, label: &str, spl_cu: u64, pt_cu: u64) {
    let pct = improvement_pct(spl_cu as f64, pt_cu as f64);
    table.add_row(vec![
        Cell::new(label),
        Cell::new(format!("{} CU", fmt_num(spl_cu)))
            .fg(Color::Red).set_alignment(CellAlignment::Right),
        Cell::new(format!("{} CU", fmt_num(pt_cu)))
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

fn cu_cell(cu: u64) -> Cell {
    Cell::new(format!("{} CU", fmt_num(cu)))
        .fg(Color::Yellow).set_alignment(CellAlignment::Right)
}

fn ms_cell(ms: u128) -> Cell {
    Cell::new(ms.to_string()).set_alignment(CellAlignment::Right)
}

fn fee_cell(lamports: u64) -> Cell {
    Cell::new(format!("{:.6}", lamports as f64 / 1e9))
        .set_alignment(CellAlignment::Right)
}

fn print_explorer_links(result: &RunResult) {
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
}

fn print_footer(result: &RunResult) {
    println!();
    println!("{}", "  ═══════════════════════════════════════════════════════════════".cyan());
    println!("  {} CU values from transaction.meta.compute_units_consumed", "ℹ".cyan());
    println!("  {} Program : {}", "ℹ".cyan(), result.program_id.dimmed());
    println!("{}", "  ═══════════════════════════════════════════════════════════════".cyan());
}

pub fn export_json_run(result: &RunResult, path: &str) -> Result<()> {
    let json = serde_json::to_string_pretty(result)?;
    std::fs::write(path, &json)?;
    println!("\n  {} Saved to {}", "✓".green(), path.bold());
    Ok(())
}

pub fn export_json_compare(result: &CompareResult, path: &str) -> Result<()> {
    let json = serde_json::to_string_pretty(result)?;
    std::fs::write(path, &json)?;
    println!("\n  {} Saved to {}", "✓".green(), path.bold());
    Ok(())
}

// keep old name working for main.rs
pub use export_json_run as export_json;

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test] fn fmt_zero()           { assert_eq!(fmt_num(0),         "0"); }
    #[test] fn fmt_below_thousand() { assert_eq!(fmt_num(999),       "999"); }
    #[test] fn fmt_exactly_1000()   { assert_eq!(fmt_num(1_000),     "1,000"); }
    #[test] fn fmt_12450()          { assert_eq!(fmt_num(12_450),    "12,450"); }
    #[test] fn fmt_million()        { assert_eq!(fmt_num(1_000_000), "1,000,000"); }
}