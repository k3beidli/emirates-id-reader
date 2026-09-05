# Getting started

Emirates ID Reader is an unofficial Rust SDK for local public-data extraction
from contact chips. It uses Windows PC/SC directly. It requires no proprietary
toolkit, background application server, or runtime network connection.

## Requirements

- Windows 10 or 11, with the Windows Smart Card service available.
- Rust 1.85 or newer and a working Windows Rust linker/build toolchain.
- A PC/SC contact reader with its normal driver, and an inserted Emirates ID.

The current transport is Windows-only. Linux, macOS, browser/WebUSB, mobile,
contactless NFC, and bindings for other languages are not implemented.

## Add the dependency

```toml
[dependencies]
emirates-id-reader = { git = "https://github.com/k3beidli/emirates-id-reader" }
```

For development alongside this checkout, use a path dependency instead:

```toml
[dependencies]
emirates-id-reader = { path = "../emirates-id-reader" }
```

Commit your application's `Cargo.lock` to retain its resolved Git revision.
For a release, you can also add `rev = "<full reviewed commit SHA>"` to the Git
dependency. Version 0.3.0 is the package version in this repository; these
instructions do not assume a crates.io publication or Git release tag.

## Read a card

```rust,no_run
use emirates_id_reader::{CardSession, Language, ReadOptions};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let session = CardSession::connect_first()?;
    let card = session.read_with_options(
        ReadOptions::identity_only().with_photo(true),
    )?;

    let id = card.get_id_number();
    let name = card.get_name();
    let arabic_name = card.get_name_in(Language::Arabic);
    let photo = card.get_photo();
    // Pass these borrowed values to your application's UI.
    // The example deliberately does not print identity values.
    let _ = (id, name, arabic_name, photo);
    Ok(())
}
```

`get_name()` prefers English, with Arabic as a fallback. `get_name_in()`
returns only the requested language. Names retain the card's content,
including punctuation; the SDK does not transliterate or split names.

`get_photo()` returns `Option<&[u8]>`. A missing value can mean a blank field,
a skipped read, a missing file, or a protected file. Inspect
`card.read_status.photo` for the group outcome. The photo accessor does not
trigger another card read. Use `session.read()` to request all public groups.

## Choose a reader

Call `CardSession::reader_names()` and present the returned names in your UI.
Pass the exact selected name to `CardSession::connect(&name)`. An empty list
means there are no installed readers; a stopped or unavailable PC/SC service
can instead return an error. `connect_first()` chooses the first accessible
reader with any card, and verifies the Emirates ID application when you read.
It does not search every reader for a particular cardholder or card type.

## Try the examples

From this repository:

```powershell
cargo run --example read_identity
cargo run --example read_photo
cargo run --example watch_removal
cargo run --features cli -- --help
cargo run --features cli -- probe
cargo run --features cli -- read --identity-only
```

The CLI is optional and reads with redacted output by default. To deliberately
print basic personal values, use `read --show-personal-data`.

Continue with [API reference](api-reference.md), [application integration](integration.md),
and [troubleshooting](troubleshooting.md).
