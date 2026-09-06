<a id="getting-started"></a>

# Your first read

Read one card and access its values without making a separate chip request for
each field.

<a id="requirements"></a>
<a id="add-the-dependency"></a>

First, follow [installation and platforms](Platforms), connect your reader,
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
    let name = card.formatted_name();
    let arabic_name = card.formatted_name_in(Language::Arabic);
    let photo = card.photo();
    // Pass these values to your application's UI.
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
| `connect_first()`, `read_with_options()` | [Readers, sessions, and reading options](Readers-And-Sessions) |
| `formatted_name()`, `formatted_name_in()` | [Names](Names) |
| `formatted_id_number()` | [Codes and identifiers](Codes-And-Identifiers) |
| `photo()` | [Photos and signatures](Photos-And-Signatures) |

`ReadOptions::identity_only()` keeps the read fast by skipping the expensive
optional groups; `.with_photo(true)` adds one back. Use `session.read_all()` when
you want every public group supported by this library.

Optional values return `None` when they are absent. That is not the same as a
group being unreadable, so inspect `card.read_status()` when the difference
matters; see [errors and read statuses](Error-Handling).

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

- [Data model and formatting](Data-Model): what the library decodes, and what it
  leaves to you
- [Application integration](Application-Integration): UI workers, card removal, ownership
- [Troubleshooting](Troubleshooting): setup and common failures
