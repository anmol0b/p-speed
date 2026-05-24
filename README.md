<div align="center">

# ⚡ P-Speed

**Benchmark P-Token (SIMD-0266) compute units live on Solana**

[![Crates.io](https://img.shields.io/crates/v/p-speed.svg)](https://crates.io/crates/p-speed)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Built with Rust](https://img.shields.io/badge/built%20with-Rust-orange.svg)](https://www.rust-lang.org/)

```
╭───────────────────────┬─────────────────┬────────────────┬─────────────╮
│ Operation             │ SPL Token (old) │ P-Token (live) │ Improvement │
╞═══════════════════════╪═════════════════╪════════════════╪═════════════╡
│ Mint creation         │       2,967 CU  │        386 CU  │  87.0% less │
│ Mint tokens           │       2,921 CU  │        270 CU  │  90.8% less │
│ Transfer  (avg n=20)  │       4,645 CU  │        255 CU  │  94.5% less │
│ Transfer  (min / max) │                 │  255 / 255 CU  │             │
│ Total fees (all TXs)  │             —   │  0.000120 SOL  │             │
╰───────────────────────┴─────────────────┴────────────────┴─────────────╯
```

*Real numbers. Real transactions. Verify every signature on Solana Explorer.*

</div>

---

## What is P-Token?

[P-Token (SIMD-0266)](https://github.com/solana-program/token/tree/main/pinocchio/program) is a **drop-in replacement** for the SPL Token program, rebuilt from scratch using [Pinocchio](https://github.com/febo/pinocchio) — a zero-dependency, `no_std` Solana framework.

- ✅ Same program address: `TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA`
- ✅ Same instruction and account layout — byte for byte compatible
- ✅ Zero client-side changes required
- ✅ **94.5% fewer compute units per transfer**
- ✅ Live on mainnet

The Solana runtime transparently swaps the bytecode when the feature gate is active. Your existing code works with no modifications — it just costs far less.

---

## Install

```bash
cargo install p-speed
```

Or build from source:

```bash
git clone https://github.com/anmol0b/p-speed
cd p-speed
cargo build --release
```

**Prerequisites:**
- Rust stable (`rustup update stable`)
- Solana CLI (`sh -c "$(curl -sSfL https://release.solana.com/stable/install)"`)
- A funded wallet at `~/.config/solana/id.json`

---

## Usage

```bash
# Run on mainnet (real P-Token, costs ~$0.02)
p-speed run --transfers 20

# Run on devnet (free)
p-speed run --rpc devnet --transfers 20

# Save results to JSON
p-speed run --rpc devnet --transfers 20 --output results.json

# Use a custom RPC endpoint
p-speed run --rpc https://your-rpc.com --transfers 20

# Use a specific keypair
p-speed run --keypair /path/to/keypair.json --transfers 20
```

### Options

| Flag | Default | Description |
|------|---------|-------------|
| `--rpc` | `devnet` | RPC endpoint: `devnet`, `mainnet`, `local`, or a full URL |
| `--transfers` / `-n` | `20` | Number of token transfers to benchmark |
| `--keypair` | `~/.config/solana/id.json` | Path to payer keypair |
| `--output` | — | Save full results to a JSON file |

---

## Quickstart (devnet)

```bash
# 1. Install
cargo install p-speed

# 2. Set up devnet wallet
solana config set --url devnet
solana airdrop 2

# 3. Run
p-speed run --rpc devnet --transfers 20
```

---

## How CU measurement works

P-Speed reads `transaction.meta.compute_units_consumed` from every confirmed transaction — the **actual** compute units used by the validator, not the budget limit you set.

```rust
let tx = client.get_transaction(&signature, UiTransactionEncoding::Json)?;
let cu = tx.transaction.meta.compute_units_consumed; // real number
```

This is the same value shown on [Solana Explorer](https://explorer.solana.com) under *"Compute units consumed"*. Every signature printed by P-Speed can be independently verified.

---

## Why the numbers matter

| | Old SPL Token | P-Token |
|---|---|---|
| Transfer CU | 4,645 | 255 |
| Mint CU | 2,921 | 270 |
| Reduction | — | **94.5%** |

At scale, this directly translates to:
- **Lower fees** for users
- **More transactions per block** for the network
- **Cheaper DeFi protocols** built on token transfers

A protocol doing 1M transfers/day goes from spending ~4.6B CU/day to ~255M CU/day. Same result, 94.5% less compute.

---

## Understanding the output

```
SPL Token column  — documented baseline from spl.solana.com / SIMD-0266
P-Token column    — live on-chain data from your run, transaction.meta.compute_units_consumed
Explorer links    — every TX signature is printed for independent verification
```

The SPL Token column uses the published baseline numbers from the official SIMD-0266 announcement. The P-Token column is always live — run it yourself and get your own verifiable proof.

---

## Project structure

```
p-speed/
├── src/
│   ├── main.rs          # CLI entry point (clap)
│   ├── config.rs        # Program ID, RPC aliases, keypair loading
│   ├── types.rs         # TxMetrics, RunResult, SplBaseline — pure data
│   ├── cu_parser.rs     # Reads real CU from transaction metadata
│   ├── token_ops.rs     # Solana TX logic: mint, ATA, transfer_checked
│   ├── benchmark.rs     # Orchestration: airdrop check, 4-step run
│   └── reporter.rs      # Colored table output + JSON export
├── Cargo.toml
├── LICENSE
└── README.md
```

---

## Extending P-Speed

P-Speed is intentionally simple and modular. Some ideas for contributors:

- **Option B — local validator comparison**: run against `solana-test-validator --deactivate-feature ptokFjwyJtrwCa9Kgo9xoDS59V4QccBGEaRFnRPnSdP` for a fully live both-sides benchmark
- **Batch instructions**: pack multiple transfers into one TX and measure CU per transfer
- **Token-2022 comparison**: add a third column benchmarking Token Extensions
- **CSV export**: add `--output-csv` for spreadsheet analysis

PRs welcome.

---

## Running tests

```bash
cargo test
```

28 unit tests covering CU math, improvement percentages, fee calculations, program ID validity, and number formatting. All tests run offline with no RPC required.

```
running 28 tests
test result: ok. 28 passed; 0 failed
```

---

## Built with

- [`solana-sdk`](https://crates.io/crates/solana-sdk) + [`solana-client`](https://crates.io/crates/solana-client) — Solana Rust SDK
- [`spl-token`](https://crates.io/crates/spl-token) — SPL Token program client
- [`spl-associated-token-account`](https://crates.io/crates/spl-associated-token-account) — ATA derivation
- [`clap`](https://crates.io/crates/clap) — CLI argument parsing
- [`colored`](https://crates.io/crates/colored) + [`comfy-table`](https://crates.io/crates/comfy-table) — terminal output

---

## License

MIT — see [LICENSE](LICENSE)

---

<div align="center">

Built by [@anmol0b](https://x.com/anmol0b) · Pure Rust · No Anchor · P-Token live on mainnet

*If this helped you, star the repo and share your results.*

</div>