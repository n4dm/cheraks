# Architecture and Current Components

## Scope

The repository currently contains two small, independent blockchain experiments: a Rust in-memory prototype and a Go JSON-backed local-chain prototype. They do not share a finalized block format or consensus implementation and should not be treated as interoperable clients.

## Rust prototype

Files: `src/lib.rs`, `src/main.rs`

- Uses SHA-256 to calculate block hashes from a delimited text payload.
- Mines by incrementing a nonce until the hash begins with a configured number of hexadecimal zeroes.
- Links each demonstration block to the previous block hash.
- Validates genesis fields, block heights, previous hashes, proof of work, difficulty consistency, and a reward marker in block data.
- Tracks a demonstration issued-unit counter and constants for a proposed 21-million-CHK cap.

### Important limitations

- Block fields and chain state are publicly mutable in the current library API.
- The reward is represented as text in block data, not as a validated coinbase transaction or spendable output.
- There are no signed transactions, accounts, UTXO/state-transition engine, networking, chain synchronization, persistent Rust storage, or production fork-choice rules.
- The current mining loop uses a saturating nonce increment; nonce exhaustion is not handled as a recoverable error.
- The reward/data checks are educational validation logic, not a complete monetary-policy implementation.

## Go prototype

File: `cmd/cheraks/main.go`

- Uses SHA-256 over concatenated block fields.
- Mines blocks against a fixed four-leading-zero hexadecimal prefix.
- Stores a local chain as JSON in `cheraks-chain.json`.
- Validates the genesis hash, sequential indices, previous-hash links, recalculated hashes, and proof-of-work prefix for non-genesis blocks.
- Refuses to overwrite an existing chain file when loading fails for reasons other than file-not-found.

### Important limitations

- Genesis creation does not mine the genesis block.
- Genesis validation does not enforce proof of work.
- The hash input concatenates fields without explicit separators or a canonical binary encoding.
- There is no transaction model, wallet, signature verification, networking, difficulty adjustment, chain selection, or production issuance system.
- Running the program appends a local demonstration block and changes the JSON file.

## Data safety

Before running the Go executable, make a backup of `cheraks-chain.json` if its contents matter. The Go program is intended for local experimentation, not as a production node or wallet.

## Future architecture work

A production-oriented design would first need a versioned protocol specification and canonical serialization. Separate modules should then define transaction validation, signatures, state transitions, block validation, persistent storage, peer-to-peer messages, synchronization, and chain selection. Each stage should have tests for malformed input, boundary conditions, and adversarial behavior before being integrated.
