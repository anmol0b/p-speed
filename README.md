<div align="center">

# ⚡ P-Speed

**Benchmark P-Token (SIMD-0266) compute units live on Solana**

[![Crates.io](https://img.shields.io/crates/v/p-speed.svg)](https://crates.io/crates/p-speed)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Built with Rust](https://img.shields.io/badge/built%20with-Rust-orange.svg)](https://www.rust-lang.org/)

```
╭───────────────────────┬──────────────────┬────────────────┬─────────────╮
│ Operation             │ SPL Token (live) │ P-Token (live) │ Improvement │
╞═══════════════════════╪══════════════════╪════════════════╪═════════════╡
│ Mint creation         │        3,219 CU  │        386 CU  │  88.0% less │
│ Mint tokens           │        4,641 CU  │        270 CU  │  94.2% less │
│ Transfer  (avg n=20)  │        6,323 CU  │        255 CU  │  96.0% less │
│ Transfer  (min / max) │  6,323 / 6,323   │  255 / 255 CU  │             │
╰───────────────────────┴──────────────────┴────────────────┴─────────────╯
```

*Both columns are real on-chain measurements. Nothing hardcoded.*
*Every TX signature is verifiable on Solana Explorer.*

</div>

---

## What is P-Token?

[P-Token (SIMD-0266)](https://github.com/solana-program/token/tree/main/pinocchio/program) is a **drop-in replacement** for the SPL Token program, rebuilt from scratch using [Pinocchio](https://github.com/febo/pinocchio).

- ✅ Same program address — `TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA`
- ✅ Same instruction and account layout — byte for byte compatible
- ✅ Zero client-side changes required
- ✅ **96% fewer compute units per transfer**
- ✅ Live on mainnet

---

## Install

```bash
cargo install p-speed
```

**Prerequisites:**
- Rust stable — [rustup.rs](https://rustup.rs)
- Solana CLI — [docs.solana.com](https://docs.solana.com/cli/install-solana-cli-tools)

---

## Commands

### `run` — measure P-Token live

Connects to devnet or mainnet and measures real P-Token compute units.

```bash
# devnet (free)
p-speed run --rpc devnet --transfers 20

# mainnet (costs ~$0.02)
p-speed run --rpc mainnet --transfers 20

# custom RPC
p-speed run --rpc https://your-rpc.com --transfers 20

# save results to JSON
p-speed run --rpc devnet --transfers 20 --output results.json
```

---

### `compare` — both sides live, nothing hardcoded

Runs the benchmark on two sides simultaneously:

- **Local validator** with P-Token feature gate **OFF** → old SPL Token bytecode
- **Devnet or mainnet** with feature gate **ON** → P-Token bytecode

Both columns are real on-chain measurements.

**Step 1 — open a new terminal and start local validator:**

```bash
solana-test-validator \
  --deactivate-feature ptokFjwyJtrwCa9Kgo9xoDS59V4QccBGEaRFnRPnSdP
```

Wait until you see `JSON RPC URL: http://127.0.0.1:8899`. Keep this terminal open.

**Step 2 — fund local wallet and run:**

```bash
solana airdrop 5 --url http://127.0.0.1:8899
p-speed compare --transfers 20
```

**Compare against mainnet instead of devnet:**

```bash
p-speed compare --ptoken-rpc mainnet --transfers 20
```

**If you forget to start the validator**, P-Speed tells you exactly what to do:

```
  ✗ Local validator is not running.

  Start it in a new terminal:
  solana-test-validator \
    --deactivate-feature ptokFjwyJtrwCa9Kgo9xoDS59V4QccBGEaRFnRPnSdP

  Then run p-speed compare again.
```

---

## Why local validator?

P-Token lives at the **same address** as old SPL Token. The Solana runtime swaps the bytecode when the feature gate (`ptokFjwyJtrwCa9Kgo9xoDS59V4QccBGEaRFnRPnSdP`) is active.

On mainnet and devnet the gate is already ON — you can only get P-Token numbers there. The only way to get real old SPL numbers is a local validator with the gate explicitly disabled. `--deactivate-feature` tells the validator to run as if P-Token was never shipped.

```
solana-test-validator (no flags)
  → copies mainnet → feature gate ON → P-Token bytecode → 255 CU

solana-test-validator --deactivate-feature ptok...
  → feature gate OFF → old SPL bytecode → 6,323 CU  ← what we want
```

---

## How CU measurement works

P-Speed reads `transaction.meta.compute_units_consumed` from every confirmed transaction — the actual compute units used by the validator, not the budget limit.

Every signature printed by P-Speed can be verified on [Solana Explorer](https://explorer.solana.com).

---

## Options

| Flag | Default | Description |
|------|---------|-------------|
| `--rpc` | `devnet` | RPC: `devnet` \| `mainnet` \| `local` \| full URL |
| `--transfers` / `-n` | `20` | Number of transfers to benchmark |
| `--keypair` | `~/.config/solana/id.json` | Path to payer keypair |
| `--output` | — | Save results to JSON |
| `--spl-rpc` | `local` | (`compare` only) SPL Token RPC |
| `--ptoken-rpc` | `devnet` | (`compare` only) P-Token RPC |

---

## Running tests

```bash
cargo test
```

All tests run offline — no RPC, no SOL required.

---

## License

MIT — see [LICENSE](LICENSE)

---

<div align="center">

Built by [@anmol0b](https://x.com/anmol0b) · Pure Rust · No Anchor · P-Token live on mainnet

</div>