# Dates

Date fields use `Option<String>` with values in `YYYY-MM-DD` form. The three
date getters borrow those strings as `Option<&str>`. Dates contain no time of
day or timezone.

During a card read, the decoder validates packed binary-coded decimal (BCD)
values and rejects impossible calendar dates. The snapshot builder also checks
supplied ISO calendar dates. Standalone identity records remain editable.

```rust,no_run
use emirates_id_reader::{CardSession, ReadOptions};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let session = CardSession::connect_first()?;
    let card = session.read_with_options(ReadOptions::identity_only())?;

    let birth = card.date_of_birth();
    let expiry = card.expiry_date();
    let _ = (birth, expiry);
    Ok(())
}
```

## Available dates

| Date | Getter or field |
| --- | --- |
| Date of birth | `date_of_birth()` |
| Card issue date | `issue_date()` |
| Card expiry date | `expiry_date()` |
| Residency expiry | `extended().residency_expiry_date` |
| Passport issue and expiry | `extended().passport_issue_date`, `extended().passport_expiry_date` |
| Graduation | `extended().date_of_graduation` |

Extended dates require the modifiable group. `session.read_all()` includes it;
`ReadOptions::identity_only()` does not. See
[extended information](Extended-Information).

If the group status is `Read`, `None` means the date was absent or decoded as
empty. Otherwise, check the group status to see why it was not obtained.

## Display formatting

Choose regional formats in your application, keeping the original ISO date
available for storage or exchange. For example, the same date could appear as
`12 March 1985` or `12/03/1985`. The library does not select a locale or calendar
display format.

## Calculating age

The library has no age field or age getter. Calculate age in your application from
the birthdate and an explicit reference date, such as an appointment date or
today's date in the timezone your application uses.

For completed years:

1. Parse and validate both dates using your application's date library.
2. Reject a reference date earlier than the birthdate.
3. Subtract the birth year from the reference year.
4. Subtract one if the birthday has not yet occurred in the reference year.

For example, someone born on `1985-03-12` is 40 on `2026-03-11` and 41 on
`2026-03-12`. Define how your application treats a February 29 birthday in a
non-leap year; do not leave that choice implicit in an eligibility check.

## Checking expiry

The library returns the stored expiry date without deciding whether a card is
currently valid. Your application chooses the reference date and whether the
expiry date itself is included. A future expiry date does not prove authenticity
or current administrative status.

## Related

- [Data model and formatting](Data-Model) for shared value conventions
- [Field reference](Field-Reference) for the exact fields and types
- [Architecture and chip communication](Architecture) for date decoding
