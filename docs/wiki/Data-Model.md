<a id="sdk-data-model"></a>

# Data model and formatting

A successful read returns an `EmiratesIdData` snapshot containing decoded card
values and a read status for each group. V1 and V2 use the same Rust types.
Optional fields use `Option`: a value is `Some(value)` when available and `None`
otherwise.

## Top-level fields

| Rust field | Meaning |
| --- | --- |
| `reader_name` | PC/SC reader that supplied this card |
| `card_generation` | `V1`, `V2`, or `Unknown`, from the published ATR list |
| `id_number` | Required 15-character Emirates ID number |
| `card_number` | Required card serial stored by the ID applet |
| `photo_jpeg` | JPEG bytes when requested and publicly readable |
| `holder_signature_image` | Signature payload when requested and available |
| `non_modifiable` | Core identity fields, borrowed by `identity()` |
| `modifiable` | Occupation, residency, passport, education, borrowed by `extended()` |
| `read_status` | Per-group access result |

The [field reference](Field-Reference) lists every field in every group.

## Empty values and inaccessible groups

Check the group status first, then the field:

- If the status is `Read`, `None` means the field was absent or decoded as empty.
- If the status is `NotRequested`, `NotAvailable`, or `Protected`, `None` does
  not tell you whether the card stores a value.

Required groups must succeed before a snapshot is returned. See
[errors and read statuses](Error-Handling) for the status definitions and
failure behavior.

## What the SDK formats

> The SDK preserves decoded card values and provides explicit helpers for
> documented name separators, identifier formatting, and known code
> interpretation. Applications control localization and presentation.

The decoder converts chip encodings into Rust values: UTF-8 text, digit strings
for packed binary-coded decimal (BCD) codes, and `YYYY-MM-DD` dates. For text,
it removes outer NUL padding and whitespace. Here, **stored** or **raw** means
that decoded value, not an untouched copy of the chip bytes.

Formatting helpers operate on the snapshot and leave its fields, original
getters, and serialization unchanged. Borrowed getters allocate no new strings;
formatted names and identifiers return owned strings. The `Gender` type
interprets known codes and preserves unrecognized ones.

For the same language, the field and original getter return the same value:

```rust,no_run
use emirates_id_reader::{CardSession, ReadOptions};

fn main() -> Result<(), emirates_id_reader::Error> {
    let session = CardSession::connect_first()?;
    let card = session.read_with_options(ReadOptions::identity_only())?;

    let field = card.identity().full_name_english.as_deref();  // public field
    let stored = card.get_name_in(emirates_id_reader::Language::English);
    let display = card.get_formatted_name_in(emirates_id_reader::Language::English);
    let _ = (field, stored, display);
    Ok(())
}
```

Applications choose labels and business rules: translating
`M` into `Male` or `ذكر`, expanding an occupation or marital-status code into a
label, deciding which name component is a family name, or deciding whether a
date makes someone eligible for something. The SDK ships no translation tables
and interprets no code except gender.

The rule applies to coded fields added later. Interpret a code into a type when
its meaning is verified and documented; do not ship a translation table for it.

## Choosing a language

`Language` is `English` or `Arabic`, and bilingual values are stored as separate
fields rather than one localized string.

- `get_name()` and `get_formatted_name()` prefer English and fall back to
  Arabic. The `_in` variants return only the language you ask for.
- `get_nationality_in()` has **no fallback**. A card that stores only an Arabic
  nationality description returns `None` for English, so request the language
  you want and handle its absence.
- Place of birth, titles, and the extended descriptions have no accessors. Read
  the paired `_arabic` and `_english` fields through `identity()` or
  `extended()` and apply your own preference.

Nationality also has an optional `nationality_code` field. Read it independently
of the descriptions; neither the code nor either description is guaranteed to
be populated.

<a id="fast-identity-only-reads"></a>

For identity-only reads and optional groups, see
[reading options](Readers-And-Sessions#reading-options).

## Topic guides

| Guide | Covers |
| --- | --- |
| [Names](Names) | Separators, components, bilingual fallback |
| [Codes and identifiers](Codes-And-Identifiers) | ID numbers, gender, coded fields |
| [Dates](Dates) | Date format, missing dates, calculating age |
| [Photos and signatures](Photos-And-Signatures) | Image groups and validation |
| [Extended information](Extended-Information) | Employment, family, passport, residency, education |
