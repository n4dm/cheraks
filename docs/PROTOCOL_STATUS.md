# Protocol Status and Development Gaps

This document separates observed prototype behavior from proposed Cheraks design goals. A feature is not considered implemented merely because it appears in the roadmap.

## Current repository state

| Area | Current status |
|---|---|
| Rust block structure and SHA-256 hashing | Implemented in local prototype |
| Rust simplified proof of work | Implemented |
| Rust local block-link and selected integrity checks | Implemented |
| Rust demonstration reward counter | Implemented; not a production issuance system |
| Go block structure and SHA-256 hashing | Implemented in local prototype |
| Go fixed-difficulty mining for non-genesis blocks | Implemented |
| Go JSON persistence and basic chain validation | Implemented |
| Rust and Go client interoperability | Not implemented |
| Signed transactions and signature verification | Not implemented |
| Wallet/key management | Not implemented |
| Peer-to-peer networking | Not implemented |
| Multi-node synchronization | Not implemented |
| Production fork-choice/consensus | Not implemented |
| Production difficulty adjustment | Not implemented |
| Public testnet | Not launched |
| Mainnet | Not launched |
| Independent security audit | Not completed |

## Supply design

The Rust code declares a proposed maximum of 21,000,000 CHK and uses 100,000,000 atomic units per CHK. Its local reward counter is a demonstration mechanism. The Go code does not implement this issuance model. No production-enforced supply guarantee or verified public circulating supply is claimed.

Before this can become a protocol rule, the project must specify issuance at each block height, reward changes, fees, transaction accounting, genesis allocation (if any), and how every node independently verifies the total supply.

## Suggested order of work

1. **Protocol specification:** Define canonical block/transaction encoding, hash preimages, genesis parameters, consensus and error-handling rules.
2. **Single source of truth:** Decide whether Rust or Go is the reference implementation, or define compatibility tests that both clients must pass.
3. **Transactions and state:** Implement signed transactions, replay protection, balance/UTXO rules, fees, and deterministic state transitions.
4. **Persistent node:** Add versioned storage, crash recovery, and safe database migrations.
5. **Consensus:** Specify fork-choice, difficulty adjustment, timestamp constraints, issuance, and reorganization handling.
6. **Networking:** Implement peer discovery, message validation, synchronization, resource limits, and denial-of-service protections.
7. **Testing:** Add property-based tests, fuzzing, adversarial multi-node simulations, and cross-implementation test vectors.
8. **Testnet readiness:** Publish reproducible node instructions, genesis configuration, observability, incident handling, and an independent review plan.
9. **Mainnet decision:** Consider only after sustained testnet operation and independent security review.

These are recommendations, not promises or announced release dates.
