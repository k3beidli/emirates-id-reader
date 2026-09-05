# Emirates ID Reader SDK

A Rust library for reading data from Emirates ID chips.

Read public identity fields, photographs, and holder-signature images into
structured Rust types. The library also provides reader discovery, card-presence
checks, and optional identity-only reads.

The current implementation supports Windows through PC/SC and a contact
smart-card reader. Reads run locally, and the library returns data in memory.

> [!IMPORTANT]
> This project is independent and unofficial. It is not affiliated with,
> endorsed by, or supported by the UAE Federal Authority for Identity,
> Citizenship, Customs and Port Security (ICP). Applications using this crate
> are responsible for obtaining consent and handling identity data securely.

## Status

The crate is experimental and currently supports **Windows only**. Its API and
decoded field coverage may change before version 1.0.

V1 and V2 public-data reads have been hardware-tested with an HID OMNIKEY 3x21
contact reader and cards reporting the exact ATRs published for each chip
generation. Generation describes the chip family, not the card artwork,
validity, or a fixed promise about optional fields. Unknown generations are
still probed for forward compatibility.

## Features

- Direct communication through the Windows Smart Card API (`winscard`)
- No proprietary runtime SDK or service
- V1 and V2 card-generation detection from the ATR
- Typed Rust models for identity data
- JPEG cardholder photo extraction
- Holder-signature payload extraction when available
- Optional identity-only reads for faster matching and check-in workflows
- Card-presence monitoring without repeatedly reading personal data
- Structured error kinds and ISO 7816 status words
- Serde serialization using camel-case field names

## Requirements

- Windows 10 or Windows 11
- Rust with Edition 2024 support
- The Windows **Smart Card** service
- A PC/SC-compatible contact smart-card reader and its normal CCID driver
- A supported Emirates ID card inserted chip-first in the reader

No network connection is needed at runtime.

## Installation

Use the Git repository as a Cargo dependency:

```toml
[dependencies]
emirates-id-reader = { git = "https://github.com/k3beidli/emirates-id-reader" }
```

For local development, point your application's dependency at this checkout:

```toml
[dependencies]
emirates-id-reader = { path = "../emirates-id-reader" }
```

A crates.io release is not required to use either option.

## Quick start

Connect to the first reader containing a card and read all available public
data:

```rust,no_run
use emirates_id_reader::CardSession;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let session = CardSession::connect_first()?;

    println!("Reader: {}", session.reader_name());
    println!("Card generation: {:?}", session.card_generation());

    let card = session.read()?;
    let id = card.get_id_number();
    let _ = id;
    let name = card.get_name(); // English, falling back to Arabic
    let photo = card.get_photo(); // Option<&[u8]> containing JPEG bytes
    // Bind name and photo to your UI. Getters never reread the chip.
    let _ = (name, photo);

    Ok(())
}
```

`CardSession` keeps the card connection alive. A UI can call `is_present()` to
clear identity data immediately after the holder removes the card:

```rust,no_run
use emirates_id_reader::CardSession;
use std::{thread, time::Duration};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let session = CardSession::connect_first()?;
    let card = session.read()?;

    println!("Scanned: {}", card.id_number);

    while session.is_present()? {
        thread::sleep(Duration::from_millis(200));
    }

    println!("Card removed; clear the displayed identity data now.");
    Ok(())
}
```

## Faster identity-only reads

Photos, signatures, and extended modifiable data take longer to transfer. If
an application only needs the card number and core identity fields, use
`ReadOptions::identity_only()`:

```rust,no_run
use emirates_id_reader::{CardSession, ReadOptions};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let session = CardSession::connect_first()?;
    let card = session.read_with_options(ReadOptions::identity_only())?;

    println!("{}", card.id_number);
    Ok(())
}
```

The returned `read_status` distinguishes data that was read, not requested,
unavailable, or protected.

## Simple accessors

```rust,no_run
use emirates_id_reader::{CardSession, Language, ReadOptions};
# fn main() -> Result<(), emirates_id_reader::Error> {
let session = CardSession::connect_first()?;
let options = ReadOptions::identity_only().with_photo(true);
let card = session.read_with_options(options)?;
let name = card.get_name();
let arabic_name = card.get_name_in(Language::Arabic);
let photo = card.get_photo();
let birthday = card.get_date_of_birth();
let expiry = card.get_expiry_date();
let passport = card.extended().passport_number.as_deref(); // None: extended data skipped
# Ok(())
# }
```

Methods use Rust's `snake_case`: `get_name()` and `get_photo()` are the Rust
counterparts of `getName()` and `getPhoto()`. This release is a Rust SDK;
JavaScript, Python, C, and .NET bindings are not included. Existing public
fields remain available, including every bilingual and extended field.

Getters borrow a completed snapshot, allocate nothing, and perform no I/O.
`get_name()` prefers English and falls back to Arabic; `get_name_in()` does
not fall back. `get_photo()` returns JPEG bytes or `None`; inspect
`card.read_status.photo` to distinguish skipped or inaccessible files.

