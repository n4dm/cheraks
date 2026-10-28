# Cheraks (CHK)

**A public blockchain project built for open participation.**

Cheraks is a peer-to-peer digital currency project. This repository is the public source of the project, with Rust and Go implementations that are being developed into the complete Cheraks network.

The repository foundation comes first: clear documentation, repeatable development checks, security-reporting guidance, and a structure that can support the implementation work ahead. Core network capabilities will be documented as they are implemented and verified.

> **Important:** The current code in this repository contains local blockchain implementations. It does not yet demonstrate a complete public peer-to-peer network, production wallet, or transferable CHK transaction system. Do not use the current code to store or transfer real funds.

## Repository contents

- **Rust implementation:** In-memory blockchain, SHA-256 block hashing, simplified proof of work, block-link validation, and a demonstration reward counter.
- **Go implementation:** Local JSON-backed chain, fixed-difficulty block mining, basic chain validation, and file persistence.
- **Tests:** Rust unit tests for selected block-integrity and reward-data checks.
- **CI:** GitHub Actions checks for Rust formatting, tests and release builds, plus Go tests and static analysis.
- **Documentation:** Architecture, current protocol status, contribution guidance, and security reporting.

The Rust and Go implementations are currently separate code paths; they do not yet implement a shared finalized protocol or interoperable clients.

## Quick start

### Requirements

- Rust toolchain with Cargo
- Go version compatible with the version declared in `go.mod`

### Rust

```bash
cargo fmt --check
cargo test --all-targets
cargo build --release
cargo run
```

The Rust program mines a demonstration chain in memory and prints the result of its local validation.

### Go

```bash
go test ./...
go vet ./...
go run ./cmd/cheraks
```

The Go program reads or creates `cheraks-chain.json`, mines a local demonstration block, validates the chain, and saves it. **Back up this file before running the program if you need to preserve its contents.** A successful run can append a block to the saved chain.

## Project parameters

These are current design targets, not claims about a deployed network:

| Property | Current position |
|---|---|
| Project name | Cheraks |
| Ticker | CHK (planned) |
| Consensus direction | Proof of work |
| Proposed maximum supply | 21,000,000 CHK |
| Public testnet | Not launched |
| Mainnet | Not launched |
| Production wallet | Not available |

The Rust code defines supply-related constants and a demonstration reward counter. This is not a finalized, production-enforced monetary policy. The Go implementation does not implement the same issuance model. No verified public circulating supply or official market price is claimed.

## Implementation work ahead

To deliver a complete public network, the project needs a finalized protocol and consistent implementation of transaction validation, cryptographic signatures, wallet/key handling, peer-to-peer networking, chain synchronization, persistent node storage, consensus and fork-choice rules, issuance and fee rules, and adversarial testing. Production deployment should follow thorough independent review.

Passing local tests is useful evidence about the tested behavior, but it does not establish that the whole system is secure or production-ready.

## Documentation

- [Architecture and current components](docs/ARCHITECTURE.md)
- [Protocol status and development gaps](docs/PROTOCOL_STATUS.md)
- [Contributing](CONTRIBUTING.md)
- [Security reporting](SECURITY.md)

## License

No license file is currently included. The package metadata has been updated so it does not claim a license that has not been selected and granted. Until a license is chosen and added, do not assume that public visibility grants permission to reuse, modify, or redistribute the code.

---

Cheraks is publicly documented here, and the implementation will continue to be built and verified in this repository. Documentation distinguishes current code from planned functionality so contributors can see what exists and what remains to be implemented.
