# Elo Implementation Plan

## Milestone 1: Engine Skeleton

- [x] Cargo workspace setup with `elo-core`, `elo-cli`, `elo-compat`, `elo-data` crates
- [x] Lexer scaffolding — tokenizes numbers, identifiers, operators, delimiters, labels, comments, quoted text, currency symbols, note-structure tokens
- [x] AST definition — expression tree with arithmetic, functions, conversions, percentages, variables, session tokens, scales, unit attachment, implicit multiplication
- [x] Parser scaffolding — recursive descent, operator precedence, word operators, conversion syntax, percentage operations, label/assignment detection
- [x] Evaluator scaffolding — walks AST, produces typed `Value` results (Number, WithUnit, Currency, Percent, DateTime, Duration, Boolean, Error, Empty)
- [x] Formatter scaffolding — hex/binary/octal/scientific rendering, display string generation
- [x] Session/document engine — multi-line state, variable scope, prev tracking, block-separated aggregations
- [x] Local developer CLI (`elo-cli`) — single expression and stdin/pipe modes
- [x] Deterministic test harness — 70 unit tests across lexer, parser, evaluator, formatter, session

## Milestone 2: Core Calculator Compatibility

- [x] Arithmetic — `+`, `-`, `*`, `/`, `^`, `mod`
- [x] Word operators — `plus`, `and`, `with`, `minus`, `subtract`, `without`, `times`, `mul`, `multiplied by`, `divide`, `divide by`
- [x] Operator precedence — standard math precedence, parentheses
- [x] Number formats — decimal, hex (`0xFF`), binary (`0b1010`), octal (`0o17`), scientific (`1.5e3`), comma-separated (`1,000,000`)
- [x] Functions — `sqrt`, `cbrt`, `root`, `abs`, `log`, `ln`, `fact`/`factorial`, `round`, `ceil`, `floor`, `sin`, `cos`, `tan`, `arcsin`/`asin`, `arccos`/`acos`, `arctan`/`atan`, `sinh`, `cosh`, `tanh`, `fromunix`
- [x] Variables — assignment via `=`, variable reuse
- [x] Constants — `pi`, `e`
- [x] Formatting/comments/labels — `#` headers, `//` comments, `Label: expr`
- [x] Session tokens — `prev`, `sum`/`total`, `avg`/`average`
- [x] Block separation — empty lines reset aggregation scope
- [x] Bitwise operators — `&`, `|`, `xor`, `<<`, `>>`
- [x] Format conversions — `as hex`, `as binary`, `as octal`, `in sci`/`scientific`
- [x] Oracle compatibility harness (`elo-compat`) — 9 numi-cli comparison tests, semantic matching with numeric tolerance
- [x] Compatibility matrix documented (`docs/compatibility-matrix.md`)

## Milestone 3: Units and Percentages

- [x] Unit families — length, area, volume, mass, time, temperature, angle, data, CSS, speed
- [x] Unit conversion registry — static `UnitDef` table with conversion factors and offsets
- [x] Natural conversion syntax — `in`, `into`, `as`, `to`
- [x] Temperature conversions — Celsius, Fahrenheit, Kelvin with offset handling
- [x] Percentages — `X%` as value, `X% of Y`, `X% on Y`, `X% off Y`, `expr + X%`, `expr - X%`
- [x] Inverse percentages — `X% of what is Y`
- [x] Scales — `k`/`thousand`, `M`/`million`, `billion` (case-sensitive)
- [x] CSS units — `px`, `pt`, `em`, `rem`, `pc`, `vw`, `vh`
- [ ] SI prefix support — generalized `kilo-`, `mega-`, `giga-`, etc. beyond hard-coded units
- [x] Unit sequences — compound expressions like `1 meter 20 cm`, `5 feet 3 inches` (converts to last unit, sums)
- [x] Unit arithmetic — cross-dimension addition/subtraction with auto-conversion to right operand's unit (numi-compatible)

## Milestone 4: Dates/Timezones/Currency

- [x] `now`/`time` — returns current date-time string via chrono (pure Rust)
- [x] Date keywords — `today`, `tomorrow`, `yesterday`
- [x] Date/time arithmetic — `now + 3 days`, `today + 1 week`, `tomorrow + 2 days`, `now - 2 weeks`
- [x] Timezone queries — `time in UTC`, `time in America/New_York`, `time in Asia/Tokyo`, `time in Europe/London`
- [x] IANA timezone support — full chrono-tz database, plus 90+ city/abbreviation aliases
- [x] `fromunix()` — pure Rust via chrono `Utc.timestamp_opt()`
- [x] Currency parsing — `$100`, `€50`, `100 USD` (parser recognizes, evaluator attaches codes)
- [ ] Currency conversions — `100 USD in EUR` (requires rate provider) *excluded per user request*
- [ ] Rate provider abstraction — abstract interface, mock for tests, live provider for app *excluded per user request*
- [x] Locale module — `Locale` struct with `en`, `de`, `fr`, `c` presets, system detection, `from_identifier()`
- [x] Locale-sensitive number formatting — decimal separator, thousands grouping
- [x] Locale override in CLI — `--locale` / `-l` flag

