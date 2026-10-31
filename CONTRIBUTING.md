# Contributing to Cheraks

Thanks for your interest in reviewing or improving Cheraks.

## Before contributing

Cheraks is an experimental prototype. Read the [architecture overview](docs/ARCHITECTURE.md) and [protocol status](docs/PROTOCOL_STATUS.md) before changing consensus-related behavior.

## Development checks

Run the checks relevant to your change:

```bash
cargo fmt --check
cargo test --all-targets
cargo build --release
go test ./...
go vet ./...
```

The Go demonstration writes to `cheraks-chain.json`. Back up local chain data before running it if you need to preserve the file.

## Pull requests

- Explain the problem and the intended behavior change.
- Keep claims in documentation aligned with implemented behavior.
- Include tests for validation rules and malformed or adversarial inputs.
- Avoid unrelated changes to stored chain data.
- Do not claim production readiness or security guarantees based only on passing tests.
- For consensus changes, describe compatibility and migration implications.

## License status

No project license has been added yet. Do not assume that public visibility grants permission to reuse, modify, or redistribute the project. A license decision is pending.
