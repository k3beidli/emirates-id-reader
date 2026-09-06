# Emirates ID Reader Library

A Rust library for reading public identity fields, photographs, and signature
payloads from Emirates ID contact chips through native PC/SC.

Connect to a reader, read a snapshot into memory, then access it with
`formatted_name()`, `photo()`, and the other getters. V1 and V2 share
one data model; optional fields and group statuses describe what an individual
card exposes.

## Start here

- **New to the library?** [Installation and platforms](platforms.md), then
  [your first read](getting-started.md).
- **Working with a value?** Start at [data model and formatting](data-model.md)
  and follow the topic guide for [names](names.md),
  [codes and identifiers](codes-and-identifiers.md), [dates](dates.md),
  [photos and signatures](photos-and-signatures.md), or
  [extended information](extended-information.md).
- **Looking something up?** The [API reference](api-reference.md) indexes every
  method and the [field reference](field-reference.md) every field.
- **Something went wrong?** [Troubleshooting](troubleshooting.md) for setup,
  [errors and read statuses](error-handling.md) for behavior at runtime.

## Scope and status

Version 0.1.0 remains experimental. Hardware testing is limited to the
**HID OMNIKEY 3121 on Windows**. Linux and macOS are expected to work through
native PC/SC but have not been hardware-tested. No other reader models have
been tested. See [testing and hardware validation](testing.md) for the historical
results and the remaining checks for the current library.

**Fingerprint scanning has not been implemented yet.** Reading fingerprint
templates from the chip is also not implemented. Unknown ATRs are probed, but
future card generations are not guaranteed to work.

See [security and access boundaries](security.md) and
[sources and acknowledgments](sources.md).
