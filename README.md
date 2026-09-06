<p align="center">
  <img src="docs/media/card-reader.gif" width="240" alt="Animated Emirates ID card being inserted into a chip reader">
</p>

<h1 align="center">Emirates ID Reader SDK</h1>

A Rust SDK for reading public identity data from Emirates ID contact chips.
Connect a PC/SC reader, read one snapshot, and access names, photographs,
dates, and other fields through a documented API.

```rust,no_run
use emirates_id_reader::{CardSession, Language, ReadOptions};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let session = CardSession::connect_first()?;
    let card = session.read_with_options(
        ReadOptions::identity_only().with_photo(true),
    )?;

    let name = card.get_formatted_name();
    let arabic_name = card.get_formatted_name_in(Language::Arabic);
    let photo = card.get_photo();
    let id = card.formatted_id_number();
    let expiry = card.get_expiry_date();
    // Bind these values to your UI; no additional chip reads occur.
    let _ = (name, arabic_name, photo, id, expiry);
    Ok(())
}
```

## Status and scope

**Experimental, Windows / Linux / macOS, Rust 1.85+.** Uses the operating
system PC/SC service and a compatible contact reader with its normal driver. Runtime reads
are local; the library makes no network requests and does not persist data.

V1 and V2 use one data model. Historical hardware tests with an HID OMNIKEY
3x21 came with the imported project. The 0.4 SDK has automated synthetic
coverage and requires fresh hardware validation; see [testing](docs/testing.md).
Unknown ATRs are probed, without a guarantee of support for future generations.

This project is unofficial and is not affiliated with or endorsed by ICP.
It extracts publicly accessible fields; it does not authenticate cards, read
fingerprints, or bypass protected files. JavaScript, Python, C, .NET, mobile, and contactless bindings/backends
are not included. See [platform setup](docs/platforms.md) for native prerequisites.

## Installation

Use this repository as a dependency:

```toml
[dependencies]
emirates-id-reader = { git = "https://github.com/k3beidli/emirates-id-reader" }
```

Or consume a local checkout:

```toml
[dependencies]
emirates-id-reader = { path = "../emirates-id-reader" }
```

Keep your application's `Cargo.lock` under version control. No crates.io
publication is required. See [getting started](docs/getting-started.md) for
platform prerequisites, revision pinning, and reader selection.

## Reading and accessing data

| Need | API |
| --- | --- |
| Discover readers | `CardSession::reader_names()` |
| Choose a reader | `CardSession::connect(&name)` |
| Connect to the first accessible inserted card | `CardSession::connect_first()` |
| Read all public groups | `session.read()` |
| Read identity only | `session.read_with_options(ReadOptions::identity_only())` |
| Add just the photo | `ReadOptions::identity_only().with_photo(true)` |
| Name for display, English with Arabic fallback | `card.get_formatted_name()` |
| Name for display in one language | `card.get_formatted_name_in(Language::Arabic)` |
| Stored name, separators intact | `card.get_name()`, `card.get_name_in(Language::Arabic)` |
| Individual stored name components | `card.name_components_in(Language::English)` |
| JPEG bytes | `card.get_photo()` |
| Signature payload | `card.get_signature()` |
| Identifier as printed | `card.formatted_id_number()` |
| Identifier, birthday, expiry | `get_id_number()`, `get_date_of_birth()`, `get_expiry_date()` |
| Interpreted gender code | `card.gender()` |
| Stored gender code | `card.get_gender()` |
| Every core/extended field | `card.identity()`, `card.extended()` |
| Monitor removal | `session.is_present()` |

The Rust equivalents of `getName()` and `getPhoto()` are `get_name()` and
`get_photo()`. Getters read an owned snapshot and never access the chip.
Existing public fields remain available for compatibility.

### Stored values and formatted values

The card stores some values in a form built for machines rather than for
display: names are comma-delimited, and the identifier is fifteen unbroken
digits. The SDK never overwrites what it decoded. Every raw getter, public
field, and serialized value keeps returning the stored form, and formatting is
opt-in through a separate getter.

