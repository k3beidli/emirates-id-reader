# Extended information

`extended()` holds the changeable and later-generation fields: employment,
family, passport, residency, and education. `session.read_all()` includes the group;
`ReadOptions::identity_only()` skips it. Enable it explicitly when using the
identity-only options:

```rust,no_run
use emirates_id_reader::{CardSession, ReadOptions};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let session = CardSession::connect_first()?;
    let card = session.read_with_options(
        ReadOptions::identity_only().with_modifiable_data(true),
    )?;

    let occupation = card.extended().occupation_english.as_deref();
    let passport = card.extended().passport_number.as_deref();
    let _ = (occupation, passport);
    Ok(())
}
```

`extended()` borrows `ModifiableData`; access individual fields directly.

## What the group contains

| Area | Fields |
| --- | --- |
| Employment | Occupation code, description, type, field code, company name |
| Family | Family identifier, marital status code, husband's ID, mother's name |
| Sponsorship | Sponsor type code, unified number, name |
| Residency | Residency type code, number, expiry date |
| Passport | Number, type code, country code and description, issue and expiry dates |
| Education | Qualification level, degree, field of study, place of study, graduation date |

Most descriptive text comes in both Arabic and English. Every field is
`Option<String>`; codes keep their leading zeroes. The
[field reference](field-reference.md) lists all of them with their exact names.

## Absent, blank, or never read

Check `read_status.modifiable` before interpreting the fields. If the group
was read, `None` means a field was absent or empty. Otherwise, its fields are
`None` because the library did not obtain them:

```rust,no_run
use emirates_id_reader::{CardSession, DataGroupStatus, ReadOptions};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let session = CardSession::connect_first()?;
    let card = session.read_with_options(
        ReadOptions::identity_only().with_modifiable_data(true),
    )?;

    match card.read_status().modifiable {
        DataGroupStatus::Read => { /* None means a field was absent or empty */ }
        DataGroupStatus::NotRequested => { /* the option was disabled */ }
        DataGroupStatus::NotAvailable => { /* this card has no such file */ }
        DataGroupStatus::Protected => { /* the file needs authentication */ }
        _ => { /* a status added in a future version */ }
    }
    Ok(())
}
```

Handled access-refusal and absent-file responses leave a snapshot with the
corresponding group status. Transport failures and malformed data still fail
the whole read. Core identity is always required.

## Generation differences

Most of this group is described by the V2 specification. Passport, education,
company name, and occupation type and field are not in the
published V1 field list, while occupation code, marital status, husband's ID,
sponsor, residency, and family identifier are in both.

Treat that as documentation, not as a runtime rule. A hardware-tested V1 card
returned more fields than the V1 list describes, which is why `CardGeneration`
and field availability stay independent. Inspect the values and the status; do
not branch on the generation label. See
[V1/V2 compatibility](card-generations.md) for the per-field table.

One field spans the difference: `mother_full_name_arabic` and
`mother_full_name_english` hold a first name on V1 and may hold a full name on
V2, through the same library fields.

## Related

- [Field reference](field-reference.md) for every field and Rust type
- [Codes and identifiers](codes-and-identifiers.md) for how coded values behave
- [Dates](dates.md) for the residency, passport, and graduation dates
- [Security and access boundaries](security.md) for the address, phone, and
  email fields this library never requests
