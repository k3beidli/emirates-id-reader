# Changelog

## 0.1.0

Initial SDK baseline. Earlier version labels in the repository belonged to
in-development iterations; this SDK's version numbering starts at **0.1.0**.
The Git history retains the original development changes.

### Reading cards

- Native contact PC/SC support on Windows, Linux, and macOS.
- Reader discovery, explicit reader selection, card connections, and presence checks.
- V1/V2 ATR classification with a shared data model and optional field values.
- Read options for photographs, extended information, and holder-signature images.
- Per-group statuses for read, skipped, unavailable, and protected files.
- Serialized session reads, bounded APDU exchanges, and validated UTF-8, TLV,
  packed BCD, identifier, and date decoding.

### Accessing data

- Borrowed getters for names, images, identifiers, dates, and nationality.
- Optional name and ID formatting, preserving original decoded fields and serialization.
- Name-component access that preserves empty positions.
- Typed gender interpretation with unrecognized codes preserved; display labels
  remain an application choice.
- In-memory results with Serde serialization and an opt-in diagnostic CLI that
  redacts personal values by default.

### Documentation and verification

- Topic guides, API and field references, technical explanations, and source credits.
- Generated GitHub Wiki pages with grouped navigation and stable page names.
- Synthetic protocol/API tests and compiled documentation examples.
- CI for native builds, formatting, linting, documentation, packaging, and the
  minimum Rust version. Hardware validation is documented separately.
- SDK package contents exclude the local companion application.

See the [Wiki](https://github.com/k3beidli/emirates-id-reader/wiki) for usage,
compatibility, and testing details.
