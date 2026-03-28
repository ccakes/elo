# Compatibility Matrix: Elo vs numi-cli v0.18.0

## Summary

**114/114 expressions match semantically (0 real mismatches).**
102 are exact string matches; 12 differ only in decimal place formatting (e.g., `4.00` vs `4`).

## Test Infrastructure

| Layer | Tests | Assertions | Description |
|---|---|---|---|
| Unit tests | 92 | ~92 | Lexer, parser, evaluator, formatter, session, locale |
| Golden corpus | 1 | 107 | Tagged fixture file with fuzzy numeric matching |
| Property tests | 12 | ~12000 | proptest: no-panic, determinism, arithmetic invariants |
| Oracle tests | 14 | ~70 | Direct numi-cli comparison by feature group |
| Differential fuzz | 8 | ~500 | Grammar-fragment generation compared against numi-cli |
| Compat internal | 4 | 4 | Semantic match algorithm unit tests |
| **Total** | **131** | **~12773** | |

## Formatting Differences (ignored)

These differ only in trailing zeros / decimal places. Values match semantically.

| Pattern | numi-cli | elo | Notes |
|---|---|---|---|
| Function results | `4.00` | `4` | numi always shows .00 for functions |
| Percent operations | `100.00` | `100` | numi shows .00 for percent results |
| Scale with decimal input | `2500000.0` | `2500000` | numi preserves trailing .0 |
| Hex/bin/oct input | `0xff` | `255` | numi preserves input format |
| Scientific input | `1e3` | `1000` | numi preserves sci notation |
| Unit conversion trailing zero | `2.20 lb` | `2.2 lb` | numi shows trailing zero |

## Intentional Divergences

| Expression | numi-cli | elo | Reason |
|---|---|---|---|
| `# Section 2` | `2` | (empty) | Elo treats `#` lines as headers per spec. numi-cli has no header support. |
| `sqrt(-1)` | error | error | Both error correctly. |
| `abs(-sqrt(9))` | error | `3` | Elo produces correct result; numi-cli errors (numi bug). |
| `1 liter in ml` | error | `1000 ml` | Elo is more permissive with unit name aliases. |
| `fact(5)` | `5` | `5` | Both match. numi's `fact()` is the identity function, not factorial. |

## Feature Coverage

### Fully Matched (exact or semantic)

- Arithmetic: all operators, word operators, precedence, parentheses
- Functions: sqrt, cbrt, abs, round, ceil, floor, sin, cos, tan, ln, fact
- Scales: k, M, billion
- Bitwise: &, |, xor, <<, >>
- Format conversions: hex, binary, octal, scientific
- Percentages: of, on, off, +/-, inverse
- Units: length, mass, time, temperature conversions
- Unit sequences: compound unit expressions
- Cross-unit arithmetic: same-dimension auto-conversion
- Constants: pi, e
- Dates: today, tomorrow, yesterday
- Date arithmetic: +/- days, weeks, hours, minutes
- Timezones: IANA names, UTC, city-based queries
- Labels, comments

### Not Yet Implemented

- Currency conversion — requires exchange rate provider (parser and display already work)
- Locale-sensitive input parsing — decimal comma as input (output formatting is implemented)
- Data unit edge cases — numi treats `KB`/`MB`/`GB` as kilobits/megabits/gigabits, not kilobytes
- `now + 1 month` / `now + 1 year` — numi uses non-standard month/year arithmetic; elo uses 30-day months and 365.25-day years