Run the examples with `cargo run --example read_identity`,
`cargo run --example read_photo`, or `cargo run --example watch_removal`.
Generate the complete API reference with `cargo doc --no-deps --open`.

## Included command-line utility

The optional `cli` feature builds the diagnostic program. The default package is library-only:

```powershell
# Confirm that a reader and card can be reached
cargo run --release --features cli -- probe

# Validate a read without printing identity values
cargo run --release --features cli -- read --redacted

# Validate only identifiers and core identity fields (faster)
cargo run --release --features cli -- read --redacted --identity-only

# Explicitly print basic identity values locally
cargo run --release --features cli -- read --show-personal-data
```

The CLI is opt-in and redacts reads by default. `--show-personal-data` prints personal data to the terminal. Use it only with
the cardholder's permission and avoid saving terminal output.

## Data returned

The library currently decodes:

- Emirates ID number and card number
- Card generation and reader name
- JPEG cardholder photograph
- ID type, issue date, and expiry date
- Arabic and English title and full name
- Gender
- Arabic and English nationality and nationality code
- Date and place of birth in Arabic and English
- Occupation code and occupation in Arabic and English
- Family ID, occupation type and field, and company name
- Marital status and husband ID number
- Sponsor type, unified number, and name
- Residency type, number, and expiry date
- Passport number, type, country, issue date, and expiry date
- Qualification, degree, field and place of study, and graduation date
- Mother's name in Arabic and English
- Holder-signature image payload

Fields are optional because the information populated on a card varies by
holder, card generation, and issuance.

## Protected data

Some fields described as public in available card documentation reject a plain
unauthenticated read with ISO 7816 status `6982`. This crate deliberately does
not attempt to bypass that protection.

Currently skipped protected fields include:

- Home and work address details
- Resident and mobile phone numbers
- Email address

The crate does not contain proprietary authentication keys or toolkit
credentials.

## Error handling

Errors contain a high-level `ErrorKind`, a human-readable message, and an
optional ISO 7816 status word:

```rust,no_run
use emirates_id_reader::{CardSession, ErrorKind};

match CardSession::connect_first() {
    Ok(session) => println!("Connected to {}", session.reader_name()),
    Err(error) if error.kind == ErrorKind::NoReader => {
        eprintln!("Connect a smart-card reader");
    }
    Err(error) if error.kind == ErrorKind::NoCard => {
        eprintln!("Insert an Emirates ID");
    }
    Err(error) => eprintln!("Reader error: {error}"),
}
```

## Privacy and security

An Emirates ID contains sensitive personal information. Software built with
this crate should:

- Read cards only with the holder's knowledge and permission
- Keep the minimum data required for the application's purpose
- Avoid logging IDs, names, photographs, passport details, or raw card data
- Clear in-memory and on-screen data promptly after card removal
- Encrypt any personal data that must be stored
- Restrict access and follow applicable UAE privacy and data-protection rules
- Never commit real card dumps, photographs, or identity details to source
  control or test fixtures

The crate itself performs no persistence and sends no network requests.

More detail is available in the project documentation:

- [V1 and V2 field matrix](docs/card-generations.md)
- [Data model and read status](docs/data-model.md)
- [Hardware validation checklist](docs/testing.md)
- [Security boundaries](docs/security.md)

## How it works

The library uses Windows PC/SC to locate a smart-card reader, opens a shared
card session, selects the Emirates ID application and public-data files using
ISO 7816 APDUs, and decodes the returned TLV containers into typed Rust
structures.

The low-level PC/SC handles are owned by `CardSession` and released
automatically when the session is dropped.

## Development

Clone and enter the standalone project:

```powershell
git clone https://github.com/k3beidli/emirates-id-reader.git
cd emirates-id-reader
```

The project contains the library and a diagnostic CLI. Applications such as
attendance terminals consume the crate separately.

- `src/lib.rs`: SDK exports and compiled quick start
- `src/session.rs`: public session API and serialized read lifecycle
- `src/transport.rs`: Windows PC/SC handles and FFI
- `src/protocol.rs`: testable application selection and public-file reads
- `src/error.rs`: structured error types
- `src/data.rs`: public data models, read options, and card generations
- `src/decode.rs`: public-file TLV and field decoding
- `src/apdu.rs`: ISO 7816 response handling
- `src/main.rs`: diagnostic CLI
- `src/tests.rs`, `src/sdk_tests.rs`: synthetic parser, APDU, and SDK read tests
- `examples/`: runnable consumer examples


Run the test suite:

```powershell
cargo test
```

Build the release binary:

```powershell
cargo build --release --features cli
```

Hardware-independent unit tests cover TLV parsing, packed-BCD decoding, date
decoding, and typed field mapping. Hardware validation requires a compatible
reader and card.

## Contributing

Bug reports and contributions are welcome, especially for:

- Additional card and reader compatibility reports
- More hardware-independent parser tests using synthetic data
- Improved error messages and recovery behavior
- Carefully reviewed support for other desktop operating systems

Do not submit real identity data, card dumps, proprietary SDK files,
authentication material, copyrighted documentation, or third-party binaries.

## License

MIT. See [LICENSE](LICENSE).
