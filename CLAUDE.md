# app-squads — Ledger Clear Signing App for Squads v4

## Language & Toolchain
- Rust 2021 edition, nightly-2025-12-05
- `#![no_std]` + `#![no_main]` — runs on Ledger Secure Element (ARM Cortex-M)
- `ledger_device_sdk` v1.35.0 with `io_new` and `nano_nbgl` features
- Build targets: nanosplus, nanox, stax, flex, apex_p

## Architecture
- **APDU Layer** (src/handlers/) — chunked reception, state machine, status words
- **Parser Layer** (src/parser/) — zero-copy Reader over raw transaction buffer, no Vec/alloc
- **Display Layer** (src/display/) — NBGL review screens, base58 formatting, amount formatting
- **Signing Layer** (src/crypto/) — Ed25519 derivation + signing with zeroize

## Constraints
- ~28KB usable RAM on Nano S+/X, ~40KB on Stax/Flex
- Max transaction buffer: 1300 bytes (Solana max tx = 1232 bytes)
- Zero heap allocation during parsing — all data read from static buffer via offsets
- Stack budget: keep nesting under 2KB for deepest vault transaction parsing

## Security
- All on-chain data sanitized before display (ASCII-only, 64-byte cap)
- Solana message structural validation (all indices in bounds) before any display
- Blind signing disabled by default
- Key material zeroed via `zeroize` immediately after signing
- Squads v4 program ID hardcoded as const [u8; 32]
- BIP32 path validation: 3-4 hardened components, must start with 44'/501'

## Dependencies (minimal, all no_std)
- `arrayvec` 0.7 — fixed-capacity strings for display formatting
- `bs58` 0.5 — base58 encoding into caller-provided buffer
- `numtoa` 0.2 — integer-to-string without alloc
- `zeroize` 1.x — secure memory clearing

## Testing
- Speculos emulator + Ragger (Python) for functional tests
- Docker: `ghcr.io/ledgerhq/ledger-app-dev-tools:latest`
- `cargo ledger build nanosplus` to build for Nano S+
