# Readers, sessions, and reading options

A session owns one connection to one reader. Connect, read a snapshot, then use
the snapshot for as long as you need it: accessors never touch the chip again.

```rust,no_run
use emirates_id_reader::{CardSession, ReadOptions};

fn main() -> Result<(), emirates_id_reader::Error> {
    let session = CardSession::connect_first()?;
    let card = session.read_with_options(ReadOptions::identity_only())?;

    let name = card.get_formatted_name();
    // Pass the value to your UI without logging personal data.
    let _ = name;
    Ok(())
}
```

## Choosing a reader

`CardSession::reader_names()` lists readers reported by the local PC/SC service
without reading identity fields. Present them, then pass the exact selected name to
`CardSession::connect(&name)`.

An empty list means PC/SC currently reports no readers. A stopped or unavailable PC/SC
service is a different condition and returns an error instead.

`connect_first()` takes the first accessible reader holding any card, and
verifies the Emirates ID application when you read. It does not search every
reader for a particular card or cardholder, so prefer explicit selection
whenever more than one reader may be attached. If no connection succeeds, it
returns the first non-absence failure rather than reporting "no card".

## Reading options

`ReadOptions` controls the expensive optional groups. Core identity is always
read and always required.

| Constructor or builder | Effect |
| --- | --- |
| `ReadOptions::all()`, `ReadOptions::default()` | Every supported public group |
| `ReadOptions::identity_only()` | Identifiers and core identity only |
| `.with_photo(bool)` | Photograph |
| `.with_modifiable_data(bool)` | Occupation, residency, passport, education |
| `.with_holder_signature_image(bool)` | V2 signature image |

`session.read()` is `read_with_options(ReadOptions::all())`. The builders are
`const` and chainable, so `identity_only().with_photo(true)` is the usual way to
add one group to a fast read.

A disabled group has status `DataGroupStatus::NotRequested`. Check that status
before interpreting a missing field; even a successfully read group can have
blank or absent fields.

## Session lifetime

| Method | Returns | Behavior |
| --- | --- | --- |
| `reader_name()` | `&str` | Cached name |
| `atr()`, `atr_hex()` | `&[u8]`, `String` | ATR captured at connection |
| `card_generation()` | `CardGeneration` | Classified from the ATR; not authentication |
| `is_present()` | `Result<bool, Error>` | Presence check without rereading |
| `read()`, `read_with_options()` | `Result<EmiratesIdData, Error>` | Fresh read |

There is no `close()`. Dropping a session disconnects the card and releases the
PC/SC context. After a removal or a card reset, build a new session rather than
retrying on the old handle.

A snapshot owns its data and stays usable after its session is dropped, but it
never updates itself: it contains the values returned by that read.

## Blocking and serialization

Every PC/SC call is synchronous and may block, including acquiring a
transaction. Run reads on a dedicated worker or a blocking executor, never on a
UI thread. There is no timeout, cancellation, or automatic retry in the SDK, and
wrapping a read in an async timeout does not cancel the underlying native call.

Reads on one session are serialized by a mutex and wrapped in a PC/SC
transaction. The mutex stops concurrent callers interleaving selections on the
same handle; the transaction coordinates with other PC/SC connections. Prefer a
single owner for a reader's lifecycle. A poisoned mutex returns an error asking
you to reconnect.

## Related

- [Your first read](Getting-Started) for the end-to-end walkthrough
- [Application integration](Application-Integration) for presence polling and UI wiring
- [Errors and read statuses](Error-Handling) for recovery behavior
- [API reference](API-Reference) for the complete method index
