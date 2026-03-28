# Elo Architecture

## Overview

Elo is a Numi-compatible calculator application built in Rust with a Tauri v2 shell. The architecture follows a layered engine design with clear separation between core logic and UI. All parsing, evaluation, and formatting lives in Rust; the Tauri frontend is a thin display layer.

## Project Layout

```
/crates
  /elo-core       Parser, AST, evaluator, formatter, locale, session engine
  /elo-cli        Developer CLI for the Rust engine
  /elo-compat     numi-cli oracle compatibility + differential fuzz harness
  /elo-data       Built-in unit, currency, timezone, and function metadata
  /elo-tauri      Tauri v2 backend — commands bridging elo-core to the frontend

/elo-tauri         Frontend (TypeScript + Vite)
  /src/main.ts     Editor, live eval, file ops, keyboard shortcuts
  /src/styles.css  Catppuccin Mocha dark theme

/tests
  /goldens         107-entry tagged fixture corpus
```

## Engine Layers

### Lexer (`elo-core/src/lexer.rs`)
Tokenizes input into numbers (decimal, hex, binary, octal, scientific, comma-separated), identifiers, operators (`+`, `-`, `*`, `/`, `^`, `%`, `&`, `|`, `<<`, `>>`), delimiters, labels (`:`) , comments (`//`), headers (`#`), quoted text, and currency symbols (`$`, `€`, `£`, etc.).

### AST (`elo-core/src/ast.rs`)
Expression tree nodes: `Number`, `HexLiteral`, `BinLiteral`, `OctLiteral`, `SciLiteral`, `Percentage`, `Ident`, `BinaryOp`, `UnaryOp`, `FuncCall`, `Conversion`, `PercentOf/On/Off/OfWhatIs`, `Prev/Sum/Avg`, `Scaled`, `WithUnit`, `UnitSequence`, `ImplicitMul`, `Paren`, `Today/Tomorrow/Yesterday`.

Line-level: `Empty`, `Comment`, `Header`, `Expression` (with optional label), `Assignment`.

### Parser (`elo-core/src/parser.rs`)
Recursive descent with precedence climbing. Layers (lowest to highest):
1. Conversion (`in`/`into`/`as`/`to`)
2. Percentage ops (`X% of/on/off Y`)
3. Bitwise OR → XOR → AND → Shift
4. Additive (`+`, `-`, word ops, percent-on/off sugar)
5. Multiplicative (`*`, `/`, word ops)
6. Modulo
7. Power (`^`)
8. Unary (`-`, `+`)
9. Postfix (unit attachment, scales, `%`, unit sequences, implicit multiplication)
10. Primary (numbers, identifiers, functions, parens, session/date tokens, currency symbols)

Unit sequences (`1 meter 20 cm`) are detected in postfix when a `WithUnit` is followed by `Number Unit` in the same dimension.

### Evaluator (`elo-core/src/eval.rs`)
Walks the AST with an `EvalContext` (variables, block results, previous result). Produces typed `Value` variants:
- `Number(f64)`, `WithUnit(f64, String)`, `Currency(f64, String)`, `Percent(f64)`
- `DateTime(String)`, `Duration(f64, String)`, `Boolean(bool)`, `Error(String)`, `Empty`

