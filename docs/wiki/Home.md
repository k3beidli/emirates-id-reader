# Emirates ID Reader SDK

A Rust SDK for reading public identity fields, photographs, and signature
payloads from Emirates ID contact chips through native PC/SC.

Connect to a reader, read a snapshot into memory, then access it with
`get_formatted_name()`, `get_photo()`, and the other getters. V1 and V2 share
one data model; optional fields and group statuses describe what an individual
card exposes.

## Start here

- **New to the SDK?** [Installation and platforms](Platforms), then
  [your first read](Getting-Started).
- **Working with a value?** Start at [data model and formatting](Data-Model)
  and follow the topic guide for [names](Names),
  [codes and identifiers](Codes-And-Identifiers), [dates](Dates),
  [photos and signatures](Photos-And-Signatures), or
  [extended information](Extended-Information).
- **Looking something up?** The [API reference](API-Reference) indexes every
  method and the [field reference](Field-Reference) every field.
- **Something went wrong?** [Troubleshooting](Troubleshooting) for setup,
  [errors and read statuses](Error-Handling) for behavior at runtime.

The sidebar lists every page, grouped by purpose.

## Scope and status

Version 0.4.0 remains experimental. The native backend supports Windows, Linux,
and macOS contact PC/SC. Historical hardware results came with the imported
project; the refactor needs fresh validation. Unknown ATRs are probed, but
future generations are not guaranteed to work.

This SDK is independent and unofficial. It does not authenticate the card, read
fingerprints, or bypass protected files, and it contains no proprietary ICP
credentials or runtime components. See
[security and access boundaries](Security) and
[sources and acknowledgments](Sources).

This Wiki is generated from the repository's documentation; see
[documentation maintenance](Wiki-Setup).
