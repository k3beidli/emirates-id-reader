# API reference

Look up the main session methods, data getters, and read options here. Each
section links to a guide with examples and explanations.

The crate root exports `CardSession`, `EmiratesIdData`, `EmiratesIdDataBuilder`, `NonModifiableData`,
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
| `session.read_identity()` | `Result<EmiratesIdData, Error>` | Read identifiers and core identity |
| `session.close()` | `Result<(), Error>` | Consume the session and report disconnect errors |
| `session.read_all()` | `Result<EmiratesIdData, Error>` | Read every supported public group |
| `session.read_with_options(options)` | `Result<EmiratesIdData, Error>` | Read core identity and selected optional groups |

Dropping a session disconnects its card and releases the PC/SC context; there is
also an explicit `close()` for observing disconnect errors. Reads hold a PC/SC transaction and are
serialized per session. There is no timeout, cancellation, or retry facility.
See [readers, sessions, and reading options](readers-and-sessions.md).

## Snapshot accessors

Every accessor reads the snapshot without chip I/O. The snapshot owns its data
and stays usable after the session is dropped, though its borrows cannot outlive
it. Accessors returning `&str`, `&[u8]`, or an iterator allocate nothing.

| Method | Return type | Meaning | Guide |
| --- | --- | --- | --- |
| `name()` | `Option<&str>` | Stored English name, falling back to Arabic | [Names](names.md) |
| `name_in(Language)` | `Option<&str>` | Stored name in one language; no fallback | [Names](names.md) |
| `name_components_in(Language)` | `impl Iterator<Item = &str>` | Stored components in card order | [Names](names.md) |
| `photo()` | `Option<&[u8]>` | JPEG photograph payload | [Photos](photos-and-signatures.md) |
| `signature()` | `Option<&[u8]>` | Signature payload; format may vary | [Photos](photos-and-signatures.md) |
| `id_number()` | `&str` | Required 15-digit ID | [Codes](codes-and-identifiers.md) |
| `card_number()` | `&str` | Required card number | [Codes](codes-and-identifiers.md) |
| `gender_code()` | `Option<&str>` | Stored gender code | [Codes](codes-and-identifiers.md) |
| `nationality_code()` | `Option<&str>` | Stored nationality code | [Codes](codes-and-identifiers.md) |
| `nationality_in(Language)` | `Option<&str>` | Nationality description; no fallback | [Data model](data-model.md) |
| `date_of_birth()` | `Option<&str>` | `YYYY-MM-DD` | [Dates](dates.md) |
| `issue_date()` | `Option<&str>` | `YYYY-MM-DD` | [Dates](dates.md) |
| `expiry_date()` | `Option<&str>` | `YYYY-MM-DD` | [Dates](dates.md) |
| `identity()` | `&NonModifiableData` | Every core field | [Field reference](field-reference.md) |
| `extended()` | `&ModifiableData` | Every extended field | [Extended](extended-information.md) |

`Language` is `English` or `Arabic`.

## Formatting accessors

Name and ID formatters return owned strings. Gender interpretation allocates
only when preserving an unrecognized code; `Gender::code()` borrows its result.
These methods leave stored values, raw getters, and serialization
unchanged. See
[data model and formatting](data-model.md) for the policy.

| Method | Return type | Meaning | Guide |
| --- | --- | --- | --- |
| `formatted_name()` | `Option<String>` | Display name, English with Arabic fallback | [Names](names.md) |
| `formatted_name_in(Language)` | `Option<String>` | Display name in one language | [Names](names.md) |
| `formatted_id_number()` | `String` | ID grouped as `784-YYYY-NNNNNNN-C` | [Codes](codes-and-identifiers.md) |
| `gender()` | `Option<Gender>` | Interpreted gender code | [Codes](codes-and-identifiers.md) |
| `Gender::from_code(&str)` | `Gender` | `Male`, `Female`, or `Unrecognized` | [Codes](codes-and-identifiers.md) |
| `Gender::code()` | `&str` | `M`, `F`, or the preserved unrecognized code | [Codes](codes-and-identifiers.md) |

Nested record fields remain accessible through read-only borrows. For example
`card.extended().passport_number.as_deref()` and
`card.identity().place_of_birth_arabic.as_deref()` reach fields that have no
accessor; the [field reference](field-reference.md) lists them all.

## Read options

| Method | Behavior |
| --- | --- |
| `ReadOptions::all()` | Enable photo, extended data, and signature |
| `ReadOptions::default()` | Identifiers and core identity only |
| `ReadOptions::identity_only()` | Disable all three optional groups |
| `.with_photo(bool)` | Toggle photo transfer |
| `.with_modifiable_data(bool)` | Toggle extended data transfer |
| `.with_holder_signature_image(bool)` | Toggle signature transfer |

Builders take and return `ReadOptions`, so they chain. Identifiers and core
identity are always read. Use constructors and builders; `ReadOptions` is
non-exhaustive. The options select supported public groups. An individual card
may still refuse access to one of those groups. See
[readers, sessions, and reading options](readers-and-sessions.md).

## Status and errors

`card.read_status()` reports each group as `Read`, `NotRequested`, `NotAvailable`,
or `Protected`. `Read` means the file was obtained and parsed; an individual
field can still be blank, and it is not proof of authenticity. Malformed data
and transport errors return `Err` for the whole read.

`Error` has public `kind`, `message`, and `status_word` fields and implements
`std::error::Error` and `Display`, plus `Serialize` with the `serde` feature. PC/SC numeric diagnostics
appear in the message; only ISO 7816 failures populate `status_word`. Use `kind`
and `status_word` for decisions, never message-string matching. Native codes are
available through `error.pcsc_code()`. See
[errors and read statuses](error-handling.md).

## Serialization

Enable the optional `serde` feature to serialize data structs with camel-case field names. Generation,
status, and error-kind variants use snake-case names. With `serde_json`, a
separately serialized `Gender::Male` becomes `"male"`, while
`Gender::Unrecognized("X".into())` becomes `{"unrecognized":"X"}`.
The snapshot still contains the original gender-code string: calling `gender()`
does not replace it or add a serialized field. Accessor names do not change the
serialized representation. If you add `serde_json`,
byte vectors serialize as arrays of integers, not Base64 or data URLs. The library
includes no runtime JSON dependency, persistence API, or deserialization API.
`Debug` redacts personal fields on snapshots and nested records. Serialization
still includes personal values and must not be used for routine logging.

## Constructing and extending data

`EmiratesIdData` has private fields. Use `reader_name()`, `card_generation()`,
`read_status()`, `identity()`, and `extended()` to inspect metadata and records.
For synthetic snapshots, use `EmiratesIdData::builder(id, card_number)`; see
[the runnable example](names.md#formatting-without-a-card).
The builder supports `reader_name`, `card_generation`, `identity`, `extended`,
`photo`, `signature`, and `optional_statuses`, followed by `build()`.
Identifiers, supplied calendar dates, and the JPEG prefix are validated.
Values are caller-supplied, not authenticated hardware results.

`CardGeneration`, `DataGroupStatus`, `ErrorKind`, and `Gender` are non-exhaustive;
include a fallback arm in matches. `ReadOptions`, `ReadStatus`, and the identity
record types are also non-exhaustive. Create records with `Default` and assign
fields instead of constructing struct literals.
