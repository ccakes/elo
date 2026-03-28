# CLAUDE.md

## Project overview

Elo is a notepad-style calculator for natural math expressions, inspired by [Numi](https://numi.app/). It supports arithmetic, unit conversions, percentages, dates, currencies, and more.

## Workspace structure

This is a Rust workspace (2024 edition) with the following crates:

- **elo-core** — parsing, evaluation, units, and formatting logic
- **elo-cli** — command-line interface
- **elo-tauri** — Tauri desktop app (Rust backend + JS/pnpm frontend in `elo-tauri/`)
- **elo-compat** — compatibility/fuzz tests against Numi
- **elo-data** — data definitions (currencies, etc.)

## Development

```sh
# Run the CLI
cargo run -p elo-cli -- "2 + 3 * 4"

# Run the Tauri dev server
cd elo-tauri && pnpm install && cd .. && cargo tauri dev

# Run tests
cargo test --workspace

# Build all targets
make build
```

## Mandatory checks

After **any** code change, run these commands before committing. CI will reject the PR if either fails.

```sh
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all -- --check
```

To auto-fix formatting: `cargo fmt --all`.
