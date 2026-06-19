# Changelog

All notable changes to Elo are documented here. The format is based on
[Keep a Changelog](https://keepachangelog.com/), and the project adheres to
[Semantic Versioning](https://semver.org/).

## [0.3.1] — 2026-06-19

- Universal macOS binary supporting both Apple Silicon and Intel.
- Build and packaging fixes for the macOS, Linux, and Arch (`PKGBUILD`)
  release pipeline.

## [0.2.4] — 2026-04-06

- Errors are now surfaced in the UI instead of failing silently.
- Fixed list-item parsing edge cases.

## [0.2.3] — 2026-04-05

- Fixed saving files on Linux (added filesystem read/write permissions).

## [0.2.2] — 2026-04-05

- Markdown comprehension in the UI: headers, comments, lists, and code fences
  are understood and rendered.
- Fixed evaluation inside lists and code fences.

## [0.2.1] — 2026-03-29

- Fixed a parser bug in timezone conversion.
- Added the Arch `PKGBUILD` for Linux packaging.

## [0.2.0] — 2026-03-29

- Currency conversion (`$100`, `€50`, `100 USD`).

## [0.1.0] — 2026-03-29

Initial release — a notepad-style calculator for natural math expressions,
inspired by [Numi](https://numi.app/), shipping as a CLI and a Tauri desktop
app. Highlights:

- Arithmetic with operator precedence and functions (`sqrt`, `log`, trig,
  factorials), plus word operators (`plus`, `times`, `multiplied by`).
- Number formats: decimal, hex, binary, octal, scientific, comma-separated.
- Units across length, area, volume, mass, time, temperature, angle, data,
  CSS, and speed — with natural conversion syntax and unit sequences
  (`5 feet 3 inches`).
- Percentages (`15% of 200`, `25% off 80`), scales (`k`, `million`), and
  format conversions (`as hex`, `in scientific`).
- Dates and timezones (`now`, `today + 3 days`, `time in Asia/Tokyo`).
- Variables and sessions (`=`, `prev`, `sum`, `avg`), labels and comments.
- Locale-sensitive number formatting (`en`, `de`, `fr`).
- File operations: open, save, and export notes as `.elo`, `.txt`, or `.md`.
