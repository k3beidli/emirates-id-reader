# Application integration

## Read once, display many times

Use this lifecycle for a desktop UI, kiosk, or local service:

1. Discover/select a reader and connect when a card is inserted.
2. Read the fields needed for that operation into one `EmiratesIdData`.
3. Bind borrowed getters to your UI, or make deliberate owned copies when
   your UI framework requires ownership.
4. Poll `is_present()` at a modest interval, for example 200 milliseconds.
5. On removal, clear the UI and application copies, drop the snapshot and
   session, and reconnect after reinsertion.

`read()` always performs a fresh read. `get_name()`, `get_photo()`, and other
snapshot accessors use already-read data. A snapshot does not automatically
update when a card is removed or replaced. Dropping it frees allocations but
does not guarantee cryptographic memory zeroization.

The runnable `watch_removal` example illustrates presence polling. A
production UI should also clear state when presence checks or reads fail,
and provide its own stop/shutdown control for the worker.

## Blocking and concurrency

PC/SC calls are synchronous and may block, including transaction acquisition.
Run them on a dedicated worker or your async runtime's blocking executor.
Do not execute reads on a GUI event thread. A timeout around an async wrapper
does not cancel the underlying Windows call; the SDK has no cancellation API.

Reads sharing one session are protected by a mutex and a PC/SC transaction.
The mutex prevents concurrent callers from interleaving selections on the
same handle. The transaction coordinates with other PC/SC connections.
Prefer one owner for a reader's lifecycle; a card reset or removal requires
a new session rather than repeated retries on the old handle.

## Photos and signatures

Request a photograph without other expensive groups:

```rust,no_run
use emirates_id_reader::{CardSession, ReadOptions};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let session = CardSession::connect_first()?;
    let card = session.read_with_options(ReadOptions::identity_only().with_photo(true))?;
    if let Some(jpeg) = card.get_photo() {
        // Pass jpeg to your UI image decoder; copy only if the UI needs ownership.
        let _ = jpeg;
    }
    Ok(())
}
```

The SDK checks the JPEG prefix and TLV structure, not full image decodability.
Your image decoder must validate dimensions/content and handle decoding
failure. Signature payloads are returned without an asserted MIME type.
Neither accessor writes files. Store or transmit data only as an explicit
application operation.

## Extended fields and missing values

Occupation, residency, passport, education, family, and mother-name fields
are in `card.extended()`. Enable `with_modifiable_data(true)` or use `read()`.
An absent field is `None`; do not treat it as an empty verified value. Inspect
`read_status.modifiable` to determine whether the group was requested/read.

Dates are validated calendar strings. Codes stay strings to preserve leading
zeroes. Names and codes are not normalized into application enums. Perform
application-specific formatting separately and preserve the original value
if your workflow needs it.

## Integration boundaries

This is a Rust library, usable directly from Rust applications or the Rust
backend of a desktop app. It does not expose a network service, C ABI, .NET
assembly, JavaScript package, or proprietary ICP SDK compatibility layer.
Authentication, fingerprint reads, card genuineness, signature verification,
and contactless access are outside this implementation.

See [security](Security) and [error handling](Error-Handling).
