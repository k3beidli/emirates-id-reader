# Changelog

## Unreleased

- Extracted the reader into a standalone repository with Windows CI.
- Separated public data models, field decoding, and APDU handling into modules.
- Preserved the existing public API and diagnostic CLI.

## 0.2.0 - 2026-09-04

- Added documented V1/V2 ATR classification.
- Added ISO 7816 T=0 `61xx`/`GET RESPONSE`, `6Cxx` length correction, and
  `6282` EOF-warning handling.
- Added public-file layout probing instead of an ATR-dependent path guess.
- Added `ReadOptions` and a fast identity-only read.
- Added explicit per-group status for protected, unavailable, and skipped
  optional files.
- Added named-reader discovery and connection APIs.
- Added V1/V2 field, privacy, and hardware-validation documentation.
- Restricted the crates.io package to SDK source and documentation.

## 0.1.0 - 2026-09-03

- Initial direct Windows PC/SC reader, typed V2 public-data parser, diagnostic
  CLI, and Tauri integration.
