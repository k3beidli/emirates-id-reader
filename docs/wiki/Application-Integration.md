# Application integration

## Read once, display many times

Use this lifecycle for a desktop UI, kiosk, or local service:

1. Discover and select a reader, then connect when a card is inserted.
2. Read the fields that operation needs into one `EmiratesIdData`.
3. Use formatted getters for display, or borrow the original values. Copy
   borrowed data only when the UI framework needs to own it.
4. Poll `is_present()` at a modest interval, for example 200 milliseconds.
5. On removal, clear the UI and any application copies, drop the snapshot and
   session, and reconnect after reinsertion.

`read()` performs a fresh read; getters use values already in the snapshot. A snapshot does not update itself when the card is removed or replaced,
and dropping it frees allocations without guaranteeing memory zeroization.

The runnable `watch_removal` example illustrates presence polling. A production
UI should also clear state when a presence check or read fails, and provide its
own stop control for the worker.

<a id="blocking-and-concurrency"></a>

## Threading

PC/SC calls are synchronous and may block. Run them on a dedicated worker or
your async runtime's blocking executor, never on a GUI event thread. A timeout
around an async wrapper does not cancel the underlying native call; the SDK has
no cancellation API.

Reads on one session are already serialized for you, and a card reset or removal
needs a new session rather than retries on the old handle. See
[readers, sessions, and reading options](Readers-And-Sessions) for the
connection lifetime and transaction details.

<a id="photos-and-signatures"></a>

## Displaying images

`get_photo()` and `get_signature()` hand you borrowed bytes, so copy only if
your image widget needs ownership. The SDK validates the JPEG prefix and TLV
structure, not decodability, so your decoder must handle failure, and signature
payloads carry no asserted MIME type. Neither accessor writes a file: storing or
transmitting an image is always an explicit application operation. See
[photos and signatures](Photos-And-Signatures).

<a id="extended-fields-and-missing-values"></a>

## Missing values

An absent field is `None`; do not treat it as a verified empty value. Whether
the containing optional group was requested and read is recorded separately in
`read_status`. Applying your own formatting is fine, but keep the original value
if your workflow may need it later. See
[data model and formatting](Data-Model) and
[extended information](Extended-Information).

## Integration boundaries

This is a Rust library, usable directly from Rust applications or the Rust
backend of a desktop app. It exposes no network service, C ABI, .NET assembly,
JavaScript package, or proprietary ICP SDK compatibility layer. Authentication,
fingerprint reads, card genuineness, signature verification, and contactless
access are outside this implementation.

See [security and access boundaries](Security) and
[errors and read statuses](Error-Handling).
