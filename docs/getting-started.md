<a id="getting-started"></a>

# Your first read

Read one card and access its values without making a separate chip request for
each field.

<a id="requirements"></a>
<a id="add-the-dependency"></a>

First, follow [installation and platforms](platforms.md), connect your reader,
and insert an Emirates ID with the chip facing the reader's contacts.

## Read a card

```rust,no_run
use emirates_id_reader::{CardSession, Language, ReadOptions};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let session = CardSession::connect_first()?;
    let card = session.read_with_options(
        ReadOptions::identity_only().with_photo(true),
    )?;

    let id = card.formatted_id_number();
    let name = card.get_formatted_name();
    let arabic_name = card.get_formatted_name_in(Language::Arabic);
    let photo = card.get_photo();
    // Pass these values to your application's UI.
    // The example deliberately does not print identity values.
    let _ = (id, name, arabic_name, photo);
    Ok(())
}
```

`connect_first()` opens a connection to an inserted card. `read_with_options()`
reads the selected groups into an `EmiratesIdData` snapshot. Getters use that
snapshot without additional chip communication; formatting helpers create new
strings. The snapshot remains usable after the session is dropped.

<a id="choose-a-reader"></a>

## Understanding the example

| API used | Guide |
| --- | --- |
| `connect_first()`, `read_with_options()` | [Readers, sessions, and reading options](readers-and-sessions.md) |
| `get_formatted_name()`, `get_formatted_name_in()` | [Names](names.md) |
| `formatted_id_number()` | [Codes and identifiers](codes-and-identifiers.md) |
| `get_photo()` | [Photos and signatures](photos-and-signatures.md) |

`ReadOptions::identity_only()` keeps the read fast by skipping the expensive
optional groups; `.with_photo(true)` adds one back. Use `session.read()` when
you want every public group supported by this SDK.

Optional values return `None` when they are absent. That is not the same as a
group being unreadable, so inspect `card.read_status` when the difference
matters; see [errors and read statuses](error-handling.md).

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

The CLI is optional and redacts reads by default. To deliberately print basic
personal values, use `read --show-personal-data`.

## Next

- [Data model and formatting](data-model.md): what the SDK decodes, and what it
  leaves to you
- [Application integration](integration.md): UI workers, card removal, ownership
- [Troubleshooting](troubleshooting.md): setup and common failures