## Milestone 5: Desktop App

- [x] Tauri v2 project setup — `crates/elo-tauri` Rust backend + `elo-tauri/` frontend
- [x] Web frontend scaffolding — vanilla TypeScript + Vite, Catppuccin Mocha theme
- [x] Single-note editor — multi-line textarea with monospace font
- [x] Live evaluation — debounced (30ms) per-keystroke re-evaluation via `evaluate_document` command
- [x] Result gutter/panel — right-aligned results column with scroll sync
- [x] Result styling — green for values, red for errors, blue for headers, purple for datetimes, dim for comments
- [x] Error display — error results shown in red in result gutter
- [x] Tauri commands — `evaluate_document`, `evaluate_line`, `reset_session` bridging elo-core Session
- [x] File operations — New (Cmd+N), Open (Cmd+O), Save (Cmd+S) via `tauri-plugin-dialog` + `tauri-plugin-fs`
- [x] Export — plain text with right-aligned results, via save dialog
- [x] Import — Open dialog with .elo/.txt/.md filters
- [x] Copy result — click any result line to copy; Cmd+Shift+C copies all results
- [x] Keyboard shortcuts — Cmd+N/O/S, Cmd+Shift+C
- [x] Toolbar — New, Open, Save, Export, Copy Results buttons
- [x] Status bar — line count and result count
- [x] Toast notifications — copy/save confirmation
- [x] Global shortcut plugin — `tauri-plugin-global-shortcut` wired in backend
- [ ] System tray — tray icon with menu (requires icon asset)
- [ ] Preferences UI — locale, theme, default currency
- [ ] Window state persistence — remember size/position
- [ ] Multiple notes — tab or sidebar support
- [ ] Recent notes — MRU list
- [ ] Compact quick-calc popover (nice to have)

## Milestone 6: Compatibility Hardening

- [x] Oracle test harness — `elo-compat` crate comparing against numi-cli, 14 oracle test groups
- [x] Semantic comparison — numeric tolerance, suffix/unit matching, hex/bin/oct normalization, datetime proximity
- [x] Large fixture corpus — 107 tagged test cases in `tests/goldens/expressions.txt`
- [x] Golden test runner — integration test loading corpus, normalizing results, fuzzy matching
- [x] Differential fuzzing — 8 fuzz targets generating 500+ expressions, comparing against numi-cli
- [x] Fuzz targets — operator precedence (864 combos), unary signs, scale suffixes, nested functions, unit conversions, labels/comments, percentage combos (245 combos), unit sequences
- [x] Property tests — 12 proptest-based tests: no-panic on arbitrary input, deterministic evaluation, number roundtrip, integer arithmetic exactness, double negation, division inverse
- [x] Targeted fuzz — nested parens (50 depths), long expressions (200 terms), tricky inputs (20 edge cases)
- [x] Mismatch triage — all divergences either fixed or documented in `docs/compatibility-matrix.md`
- [x] Regression fixes — `fact()` matched to numi identity behavior, `sqrt(-1)` returns error, unit display names fixed (`m`, `L`, `pt`), year conversion factor corrected, error recovery improved
- [ ] Snapshot tests — editor/render output for representative notes (deferred to M5/Tauri)
- [ ] CI pipeline — Linux and macOS, clippy, rustfmt, test suite

## Cross-Cutting Concerns (ongoing)

- [x] Error recovery — malformed input (`1 +`, `* 5`) returns error instead of crashing
- [x] No panics on malformed input — verified by proptest with 1000+ arbitrary inputs including nested parens, long expressions, tricky edge cases
- [ ] Plugin architecture Phase 1 — internal Rust extension API for custom units, functions, global variables
- [ ] Plugin architecture Phase 2 — optional sandboxed scripting (post-core-stability)
- [ ] Performance — incremental reevaluation, no UI jank on large notes
- [ ] Clippy + rustfmt enforcement
- [x] Developer docs — architecture.md, compatibility-matrix.md, plan.md

## Current Status

**131 test functions: 92 unit + 1 golden (107 assertions) + 12 property + 4 compat-internal + 14 oracle + 8 differential fuzz. All passing. Zero warnings.**

**114/114 expressions match numi-cli semantically (0 real mismatches).**

Milestones 1-6 are substantially complete. The Tauri desktop app (M5) has a working editor with live evaluation, file operations, and result gutter. Remaining items: system tray icon, preferences UI, window state persistence, multiple notes, CI pipeline.

### How to run

```bash
# Core tests (no external deps)
cargo test --workspace --exclude elo-tauri

# Oracle + fuzz tests (requires numi-cli)
cargo test -p elo-compat -- --ignored

# CLI
cargo run -p elo-cli -- "2 + 3 * 4"

# Desktop app
cd elo-tauri && pnpm build          # build frontend
cd .. && cargo tauri dev             # run from workspace root (Tauri resolves frontend from elo-tauri/)
```
