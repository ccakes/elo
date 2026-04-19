# Elo

Elo is a notepad-style calculator for natural math expressions. Type plain-English arithmetic, unit conversions, percentages, dates, and more — results appear live as you type. This is a re-implementation of [Numi](https://numi.app/) - for a long time I really wanted Numi on Linux but and made do with the CLI app but with the advent of Claude, I figured I could probably vibe code an implementation. Turns out yeah, you can!

> [!NOTE]
> **If you like this, please go buy Numi!**
>
> Support the original artist behind this, it's a great app and if you're on Mac on Windows, you don't need this. Numi is better and it's shareware, so if you're financially constrained then you can still use the original!

## Features

- **Arithmetic and functions** — standard math, operator precedence, `sqrt`, `log`, `sin`/`cos`/`tan`, factorials, and more
- **Word operators** — `plus`, `minus`, `times`, `divide`, `multiplied by`, etc.
- **Number formats** — decimal, hex (`0xFF`), binary (`0b1010`), octal (`0o17`), scientific (`1.5e3`), comma-separated (`1,000,000`)
- **Units** — length, area, volume, mass, time, temperature, angle, data, CSS, speed with natural conversion syntax (`5 feet in meters`)
- **Unit sequences** — `5 feet 3 inches`, `1 meter 20 cm`
- **Percentages** — `15% of 200`, `100 + 10%`, `25% off 80`, `X% of what is Y`
- **Scales** — `k`/`thousand`, `M`/`million`, `billion`
- **Dates and timezones** — `now`, `today + 3 days`, `time in Asia/Tokyo`, `fromunix()`
- **Currencies** — `$100`, `€50`, `100 USD`
- **Variables and sessions** — assign with `=`, reference `prev`, `sum`/`total`, `avg`/`average`
- **Labels and comments** — `# headers`, `// comments`, `Label: expr`
- **Format conversions** — `as hex`, `as binary`, `as octal`, `in scientific`
- **Locale support** — locale-sensitive number formatting (`en`, `de`, `fr`)
- **File operations** — open, save, and export notes as `.elo`, `.txt`, or `.md`

## Install

### macOS (Homebrew)

```sh
brew install --cask ccakes/elo/elo
```

This taps `ccakes/homebrew-elo` and installs the latest universal `Elo.app` into `/Applications`.

> The app isn't yet notarized, so on first launch macOS will complain. Right-click `Elo.app` → **Open** to bypass Gatekeeper once.

### Tap setup (one-time, maintainer only)

1. Create a public repo `ccakes/homebrew-elo` with a `Casks/` directory.
2. Copy `packaging/homebrew/elo.rb` from this repo into `Casks/elo.rb` in the tap.
3. In this repo's settings, add a secret `HOMEBREW_TAP_TOKEN` — a fine-grained PAT with `contents: write` on `ccakes/homebrew-elo`. The release workflow uses it to bump the cask automatically on every `v*` tag.

## Prerequisites

- [Rust](https://rustup.rs/) (2024 edition)
- [Node.js](https://nodejs.org/) and [pnpm](https://pnpm.io/)
- [Tauri CLI](https://tauri.app/start/): `cargo install tauri-cli`

## Building

Build all targets:

```sh
make build
```

Or build individually:

```sh
make build-cli      # CLI binary only
make build-macos    # macOS .app bundle
make build-linux    # Linux .deb and .appimage bundles
```

Outputs go to the `build/` directory.

## Development

```sh
cd elo-tauri && pnpm install
cd .. && cargo tauri dev
```

### CLI

```sh
cargo run -p elo-cli -- "2 + 3 * 4"
```

## Testing

```sh
make test
```

## License

MIT
