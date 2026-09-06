<p align="center">
  <img src="docs/media/card-reader.gif" width="240" alt="Animated Emirates ID card being inserted into a chip reader">
</p>

<h1 align="center">Emirates ID Reader SDK</h1>

A Rust SDK for reading public identity data from Emirates ID contact chips.
Use a compatible reader on Windows, Linux, or macOS to read names, photographs,
dates, and other public fields into memory. Access the result through Rust
getters, with optional helpers for display formatting.

## Installation

```toml
[dependencies]
emirates-id-reader = { git = "https://github.com/k3beidli/emirates-id-reader" }
```

This installs directly from GitHub. Keep your application's `Cargo.lock` under
version control to retain the resolved revision. Native prerequisites differ per platform, and Linux
needs pcsc-lite development files before the crate will build. See
[installation and platforms](docs/platforms.md).

<a id="reading-and-accessing-data"></a>

<a id="stored-values-and-formatted-values"></a>

## Example

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

[Your first read](docs/getting-started.md) walks through this line by line.

## Status and scope

**Experimental, Windows / Linux / macOS, Rust 1.85+.** Reads use the operating
system's PC/SC service and a compatible contact reader with its normal driver.
Everything is local: the library makes no network requests and persists nothing.

V1 and V2 share one data model. Historical hardware tests with an HID OMNIKEY
3x21 came with the imported project; the 0.4 SDK has synthetic coverage and
needs fresh hardware validation, tracked in
[testing](docs/testing.md). Unknown ATRs are probed, without a guarantee of
support for future generations.

This project is unofficial and is not affiliated with or endorsed by ICP. It
extracts publicly accessible fields; it does not authenticate cards, read
fingerprints, or bypass protected files. Several documented fields are
deliberately never requested, and bindings for Java, JavaScript, Python, C, and .NET are not included.
Mobile and contactless reading are not supported. See
[security and access boundaries](docs/security.md).

Card data is personal data. Do not log the snapshot or its derived `Debug`
output, collect only the fields you need, and never commit card dumps,
photographs, or identity details to this repository.

<a id="examples-and-diagnostic-cli"></a>

## Documentation

Choose a starting point below, or browse the
[documentation home](docs/wiki-home.md) for an overview.

- Get running: [installation and platforms](docs/platforms.md),
  [your first read](docs/getting-started.md)
- Work with a value: [data model and formatting](docs/data-model.md),
  [names](docs/names.md), [codes and identifiers](docs/codes-and-identifiers.md),
  [dates](docs/dates.md), [photos and signatures](docs/photos-and-signatures.md),
  [extended information](docs/extended-information.md)
- Look something up: [API reference](docs/api-reference.md),
  [field reference](docs/field-reference.md),
  [errors and read statuses](docs/error-handling.md),
  [V1/V2 compatibility](docs/card-generations.md)
- Go deeper: [architecture and chip communication](docs/architecture.md),
  [application integration](docs/integration.md),
  [troubleshooting](docs/troubleshooting.md)

Generate the API documentation matching your checkout with
`cargo doc --no-deps --open`.

<a id="development"></a>

## Credits

The field reference and compatibility notes draw on ICP's V1/V2 field
specifications and EIDA Toolkit documentation. Native reader access uses
[pcsc-rust](https://github.com/bluetech/pcsc-rust). See
[sources and acknowledgments](docs/sources.md).

Contributions are welcome; see [contributing](CONTRIBUTING.md) for the local
checks, and [documentation maintenance](docs/wiki-setup.md) for how the guides
and Wiki are generated.

## License

MIT. See [LICENSE](LICENSE).
