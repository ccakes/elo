# CLAUDE.md

## Project overview

Elo is a notepad-style calculator for natural math expressions, inspired by [Numi](https://numi.app/). It supports arithmetic, unit conversions, percentages, dates, currencies, and more.

## Workspace structure

This is a Rust workspace (2024 edition) with the following crates:

- **elo-core** — parsing, evaluation, units, and formatting logic
- **elo-cli** — command-line interface
- **elo-tauri** — Tauri app for desktop (macOS/Linux/Windows) and iOS (Rust backend + JS/pnpm frontend in `elo-tauri/`; see the iOS section below)
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

When a change adds, removes, or alters user-facing functionality, add a bullet to
the `## [Unreleased]` section of `CHANGELOG.md` describing it (match the existing
style). Pure refactors, test-only changes, and internal tweaks don't need an entry.

## iOS

The same Tauri crate (`crates/elo-tauri`) and frontend (`elo-tauri/`) also target
iOS. The generated Xcode project lives at `crates/elo-tauri/gen/apple/` (commit it;
its own `.gitignore` excludes `build/`, `xcuserdata/`, `Externals/`).

Platform splits to preserve when editing the backend:

- `tray-icon` and `tauri-plugin-global-shortcut` are gated to a
  `cfg(not(any(target_os = "ios", target_os = "android")))` target table in
  `Cargo.toml`; global-shortcut registers inside `.setup()` behind `#[cfg(desktop)]`.
- iOS-only iCloud commands (`icloud_*`) live in `src/lib.rs` with cfg-gated bodies
  and use the `objc2`/`objc2-foundation` deps (also iOS-only target table).
- Capabilities are split: `capabilities/default.json` (desktop, has
  `global-shortcut`) vs `capabilities/mobile.json` (iOS/Android, no global-shortcut).

```sh
# Prereqs (macOS only): rustup targets + CocoaPods (system Ruby is too old;
# brew bundles its own). pod must be on PATH: export PATH="/opt/homebrew/bin:$PATH"
rustup target add aarch64-apple-ios aarch64-apple-ios-sim
brew install cocoapods

# One-time scaffold (regenerable)
cargo tauri ios init

# Live-reload dev loop in the Simulator
cargo tauri ios dev

# Build for the simulator (no code signing needed)
cargo tauri ios build --debug --target aarch64-sim
# → crates/elo-tauri/gen/apple/build/arm64-sim/Elo.app
```

Gotchas:

- **Frontend assets are embedded into the Rust binary at compile time**
  (`generate_context!`). A frontend-only change does **not** re-embed via an
  incremental `ios build` — either use `cargo tauri ios dev` (live reload) or force
  a clean rebuild: `rm -rf ~/Library/Developer/Xcode/DerivedData/elo-tauri-*
  crates/elo-tauri/gen/apple/build`.
- Drive builds through the `cargo tauri ios` CLI, not raw `xcodebuild` (the Rust
  build-script phase expects the dev-server address file the CLI sets up).
- The Simulator runtime must match the Xcode SDK version; install it from Xcode →
  Settings → Components if `ios build` reports the SDK is missing.
- iCloud requires a paid Apple Developer account + device: add the **iCloud Documents**
  capability via Xcode's Signing & Capabilities (so provisioning matches) — don't
  hand-edit the `.entitlements`. The `NSUbiquitousContainers` `Info.plist` block is a
  manual edit Xcode does not manage; bump `CFBundleVersion` when changing it.
