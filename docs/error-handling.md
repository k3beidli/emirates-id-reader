# Error handling

Use the error kind to choose a recovery action. Error messages are diagnostic
text and may change; do not parse them as a stable interface.

| `ErrorKind` | Meaning | Suggested application action |
| --- | --- | --- |
| `InvalidArgument` | Empty reader name or embedded NUL | Correct the selected reader name |
| `NoReader` | No PC/SC reader available | Ask the user to connect a reader |
| `NoCard` | No inserted card | Wait for insertion |
| `CardRemoved` | Card/reader disappeared or card reset | Clear stale data and reconnect |
| `Pcsc` | Windows smart-card operation failed | Inspect driver/service/reader sharing; offer retry |
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

`connect_first()` skips empty readers, continues trying remaining readers
after connection failures, and returns the first non-absence failure if no
connection succeeds. A stopped service or sharing failure is not silently
converted into `NoCard`.

## Optional data is not a connection error

For photo, modifiable data, and signature files:

| Card status | SDK outcome |
| --- | --- |
| `6982`, `6985` | `DataGroupStatus::Protected` |
| `6A82`, `6A83` | `DataGroupStatus::NotAvailable` |
| Group disabled | `DataGroupStatus::NotRequested` |
| Successful read/parse | `DataGroupStatus::Read` |

Those statuses do not fail the entire read for optional groups. The required
identifier/core files must succeed. Transport failure or malformed content
fails the read regardless of group; no partial snapshot is returned.

## Protocol recovery limits

The SDK follows `61xx` response continuation, permits one `6Cxx` length
correction per command, and caps a complete APDU exchange at 32 responses.
It accepts data returned with the `6282` end-of-file warning. Public files are
bounded to 16 KiB, including their four-byte header; empty continuation chunks
are errors. Application-root fallback occurs only when the public directory
is reported absent, not on transport/security failures.

Do not build a tight retry loop. Allow user intervention or bounded application
backoff, and reconnect after removal/reset. The SDK performs no automatic
reconnection or authentication.