| Stored | Formatted |
| --- | --- |
| `get_name()` → `Some("AHMED,ALI,,ALKAABI")` | `get_formatted_name()` → `Some("AHMED ALI ALKAABI")` |
| `get_id_number()` → `"784198512345671"` | `formatted_id_number()` → `"784-1985-1234567-1"` |
| `get_gender()` → `Some("M")` | `gender()` → `Some(Gender::Male)` |

Formatting reproduces what the card prints: separators become spaces, and the
identifier takes its printed grouping. Anything beyond that stays with the
application. `gender()` returns a typed value rather than a label because
`Male` and `ذكر` are translations, not card data, and the card's own `Sex`
field prints `M`. Unrecognized codes are preserved in `Gender::Unrecognized`
so an unknown value never becomes an absent one.

`name_components_in()` borrows the stored components in card order and keeps
empty positions, so nothing is lost; `get_formatted_name()` drops them. The SDK
does not identify which position holds a given name or a family name.

Optional values return `None` when absent. Inspect `card.read_status` to
distinguish `Read`, `NotRequested`, `NotAvailable`, and `Protected` groups.
A read group can still have blank fields. Malformed data and transport
failures return an error rather than a partial snapshot.

Dates are validated `YYYY-MM-DD` strings; identifiers and codes retain leading
zeroes. Photos are JPEG bytes, while signature format may vary. Public data
models support Serde serialization with camel-case field names. See the
[API reference](docs/api-reference.md) and [complete field list](docs/field-reference.md).

Reads are blocking: use a worker in UI/async applications. Concurrent reads
on one session are serialized. Keep the session for presence checks, clear
application state on removal, and reconnect after reinsertion. Dropping the
session releases its native resources automatically.

## Examples and diagnostic CLI

```powershell
cargo run --example read_identity
cargo run --example read_photo
cargo run --example watch_removal

# The diagnostic binary is an opt-in feature.
cargo run --features cli -- probe
cargo run --features cli -- read
cargo run --features cli -- read --identity-only

# Explicitly display basic personal values locally.
cargo run --features cli -- read --show-personal-data
```

CLI reads redact by default. Avoid logging the snapshot or its derived
`Debug` output: it contains personal data. Obtain the cardholder's permission
and collect only the fields your application needs.

## Documentation

- [Getting started](docs/getting-started.md)
- [API reference](docs/api-reference.md), [data model](docs/data-model.md), and [field reference](docs/field-reference.md)
- [Application integration](docs/integration.md) and [error handling](docs/error-handling.md)
- [V1/V2 compatibility](docs/card-generations.md) and [hardware testing](docs/testing.md)
- [Troubleshooting](docs/troubleshooting.md) and [security boundaries](docs/security.md)
- [Architecture](docs/architecture.md) and [migration from 0.2](docs/migration.md)
- [GitHub Wiki setup](docs/wiki-setup.md) and [prepared Wiki pages](docs/wiki/Home.md)

Generate the version-matched API documentation with:

```powershell
cargo doc --no-deps --open
```

Wiki source pages are prepared in this repository. Publishing them to
GitHub's separate Wiki repository is described in the setup guide; pushing
`main` alone does not publish the Wiki.

## Development

```powershell
cargo fmt --all -- --check
cargo clippy --locked --all-targets --all-features -- -D warnings
cargo test --locked --all-features
python scripts/build_wiki.py --check
```

The library separates sessions, native PC/SC transport, file reading, APDU
handling, data models, and decoding. Tests use synthetic data and emulate
card commands without accessing a physical card. See [contributing](CONTRIBUTING.md)
for minimum-version checks, packaging, and documentation maintenance.

Do not submit real card dumps, photographs, identity details, proprietary
SDK files, authentication material, or third-party binaries.

## License

MIT. See [LICENSE](LICENSE).
