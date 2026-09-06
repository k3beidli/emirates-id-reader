# API reference

Look up the main session methods, data getters, and read options here. Each
section links to a guide with examples and explanations.

The crate root exports `CardSession`, `EmiratesIdData`, `NonModifiableData`,
`ModifiableData`, `ReadOptions`, `ReadStatus`, `DataGroupStatus`,
`CardGeneration`, `Gender`, `Language`, `Error`, `ErrorKind`, and
`PROTECTED_AND_SKIPPED_FIELDS`. All public items have Rustdoc. Run
`cargo doc --no-deps --open` for signatures and cross-references matching your
checkout.

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

Dropping a session disconnects its card and releases the PC/SC context; there is
no `close()`. Reads are synchronous, hold a PC/SC transaction, and are
serialized per session. There is no timeout, cancellation, or retry facility.
See [readers, sessions, and reading options](readers-and-sessions.md).

## Snapshot accessors

Every accessor reads the snapshot without chip I/O. The snapshot owns its data
and stays usable after the session is dropped, though its borrows cannot outlive
it. Accessors returning `&str`, `&[u8]`, or an iterator allocate nothing.

| Method | Return type | Meaning | Guide |
| --- | --- | --- | --- |
| `get_name()` | `Option<&str>` | Stored English name, falling back to Arabic | [Names](names.md) |
| `get_name_in(Language)` | `Option<&str>` | Stored name in one language; no fallback | [Names](names.md) |
| `name_components_in(Language)` | `impl Iterator<Item = &str>` | Stored components in card order | [Names](names.md) |
| `get_photo()` | `Option<&[u8]>` | JPEG photograph payload | [Photos](photos-and-signatures.md) |
| `get_signature()` | `Option<&[u8]>` | Signature payload; format may vary | [Photos](photos-and-signatures.md) |
| `get_id_number()` | `&str` | Required 15-digit ID | [Codes](codes-and-identifiers.md) |
| `get_card_number()` | `&str` | Required card number | [Codes](codes-and-identifiers.md) |
| `get_gender()` | `Option<&str>` | Stored gender code | [Codes](codes-and-identifiers.md) |
| `get_nationality_code()` | `Option<&str>` | Stored nationality code | [Codes](codes-and-identifiers.md) |
| `get_nationality_in(Language)` | `Option<&str>` | Nationality description; no fallback | [Data model](data-model.md) |
| `get_date_of_birth()` | `Option<&str>` | `YYYY-MM-DD` | [Dates](dates.md) |
| `get_issue_date()` | `Option<&str>` | `YYYY-MM-DD` | [Dates](dates.md) |
| `get_expiry_date()` | `Option<&str>` | `YYYY-MM-DD` | [Dates](dates.md) |
| `identity()` | `&NonModifiableData` | Every core field | [Field reference](field-reference.md) |
| `extended()` | `&ModifiableData` | Every extended field | [Extended](extended-information.md) |

`Language` is `English` or `Arabic`. The API uses Rust's `snake_case`, so
`get_name()` corresponds to the `getName()` style used elsewhere. No JavaScript
or other language bindings are supplied.

## Formatting accessors

Name and ID formatters return owned strings. Gender interpretation allocates
only when preserving an unrecognized code; `Gender::code()` borrows its result.
These methods leave the original getters, public fields, and serialization
unchanged. See
[data model and formatting](data-model.md) for the policy.

| Method | Return type | Meaning | Guide |
| --- | --- | --- | --- |
| `get_formatted_name()` | `Option<String>` | Display name, English with Arabic fallback | [Names](names.md) |
| `get_formatted_name_in(Language)` | `Option<String>` | Display name in one language | [Names](names.md) |
| `formatted_id_number()` | `String` | ID grouped as `784-YYYY-NNNNNNN-C` | [Codes](codes-and-identifiers.md) |
| `gender()` | `Option<Gender>` | Interpreted gender code | [Codes](codes-and-identifiers.md) |
| `Gender::from_code(&str)` | `Gender` | `Male`, `Female`, or `Unrecognized` | [Codes](codes-and-identifiers.md) |
| `Gender::code()` | `&str` | `M`, `F`, or the preserved unrecognized code | [Codes](codes-and-identifiers.md) |

Public data fields remain available for compatibility. For example
`card.extended().passport_number.as_deref()` and
`card.identity().place_of_birth_arabic.as_deref()` reach fields that have no
accessor; the [field reference](field-reference.md) lists them all.

## Read options

| Method | Behavior |
| --- | --- |
| `ReadOptions::all()` / `default()` | Enable photo, extended data, and signature |
| `ReadOptions::identity_only()` | Disable all three optional groups |
| `.with_photo(bool)` | Toggle photo transfer |
| `.with_modifiable_data(bool)` | Toggle extended data transfer |
| `.with_holder_signature_image(bool)` | Toggle signature transfer |

Builders take and return `ReadOptions`, so they chain. Identifiers and core
identity are always read. The public Boolean fields remain usable in struct
literals. The options select only the SDK's supported public groups. An individual card
may still refuse access to one of those groups. See
[readers, sessions, and reading options](readers-and-sessions.md).

## Status and errors

`card.read_status` reports each group as `Read`, `NotRequested`, `NotAvailable`,
or `Protected`. `Read` means the file was obtained and parsed; an individual
field can still be blank, and it is not proof of authenticity. Malformed data
and transport errors return `Err` for the whole read.

`Error` has public `kind`, `message`, and `status_word` fields and implements
`std::error::Error`, `Display`, and Serde `Serialize`. PC/SC numeric diagnostics
appear in the message; only ISO 7816 failures populate `status_word`. Use `kind`
and `status_word` for decisions, never message-string matching. See
[errors and read statuses](error-handling.md).

## Serialization

Data structs implement `Serialize` with camel-case field names. Generation,
status, and error-kind variants use snake-case names. With `serde_json`, a
separately serialized `Gender::Male` becomes `"male"`, while
`Gender::Unrecognized("X".into())` becomes `{"unrecognized":"X"}`.
The snapshot still contains the original gender-code string: calling `gender()`
does not replace it or add a serialized field. Accessor names do not change the
serialized representation. If you add `serde_json`,
byte vectors serialize as arrays of integers, not Base64 or data URLs. The SDK
includes no JSON dependency, persistence API, or deserialization API. Derived
`Debug` output contains personal fields; do not log the snapshot or its nested
structs.