Key capabilities:
- **Arithmetic**: standard + word operators, operator precedence
- **Functions**: sqrt, cbrt, abs, log, ln, fact, round/ceil/floor, sin/cos/tan + arc/hyp variants, fromunix
- **Units**: conversion via elo-data registry, cross-unit arithmetic (auto-converts to right operand's unit)
- **Date/time**: `now`/`time` via chrono, `today`/`tomorrow`/`yesterday`, date arithmetic (`now + 3 days`), timezone conversion (`time in UTC`, `time in America/New_York`) via chrono-tz
- **Percentages**: of, on, off, inverse, `expr ± X%` sugar
- **Session tokens**: prev, sum/total, avg/average with block separation on empty lines

### Formatter (`elo-core/src/formatter.rs`)
Renders `Value` to display strings. Handles special `__hex__`/`__binary__`/`__octal__`/`__sci__` markers for format-conversion expressions (`10 as hex` → `0xa`).

### Locale (`elo-core/src/locale.rs`)
`Locale` struct with decimal separator and thousands grouping. Presets for `en` (1,234.56), `de` (1.234,56), `fr` (1 234,56), `c` (no grouping). System detection via `LANG`/`LC_NUMERIC` env vars.

### Session (`elo-core/src/session.rs`)
Document-level evaluation engine. Wraps `EvalContext`, manages multi-line state. API: `eval_line(input) → LineResult { input, value, display }`, `eval_document(text) → Vec<LineResult>`.

## Data Layer (`elo-data`)

### Units (`units.rs`)
40+ units across 10 dimensions (length, area, volume, mass, time, temperature, angle, data, CSS, speed). Each `UnitDef` has a conversion factor to its dimension's base unit (with offset for temperature). `find_unit()` does case-sensitive then case-insensitive lookup. `convert()` routes through the base unit.

### Currencies (`currencies.rs`)
30 currencies with ISO 4217 codes, common names, and symbols. `find_currency()` matches by code, symbol, or name. Rate conversion is not yet implemented (requires external provider).

### Timezones (`timezones.rs`)
90+ aliases mapping abbreviations (EST, UTC, CET, JST) and city names (Tokyo, London, New York) to IANA identifiers. `find_timezone()` returns the IANA string; the evaluator also falls back to direct chrono-tz parsing for full IANA names like `America/New_York`.

### Functions (`functions.rs`)
Registry of 20+ built-in function names. Used by the parser to distinguish `sqrt(16)` from a variable named `sqrt`.

## Tauri Desktop App

### Backend (`crates/elo-tauri/src/lib.rs`)
Tauri v2 app with managed `AppState` holding a `Mutex<Session>`. Commands:
- `evaluate_document(text)` — resets session, evaluates all lines, returns `Vec<LineResult>`
- `evaluate_line(line)` — evaluates one line in existing session
- `reset_session()` — clears state

Plugins: `tauri-plugin-opener`, `tauri-plugin-dialog`, `tauri-plugin-fs`, `tauri-plugin-global-shortcut`.

### Frontend (`elo-tauri/src/main.ts`)
Vanilla TypeScript + Vite. Layout: toolbar, editor textarea (left), result gutter (right), status bar.

- **Live evaluation**: 30ms debounced — every keystroke triggers `evaluate_document` via Tauri IPC
- **Result gutter**: right-aligned panel with scroll sync to editor. Color-coded: green (values), red (errors), blue (headers), purple (datetimes), dim (comments)
- **File operations**: New/Open/Save/Export via `@tauri-apps/plugin-dialog` + `@tauri-apps/plugin-fs`
- **Copy**: click any result to copy; Cmd+Shift+C copies all results
- **Keyboard shortcuts**: Cmd+N (new), Cmd+O (open), Cmd+S (save), Cmd+Shift+C (copy all)

### Theme
Catppuccin Mocha dark theme with monospace font stack (JetBrains Mono → SF Mono → Fira Code → Cascadia Code → Menlo → Consolas).

## Testing Strategy

| Layer | Count | Tool |
|---|---|---|
| Unit tests | 92 | `cargo test` — lexer, parser, evaluator, formatter, session, locale |
| Golden corpus | 107 | Tab-delimited fixture file, integration test runner with fuzzy numeric matching |
| Property tests | 12 | proptest — no-panic, determinism, arithmetic invariants, roundtrip |
| Oracle tests | 14 | elo-compat — direct numi-cli comparison by feature group |
| Differential fuzz | 8 | Grammar-fragment generation (500+ expressions) compared against numi-cli |
| **Total** | **233** | |

Oracle and fuzz tests are `#[ignore]`'d by default (require numi-cli). Run with `cargo test -- --ignored`.

## Design Decisions

1. **Pure Rust core**: No parsing logic in the UI layer. The engine is independently testable via elo-cli and unit tests.
2. **Semantic values**: Rich typed `Value` enum rather than string-in/string-out.
3. **Session model**: Multi-line documents are first-class, with block-scoped aggregation.
4. **Oracle-driven development**: Every feature is validated against numi-cli. Mismatches are either fixed or documented.
5. **Tolerance-based comparison**: Compat tests use numeric tolerance and formatting normalization rather than exact string matching.
6. **Thin UI**: Tauri frontend does zero parsing — it sends text, receives results, renders them.
7. **chrono/chrono-tz**: Pure Rust date/time and timezone support, no shelling out.
