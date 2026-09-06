# Migrating from the extracted project

The untouched standalone import is commit `14e415a`. The SDK refactor starts
with package version 0.1.0. Existing `CardSession` methods and public data
fields remain available.

## Simpler data access

| Existing expression | SDK accessor |
| --- | --- |
| `card.non_modifiable.full_name_english.as_deref()` | `card.get_name_in(Language::English)` |
| English name with Arabic fallback | `card.get_name()` |
| `card.photo_jpeg.as_deref()` | `card.get_photo()` |
| `card.holder_signature_image.as_deref()` | `card.get_signature()` |
| `card.id_number.as_str()` | `card.get_id_number()` |
| `card.non_modifiable.date_of_birth.as_deref()` | `card.get_date_of_birth()` |
| `&card.modifiable` | `card.extended()` |

The getters above borrow the same result rather than transferring or cloning fields.
No renaming of your application's serialized camel-case fields is needed.

## Replacing your own formatting

Applications commonly strip the card's comma separators from names and group the
identifier for display. Those helpers can move to the SDK:

| Your existing helper | SDK accessor |
| --- | --- |
| Replacing commas in a name with spaces | `card.get_formatted_name()` |
| The same for one language | `card.get_formatted_name_in(Language::Arabic)` |
| Splitting a name on commas | `card.name_components_in(Language::English)` |
| Grouping the ID as `784-YYYY-NNNNNNN-C` | `card.formatted_id_number()` |
| Comparing the gender code against `"M"` | `card.gender()` |

These are additions. Existing getters, public fields, and serialized values are
unchanged, so an application can adopt them field by field. Gender *labels* stay
in the application: the SDK interprets the code into `Gender` but supplies no
`Male` or `ذكر` text.

## Behavior changes

- Enable `cli` to build/run the binary: `cargo run --features cli -- read`.
- CLI reads now redact by default. Explicitly add `--show-personal-data` for
  the previous personal-value output. `--redacted` remains supported.
- `reader_names()` returns an empty vector for the PC/SC no-readers condition.
- `connect_first()` preserves non-absence connection errors if none succeeds.
- Exhaustive `ErrorKind` matches must handle the new `InvalidArgument` variant.
- Corrupt image TLVs, non-JPEG photo payloads, impossible calendar dates,
  trailing TLV corruption, and duplicate requested fields now return errors.
- Directory probing only falls back for absent-file statuses. Repeated
  wrong-length responses no longer trigger repeated correction attempts.
- Core identity is decoded before optional image/extended transfers.

There is no SDK-level timeout or asynchronous API. Place synchronous reads
on a worker just as before. Repeat [hardware validation](Testing) for each
card generation and reader model used in your application.
