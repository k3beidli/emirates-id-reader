# Codes and identifiers

Identifiers and coded fields are returned as strings. That keeps leading zeroes
intact and stops a code from being mistaken for a quantity.

## Identifiers

| Method | Returns | Value |
| --- | --- | --- |
| `id_number()` | `&str` | Fifteen stored digits |
| `formatted_id_number()` | `String` | Digits grouped as `3-4-7-1` |
| `card_number()` | `&str` | Card serial stored by the ID application |

Both identifiers are required: a read fails rather than returning a snapshot
without them, so neither is optional.

`formatted_id_number()` applies the printed grouping only to exactly fifteen
ASCII digits, validated when a snapshot is read or built. Formatting does not
validate a check digit, infer a birth year, or prove
that the identifier was issued.

`id_type` is a separate document-type code on `identity()`. It is not the card
number or the chip-generation classification.

## Gender

The card stores a single-letter code. `gender()` interprets it; `gender_code()`
returns it verbatim.

```rust
use emirates_id_reader::Gender;

assert_eq!(Gender::from_code("M"), Gender::Male);
assert_eq!(Gender::from_code("f"), Gender::Female);   // matched case-insensitively
assert_eq!(Gender::Female.code(), "F");               // canonical uppercase code

// An unknown code is preserved, not discarded.
let other = Gender::from_code("X");
assert_eq!(other, Gender::Unrecognized("X".to_string()));
assert_eq!(other.code(), "X");
```

`gender()` returns `None` only when the field is absent. An unrecognized code
never collapses into `None`, so "the card said something we do not interpret"
stays distinguishable from "the card said nothing".

Applications choose display labels such as `Male` or `Ø°ÙƒØ±`. The library maps `M`
and `F` to enum variants without providing translations. The
[V1 specification](Sources#fields-stored-in-uae-id-card-v1) also lists `X`;
it remains `Unrecognized("X")` because this library does not assign it a meaning.
That variant means unsupported interpretation, not an invalid card value.

## Other coded fields

Occupation, occupation field, marital status, sponsor type, residency type,
passport type, passport country, qualification level, field of study, and
nationality all have code fields on [`identity()` or `extended()`](Field-Reference).
They are returned as stored strings, with leading zeroes preserved. The library
interprets no code but gender, and ships no lookup tables for the rest.

Where the card also stores a description for a code, both are available. For
example `occupation_code` sits alongside `occupation_english` and
`occupation_arabic`; `nationality_code` sits alongside `nationality_english`
and `nationality_arabic`. Descriptions come from the card, so their availability
varies by generation, and `nationality_in()` does not fall back between
languages the way the name accessors do.

## Related

- [Data model and formatting](Data-Model) for what the library does and does not
  interpret
- [Field reference](Field-Reference) for every code field and its type
- [V1/V2 compatibility](Card-Generations) for which descriptions each
  generation stores
