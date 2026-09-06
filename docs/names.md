# Names

Use `formatted_name()` for display and `name()` when you need the
original decoded value. Names can contain comma separators; the formatted
getter joins the populated components with spaces.

For example, the synthetic value `AHMED,ALI,,ALKAABI` becomes
`AHMED ALI ALKAABI`. No card data is modified.

```rust,no_run
use emirates_id_reader::{CardSession, Language, ReadOptions};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let session = CardSession::connect_first()?;
    let card = session.read_with_options(ReadOptions::identity_only())?;

    let display = card.formatted_name();
    let arabic = card.formatted_name_in(Language::Arabic);
    let stored = card.name();
    let _ = (display, arabic, stored);
    Ok(())
}
```

## Accessors

| Method | Returns | Value |
| --- | --- | --- |
| `formatted_name()` | `Option<String>` | Display name, English with Arabic fallback |
| `formatted_name_in(Language)` | `Option<String>` | Display name in one language |
| `name()` | `Option<&str>` | Stored name, English with Arabic fallback |
| `name_in(Language)` | `Option<&str>` | Stored name in one language |
| `name_components_in(Language)` | `impl Iterator<Item = &str>` | Stored components, card order |

The `_in` accessors never fall back. Missing text returns `None`; the component
iterator instead yields no items. Titles are separate fields, `title_english` and `title_arabic` on
[`identity()`](field-reference.md), and are not part of any name accessor.

## Formatting

`formatted_name_in()` replaces commas with single spaces, drops empty
positions, and trims each component. Whitespace inside a component is
preserved, as are capitalization, spelling, diacritics, and component order. The library never transliterates a name and never decides which
position holds a given name or a family name.

`name_components_in()` keeps the empty positions, trimming each component, so a
caller that needs the card's own structure can still see it. A value with no
comma yields exactly one component; an absent field yields nothing.

## Two fallbacks that differ

`name()` falls back to Arabic when the English field is absent.
`formatted_name()` also falls back when the English field is *present but
holds only separators and whitespace*, because such a field has no formatted
value. In that case `name()` returns the stored separators while
`formatted_name()` returns the Arabic name.

Both return `None` only when neither language yields a value.

## Formatting without a card

Every accessor reads an owned snapshot, so tests can build one directly:

```rust
use emirates_id_reader::{EmiratesIdData, Language, NonModifiableData};

let mut identity = NonModifiableData::default();
identity.full_name_english = Some("AHMED,ALI,,ALKAABI".to_string());
let card = EmiratesIdData::builder("784198512345671", "123456789")
    .identity(identity)
    .build()?;
assert_eq!(card.formatted_name().as_deref(), Some("AHMED ALI ALKAABI"));
assert_eq!(card.name(), Some("AHMED,ALI,,ALKAABI"));
assert_eq!(
    card.name_components_in(Language::English).collect::<Vec<_>>(),
    ["AHMED", "ALI", "", "ALKAABI"],
);
# Ok::<(), emirates_id_reader::Error>(())
```

## Why names contain commas

The [V1 field specification](sources.md#fields-stored-in-uae-id-card-v1)
explicitly lists six comma separators in both full-name fields. Six separators
divide a string into seven positions. Consecutive separators leave empty
positions: `A,,,B` contains two empty components between `A` and `B`.

The field list does not define the meaning of each position. The library therefore
preserves their order without assigning first-name or surname roles. It also
accepts names with a different number of separators or no commas.

## Related

- [Data model and formatting](data-model.md) for the policy behind the split
- [Field reference](field-reference.md) for the underlying fields
- [V1/V2 compatibility](card-generations.md): both generations store Arabic and
  English full names
