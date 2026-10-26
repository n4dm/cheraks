# Cheraks (CHK) — Phase 1 prototype

Cheraks is an educational, Bitcoin-inspired proof-of-work blockchain prototype. It is **not a production cryptocurrency**, has no wallet for real funds, and is not deployed to a public network. All demonstration coins are worthless test units.

## Current features

- Genesis block with a Cheraks-specific message
- SHA-256 block hashing using the `sha2` crate
- A simple proof-of-work puzzle
- Block linking and basic chain validation
- Prototype issuance accounting with a 21 million CHK intended cap
- Automated tests for mining, linkage, tampering, and supply constants

## Important limitations

This is a first learning milestone, not a complete Bitcoin-like protocol. It does **not** yet implement signed transactions, a UTXO set, real coinbase transactions, difficulty adjustment, block rewards/halving, peer-to-peer networking, chain synchronization, persistent storage, a wallet, or an explorer. Its issuance accounting is a simplified demonstration and must not be treated as consensus-critical production code. The difficulty setting is fixed and intentionally low for local testing.

Do not use this code for real money or describe it as secure or launched.

## Requirements

Install Rust using the official instructions: https://www.rust-lang.org/tools/install

On Windows, install the Rust toolchain and follow any prompts to install the Microsoft C++ Build Tools if requested.

## Run it

Open PowerShell in this folder and run:

```powershell
cargo run
```

The first run downloads the `sha2` dependency from crates.io, so internet access is needed.

## Run tests

```powershell
cargo test
```

Expected: 4 tests pass.

## Suggested next milestones

1. Add transaction types and digital signatures using established libraries.
2. Implement a proper UTXO model and consensus-validated coinbase transactions.
3. Add persistent chain storage and stronger validation.
4. Add peer-to-peer networking and sync between independent nodes.
5. Create a private multi-node testnet with worthless test coins.
6. Obtain independent security review before considering any public mainnet.

## License

MIT. This prototype is provided for education and experimentation.
