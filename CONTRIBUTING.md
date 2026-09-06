# Contributing

Keep the library independent of any consuming application. Public API additions
need Rustdoc and a consumer-facing example. Preserve explicit missing-data
semantics and add synthetic regression cases for parser/protocol changes.
Never commit real identities, images, card dumps, credentials, proprietary
toolkit files, or third-party binaries.

## Local checks

```powershell
cargo fmt --all -- --check
cargo clippy --locked --all-targets --all-features -- -D warnings
cargo test --locked --all-features
$env:RUSTDOCFLAGS = '-D warnings'
cargo doc --locked --no-deps
cargo package --locked --all-features
python scripts/build_wiki.py --check
```

`cargo package` expects committed changes; use `--allow-dirty` only for local
precommit verification. It builds a local archive and does not publish it.
Examples are built by the all-targets Clippy check. Unit tests do not require
hardware. Reader/driver validation must follow [testing and hardware validation](docs/testing.md) separately.

## Documentation

Edit canonical guides in `docs/`, then run `python scripts/build_wiki.py`.
Commit generated `docs/wiki/` pages alongside the guide changes. The generator
also derives field tables from public Rustdoc comments in `src/data.rs`.
Do not edit generated Wiki pages directly. See [documentation maintenance](docs/wiki-setup.md) for the
separate GitHub Wiki publishing procedure.

## Releases

Update the package version, lockfile, and changelog together.
Check the declared minimum Rust version as well as current stable. Validate
the packed crate and relevant V1/V2 hardware flows before declaring hardware
support for a release. Publishing a crate or release tag is a separate step;
the Git dependency remains usable without either.
