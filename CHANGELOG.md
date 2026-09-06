# Changelog

## Unreleased

- Added opt-in formatting accessors: `get_formatted_name()`,
  `get_formatted_name_in()`, `name_components_in()`, and
  `formatted_id_number()`. Name formatting replaces the card's comma separators
  with spaces and drops empty positions; ID formatting applies the printed
  `784-YYYY-NNNNNNN-C` grouping and returns any other value unchanged.
- Added a `Gender` type with `from_code()` and `code()`, plus the `gender()`
  accessor. Unrecognized codes are preserved in `Gender::Unrecognized` rather
  than discarded. The SDK supplies no gender labels; translations stay with the
  application.
- All of the above are additive. Existing accessors, public fields, and Serde
  output are unchanged, so decoded card values are never overwritten.
- Documented the rule that decides what the SDK formats: format toward what the
  card prints, and leave anything beyond that to the application.
- Added the card-reader animation as a GIF in the README.

## 0.4.0 - 2026-09-05

- Added Windows, Linux, and macOS native transport through the portable `pcsc`
  bindings, replacing the hand-written Windows FFI.
- Preserved the public SDK API and LeaveCard disconnect behavior; transaction
  cleanup failures now propagate after an otherwise successful read.
- Added native CI builds, tests, linting, docs, and packaging on all three OSes.
- Documented platform build/runtime dependencies and hardware validation limits.

## 0.3.0 - 2026-09-05

- Added borrowed name, photo, signature, identifier, date, and nationality
  accessors, explicit bilingual selection, and fluent read options.
- Split session lifecycle, Windows transport, file protocol, and error types.
- Serialized reads per session and corrected card/context destruction order.
- Preserved reader connection failures instead of reporting every failure as
  an absent card; invalid reader names now return `InvalidArgument`.
- Limited directory fallback to absent-file statuses. Malformed image fields,
  duplicate requested tags, trailing TLV corruption, and invalid calendar dates
  now fail explicitly. Wrong-length APDU correction is limited to one retry.
- Made the diagnostic CLI opt-in via `--features cli` and redacted by default;
  personal values require `--show-personal-data`.
- Added runnable examples and synthetic SDK read-flow regression tests.
- Added complete integration/API/field guides, generated GitHub Wiki pages,
  documentation example tests, and minimum-Rust-version CI.
- `reader_names()` now returns an empty vector when PC/SC reports no readers.

### Migration from 0.2

Existing session methods and public fields remain. Add a branch for
`ErrorKind::InvalidArgument` if matching the error enum exhaustively. Enable
`cli` when running/installing the diagnostic binary. Previously tolerated
malformed data now returns an error; do not rely on partial corrupt results.

### Standalone import

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
