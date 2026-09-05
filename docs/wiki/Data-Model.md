# SDK data model

See the [complete field reference](Field-Reference) for every public field,
Rust type, and description.

`EmiratesIdData` is generation-neutral. A V1 caller and a V2 caller use the
same Rust types; fields that do not exist on a generation or are blank on a
particular card remain `None`.

## Top-level fields

| Rust field | Meaning |
| --- | --- |
| `reader_name` | PC/SC reader that supplied this card |
| `card_generation` | `V1`, `V2`, or `Unknown`, based on the published ATR list |
| `id_number` | Required 15-character Emirates ID number |
| `card_number` | Required card serial/number stored by the ID applet |
| `photo_jpeg` | JPEG bytes when requested and publicly readable |
| `holder_signature_image` | Signature-image payload when requested and available |
| `non_modifiable` | Core identity fields |
| `modifiable` | Occupation, residency, passport and education fields |
| `read_status` | Per-group access result |

## Empty values and inaccessible groups

An individual `Option<String>` being `None` means that field was empty or not
present in a successfully decoded group. Consult `read_status` to understand
whether the containing optional group was read:

| Status | Meaning |
| --- | --- |
| `Read` | The elementary file was read and decoded |
| `NotRequested` | Disabled through `ReadOptions` |
| `NotAvailable` | The card reports that the optional file does not exist |
| `Protected` | The card requires an authenticated/secure-messaging operation |

Dates are returned as `YYYY-MM-DD`. Numeric BCD codes are returned as strings
so leading zeroes are preserved and callers do not accidentally treat codes as
quantities.

## Fast identity-only reads

```rust,no_run
use emirates_id_reader::{CardSession, ReadOptions};

fn main() -> Result<(), emirates_id_reader::Error> {
let session = CardSession::connect_first()?;
let card = session.read_with_options(ReadOptions::identity_only())?;

println!("{:?}", card.card_generation);
println!("{}", card.id_number);
println!(
    "{}",
    card.non_modifiable.full_name_english.as_deref().unwrap_or("")
);
Ok(())
}
```

Use `session.read()` when photographs and all optional public groups are
needed. Large binary values are owned byte vectors and remain only in memory
unless the calling application explicitly persists them.
