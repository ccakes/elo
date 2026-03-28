# Elo

Elo is a notepad-style calculator for natural math expressions. Type plain-English arithmetic, unit conversions, percentages, dates, and more — results appear live as you type.

Uses [Tauri](https://tauri.app/) for the cross-platform desktop GUI.

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
