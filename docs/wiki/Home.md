# Emirates ID Reader Library

A Rust library for reading public identity fields, photographs, and signature
payloads from Emirates ID contact chips through native PC/SC.

Connect to a reader, read a snapshot into memory, then access it with
`formatted_name()`, `photo()`, and the other getters. V1 and V2 share
one data model; optional fields and group statuses describe what an individual
card exposes.

## Start here

- **New to the library?** [Installation and platforms](Platforms), then
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

## Scope and status

Version 0.1.0 remains experimental. Hardware testing is limited to the
**HID OMNIKEY 3121 on Windows**. Linux and macOS are expected to work through
native PC/SC but have not been hardware-tested. No other reader models have
been tested. See [testing and hardware validation](Testing) for the historical
results and the remaining checks for the current library.

**Fingerprint scanning has not been implemented yet.** Reading fingerprint
templates from the chip is also not implemented. Unknown ATRs are probed, but
future card generations are not guaranteed to work.

See [security and access boundaries](Security) and
[sources and acknowledgments](Sources).
