# API reference

The crate root exports `CardSession`, `EmiratesIdData`, `NonModifiableData`,
`ModifiableData`, `ReadOptions`, `ReadStatus`, `DataGroupStatus`, `CardGeneration`,
`Language`, `Error`, `ErrorKind`, and `PROTECTED_AND_SKIPPED_FIELDS`.
All public items have Rustdoc documentation. Run `cargo doc --no-deps --open`
for signatures, field descriptions, and cross-references for your checkout.

## Session lifecycle

| Method | Returns | Behavior |
| --- | --- | --- |
| `CardSession::reader_names()` | `Result<Vec<String>, Error>` | Discover readers; no identity read |
| `CardSession::connect(name)` | `Result<CardSession, Error>` | Connect to an exact, nonempty reader name |
| `CardSession::connect_first()` | `Result<CardSession, Error>` | Connect to the first accessible reader containing a card |
| `session.reader_name()` | `&str` | Cached reader name |
| `session.atr()` | `&[u8]` | ATR captured at connection time |
| `session.atr_hex()` | `String` | Formatted ATR; allocates a new string |
| `session.card_generation()` | `CardGeneration` | `V1`, `V2`, or `Unknown`; not authentication |
| `session.is_present()` | `Result<bool, Error>` | Check presence without rereading identity |
| `session.read()` | `Result<EmiratesIdData, Error>` | Read every supported public group |
| `session.read_with_options(options)` | `Result<EmiratesIdData, Error>` | Read core identity and selected optional groups |

There is no manual `close()` requirement: dropping a session disconnects its
card, then releases the PC/SC context. Reconnect after card removal/reset.
Reads are synchronous, hold a PC/SC transaction, and are serialized for each
session. There is no SDK timeout, cancellation, or automatic retry facility.

## Snapshot accessors

Every accessor below reads the snapshot without chip I/O. The snapshot owns its
data and remains usable after the session is dropped. Its borrows cannot
outlive it. Accessors returning `&str`, `&[u8]`, or an iterator borrow and
allocate nothing; the formatting accessors in the next section build a new
`String`.

| Method | Return type | Meaning |
| --- | --- | --- |
| `get_name()` | `Option<&str>` | Stored English name, falling back to Arabic |
| `get_name_in(Language)` | `Option<&str>` | Stored name in the exact language; no fallback |
| `name_components_in(Language)` | `impl Iterator<Item = &str>` | Stored name components in card order |
| `get_photo()` | `Option<&[u8]>` | JPEG photograph payload |
| `get_signature()` | `Option<&[u8]>` | Signature-image payload; format may vary |
| `get_id_number()` | `&str` | Required 15-digit ID |
| `get_card_number()` | `&str` | Required 9-digit card number |
| `get_date_of_birth()` | `Option<&str>` | `YYYY-MM-DD` |
| `get_issue_date()` | `Option<&str>` | `YYYY-MM-DD` |
| `get_expiry_date()` | `Option<&str>` | `YYYY-MM-DD` |
| `get_gender()` | `Option<&str>` | Stored gender code |
| `get_nationality_code()` | `Option<&str>` | Stored nationality code |
| `get_nationality_in(Language)` | `Option<&str>` | Nationality description in the exact language |
| `identity()` | `&NonModifiableData` | Every core field |
| `extended()` | `&ModifiableData` | Every extended field |

`Language` is `English` or `Arabic`. The API uses Rust's `snake_case`, so
`get_name()` corresponds to the `getName()` style used in other languages.
No JavaScript or other language bindings are supplied.

## Formatting accessors

These are additive. They never change what the raw accessors, the public
fields, or serialization return; see [data model](data-model.md) for the rule
that decides what the SDK formats.

| Method | Return type | Meaning |
| --- | --- | --- |
| `get_formatted_name()` | `Option<String>` | Display name, English with Arabic fallback |
| `get_formatted_name_in(Language)` | `Option<String>` | Display name in the exact language |
| `formatted_id_number()` | `String` | ID grouped as `784-YYYY-NNNNNNN-C` |
| `gender()` | `Option<Gender>` | Interpreted gender code |

Name formatting replaces the card's comma separators with single spaces, drops
empty positions, and trims. Capitalization, spelling, diacritics, and component
order are preserved; the SDK does not identify which position holds a given
name or a family name. A field that is absent, or that holds only separators
and whitespace, yields `None`. Because a separator-only English field has no
formatted value, `get_formatted_name()` falls back to Arabic in a case where
`get_name()` would return the stored separators.

`formatted_id_number()` returns any value that is not exactly fifteen ASCII
digits unchanged. A read cannot produce one, but `id_number` is a public field
a caller can replace.

`Gender` is `Male`, `Female`, or `Unrecognized(String)`. `Gender::from_code()`
matches `M` and `F` case-insensitively, and `Gender::code()` returns the
canonical uppercase code printed on the card, or an unrecognized value exactly
as stored. An unknown code never becomes `None`. The SDK supplies no `Male` or
`ذكر` labels: those are translations rather than card data.

Public data fields remain available for compatibility. For example,
`card.extended().passport_number.as_deref()` accesses the optional passport
number and `card.identity().place_of_birth_arabic.as_deref()` accesses the
Arabic birthplace. See [data model](data-model.md) for the complete field list.

## Read options

| Method | Behavior |
| --- | --- |
| `ReadOptions::all()` / `default()` | Enable photo, extended data, and signature |
| `ReadOptions::identity_only()` | Disable all three optional groups |
| `.with_photo(bool)` | Toggle photo transfer |
| `.with_modifiable_data(bool)` | Toggle extended data transfer |
| `.with_holder_signature_image(bool)` | Toggle signature transfer |

Builders take and return `ReadOptions`, so they can be chained. Identifiers
and core identity are always read. The public Boolean fields remain usable
in struct literals. None of these options requests protected private data.

## Status and errors

`card.read_status` reports each group as `Read`, `NotRequested`,
`NotAvailable`, or `Protected`. `Read` means the file was obtained and parsed;
an individual field can still be blank. It is not proof of authenticity.
Malformed data and transport errors return `Err` for the whole read.

`Error` has public `kind`, `message`, and `status_word` fields and implements
`std::error::Error`, `Display`, and Serde `Serialize`. PC/SC numeric diagnostics
appear in the message; only ISO 7816 failures populate `status_word`.
Use `kind`/`status_word` for decisions, not message-string matching.
See [error handling](error-handling.md).

## Serialization

Data structs implement `Serialize` with camel-case field names. Generation,
status, and error-kind enums serialize as snake-case strings. Accessor names
do not change the serialized representation. If you add `serde_json` to your
application, byte vectors serialize as arrays of integers, not Base64 or data
URLs. The SDK does not include a JSON dependency, persistence API, or
deserialization API. Derived `Debug` output contains personal fields; do not
log the snapshot or its nested structs.
