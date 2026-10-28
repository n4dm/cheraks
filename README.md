# Cheraks (CHK)

**An experimental proof-of-work blockchain project under active development.**

Cheraks explores the design and implementation of a peer-to-peer digital currency. The repository currently contains small Rust and Go blockchain prototypes. It is not a production cryptocurrency and is not connected to a public peer-to-peer network.

> **Development status:** Experimental prototype. Do not use it to store, send, or receive real funds. CHK has no verified public circulating supply or official market price.

## What exists today

- **Rust prototype:** A local in-memory chain with SHA-256 block hashing, simplified proof of work, block-link validation, and a demonstration reward counter.
- **Go prototype:** A local JSON-backed chain that creates and mines demonstration blocks, validates saved block links, and refuses to overwrite a saved chain when loading or validation fails.
- **Validation tests:** Rust unit tests covering genesis proof, block linkage, tampering, reward-data checks, difficulty consistency, and supply constants.
- **Development checks:** The Rust and Go code can be checked locally with the commands below.

These are separate experiments, not two compatible implementations of one finalized consensus protocol. The current prototypes do not implement a complete transaction model, cryptographic wallet/signature system, peer-to-peer networking, chain synchronization, or production consensus.

## Quick start

### Requirements

- Rust toolchain with Cargo
- Go version compatible with the version declared in `go.mod`

### Run the Rust demonstration

```bash
cargo test --all-targets
cargo run
```

The Rust program mines a demonstration chain locally and prints whether its in-memory validation succeeds.

### Run the Go demonstration

```bash
go test ./...
go vet ./...
go run ./cmd/cheraks
```

The Go program reads or creates `cheraks-chain.json`, mines a local demonstration block, validates the chain, and saves it. **Back up this file before running the Go program if you need to preserve its contents.** The program can append a block when the saved chain is valid.

## Project design targets

The following are design intentions, not guarantees of a deployed network:

- **Name:** Cheraks
- **Ticker:** CHK (planned)
- **Consensus direction:** Proof of work
- **Proposed maximum supply:** 21,000,000 CHK
- **Public testnet:** Not launched
- **Mainnet:** Not launched
- **Production wallet:** Not available

The Rust prototype defines supply-related constants and a simplified demonstration reward counter. Those mechanisms are not a finalized, production-enforced monetary policy. The Go prototype is a separate local chain experiment and does not implement CHK issuance.

## Current limitations

Cheraks is not ready for public use. Before a testnet or mainnet can be considered, the project needs a specified and consistently implemented protocol, real transaction validation and signatures, a peer-to-peer transport and synchronization design, persistent storage rules, fork-choice and difficulty-adjustment rules, comprehensive adversarial testing, and independent security review.

Passing local tests does not prove that a blockchain is secure. The repository has not been independently audited, and no production-security guarantee is made.

## Documentation

- [Architecture and current components](docs/ARCHITECTURE.md)
- [Protocol status and development gaps](docs/PROTOCOL_STATUS.md)
- [Contributing](CONTRIBUTING.md)
- [Security reporting](SECURITY.md)

## Development checks

The GitHub Actions workflow runs Rust tests/build checks and Go tests/vet on pushes and pull requests. A green CI run means those checks passed; it is not a security audit or proof of protocol correctness.

## License

No license file is currently included in this repository. The `license = "MIT"` package metadata previously present in `Cargo.toml` was not a substitute for an actual license grant and has been removed. Until a license is selected and added, do not assume this code is licensed for reuse, modification, or redistribution.

---

Cheraks is an experimental software project. Project descriptions distinguish implemented prototype behavior from future design goals; nothing here should be interpreted as a claim that a live public cryptocurrency network exists.
