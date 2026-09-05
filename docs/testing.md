# Hardware validation

No personal values should be copied into issues, fixtures, screenshots, or
test logs. Use the CLI's redacted mode, which reports only generation, field
presence, character counts, image byte counts and group status.

```powershell
cargo run --release -- probe
cargo run --release -- read --redacted
```

## Per-generation checklist

Run the following on at least one card from each chip generation:

1. Confirm the ATR and detected `CardGeneration` agree with the ICP table.
2. Confirm application selection succeeds through Windows PC/SC.
3. Confirm ID number, card number and both full names are present.
4. Confirm all non-modifiable fields decode without invalid UTF-8 or BCD.
5. Confirm the modifiable group is either decoded or has an explicit status.
6. Confirm a returned photograph starts with the JPEG SOI marker and can be
   decoded by the consuming application.
7. Confirm unavailable/protected image groups do not fail the whole read.
8. Remove the card and confirm `CardSession::is_present()` becomes false.
9. Reinsert it and confirm a fresh session reads it once without stale data.
10. Run unit tests, Clippy, API docs, and `cargo package`.

## Evidence table

| Generation | Reader | Core identity | Extended data | Photo | Signature image | Removal/reinsert | Status |
| --- | --- | --- | --- | --- | --- | --- | --- |
| V1 | HID OMNIKEY 3x21 | Pass | Pass on tested card | Pass on tested card | Pass on tested card | Pending | Read path hardware validated |
| V2 | HID OMNIKEY 3x21 | Pass | Pass | Pass | Pass | Pass | Hardware validated |

The table records compatibility, not card authenticity. Offline genuineness or
digital-signature verification requires a separate trust policy and current
ICP signing certificates; this version of the SDK does not claim it.

The V1 result was obtained from a live card with ATR
`3B 6A 00 00 80 65 A2 01 31 01 3D 72 D6 41`. Both `identity_only()` and full
reads succeeded. The redacted full read returned all five file groups as
`Read`, a JPEG photograph, and a holder-signature payload. No cardholder field
values are retained in this repository.
