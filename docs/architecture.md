# Architecture

The SDK separates the public lifecycle from platform resources and card
decoding. Its core read flow can be exercised with a synthetic command
transport, without a connected reader.

```text
Application
  -> CardSession (session.rs): lifecycle, read mutex, transaction boundary
       -> Connection (transport.rs): Windows handles and PC/SC FFI
       -> Reader (protocol.rs): application/files, chunking, optional groups
            -> exchange_apdu (apdu.rs): continuation and length correction
            -> decode.rs: TLV, UTF-8, BCD, calendar dates
       -> EmiratesIdData (data.rs): owned snapshot and borrowed accessors
```

`lib.rs` exports the consumer API. Internal modules are private so applications
do not depend on raw handles, APDU details, or implementation-specific file
layouts. `error.rs` defines shared structured errors. No app-specific state,
Tauri commands, HTTP endpoints, or persistence is part of the SDK.

## Resource ownership

A connection owns a card and its PC/SC context. Rust field drop order releases
the card before its parent context. A transaction guard ends the transaction
on normal return, error return, or unwinding. The session mutex covers the
whole read because native transactions alone do not prevent same-handle
callers from interleaving commands. A poisoned mutex returns an error asking
the caller to reconnect.

## Read sequence

1. Acquire the session mutex and begin a PC/SC transaction.
2. Select the Emirates ID AID.
3. Select public directory `0200`; if absent, reselect the application root.
4. Read `0201` identifiers and `0203` core identity, validating required data.
5. Read requested `0202` photo, `0205` modifiable data, and `0207` signature.
6. Decode and return an owned snapshot with per-group status.
7. Release transaction and mutex on return.

The ATR classifies V1/V2; it does not determine optional field access. Unknown
ATRs use the same application/file probes. This preserves one data model
without claiming support for future generations.

## Defensive decoding

The card format uses two-byte tags and two-byte lengths inside a four-byte
container header. Parsing checks field boundaries throughout the container,
including after a requested field, and rejects duplicates of that requested
tag. Unknown fields are skipped after validating their boundaries.
Dates require valid BCD and a calendar-valid day/month/year. Required
identifiers require fixed-length ASCII digits.

Image TLV errors propagate. Nonempty photo payloads must have a JPEG prefix;
full image validation remains the consumer's responsibility. Signature bytes
are opaque. Files remain bounded; no parser reads private or authenticated
files as a fallback.

## Tests and extension points

`tests.rs` covers inherited parser/APDU cases. `sdk_tests.rs` emulates a card's
SELECT and READ BINARY behavior to test options, statuses, image chunking,
fallback, and malformed data. These tests do not establish hardware support.

The private protocol reader accepts a command callback, isolating it from
Windows. A future platform backend should preserve the public lifecycle,
transaction guarantees, and error semantics. Platform portability or a
public raw-transport API is not promised by this release.
