<a id="error-handling"></a>

# Errors and read statuses

A read returns either a complete `EmiratesIdData` snapshot or an `Error`.
Optional groups that were disabled, absent, or refused have a `read_status`
entry instead of a value. Transport failures and malformed data return `Err`;
the library does not return a partial snapshot for those failures.

## Error kinds

Use the error kind to choose a recovery action. Error messages are diagnostic
text and may change; do not parse them as a stable interface.

| `ErrorKind` | Meaning | Suggested application action |
| --- | --- | --- |
| `InvalidArgument` | Empty reader name or embedded NUL | Correct the selected reader name |
| `NoReader` | No PC/SC reader available | Ask the user to connect a reader |
| `NoCard` | No inserted card | Wait for insertion |
| `CardRemoved` | Card/reader disappeared or card reset | Clear stale data and reconnect |
| `Pcsc` | Native smart-card operation failed | Inspect driver/service/reader sharing; offer retry |
| `Protocol` | APDU rejected or invalid response | Inspect `status_word`; check card compatibility |
| `InvalidData` | Truncated or malformed file/field | Reject the result; report only redacted diagnostics |

```rust,no_run
use emirates_id_reader::{CardSession, ErrorKind};

fn main() {
    match CardSession::connect_first() {
        Ok(session) => {
            // Read on a worker thread in a GUI application.
            let _ = session;
        }
        Err(error) => match error.kind {
            ErrorKind::NoReader => eprintln!("Connect a smart-card reader."),
            ErrorKind::NoCard => eprintln!("Insert an Emirates ID."),
            ErrorKind::CardRemoved => eprintln!("Reconnect after card insertion."),
            _ => eprintln!("Reader operation failed: {error}"),
        },
    }
}
```

`connect_first()` skips empty readers, keeps trying the remaining readers after
a connection failure, and returns the first non-absence failure if none
succeeds. A stopped service or a sharing failure is never quietly converted into
`NoCard`.

<a id="optional-data-is-not-a-connection-error"></a>

## Read statuses

`read_status` carries one `DataGroupStatus` per group. It answers "was this
group readable", which is a different question from whether an individual field
holds a value.

| Status | Meaning |
| --- | --- |
| `Read` | The elementary file was read and decoded |
| `NotRequested` | The group was disabled through `ReadOptions` |
| `NotAvailable` | The card reports that the optional file does not exist |
| `Protected` | The card requires an authenticated or secure-messaging operation |

For the photo, extended data, and signature files, the card's own response
determines the status:

| Card status word | Library outcome |
| --- | --- |
| `6982`, `6985` | `DataGroupStatus::Protected` |
| `6A82`, `6A83` | `DataGroupStatus::NotAvailable` |
| Group disabled in `ReadOptions` | `DataGroupStatus::NotRequested` |
| Successful read and parse | `DataGroupStatus::Read` |

None of these fail the read. The required identifier and core identity files
must succeed; a transport failure or malformed content fails the read for any
group, and no partial snapshot is ever returned.

Retrying under the same access conditions does not unlock a `Protected` group.
The library does not perform authentication or establish secure messaging. Fields the library never requests at all are listed in the
[field reference](field-reference.md) and explained in
[security and access boundaries](security.md).

## Protocol recovery limits

The library follows `61xx` response continuation, permits one `6Cxx` length
correction per command, and caps a complete APDU exchange at 32 responses. It
accepts `6282` only for a `READ BINARY` operation, including its response
continuation; `SELECT` warnings remain errors. Public files are
bounded to 16 KiB including their four-byte header, and empty continuation
chunks are errors. Application-root fallback happens only when the public
directory is reported absent, never on a transport or security failure.

Do not build a tight retry loop. Allow user intervention or bounded application
backoff, and reconnect after a removal or reset. The library performs no automatic
reconnection and no authentication.

## Native diagnostics

`error.pcsc_code()` returns the original PC/SC code as `Option<u32>`.
`error.status_word` is reserved for ISO card responses. Match on `ErrorKind`
for recovery and use the numeric fields for diagnostics; messages are not stable.
`ErrorKind` is non-exhaustive, so include a fallback arm.
