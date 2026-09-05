# Emirates ID Reader SDK

A Rust SDK for reading public identity fields, photographs, and signature
payloads from Emirates ID contact chips through Windows PC/SC.

The public API is designed around connecting once, reading an owned snapshot,
and accessing values with `get_name()`, `get_photo()`, and other borrowed
getters. V1 and V2 use the same data model; optional fields and group statuses
describe what the individual card exposes.

## Start here

- [Getting started](Getting-Started): prerequisites, installation, first read.
- [API reference](API-Reference): session methods, getters, options, serialization.
- [Field reference](Field-Reference): every decoded public field and its type.
- [Application integration](Application-Integration): UI workers, images, removal, ownership.
- [Error handling](Error-Handling): statuses and recovery behavior.
- [Card generations](Card-Generations): V1/V2 classification and field availability.
- [Troubleshooting](Troubleshooting): setup and common failures.
- [Security](Security): data handling and authenticity boundaries.
- [Testing](Testing): automated coverage and hardware validation status.
- [Architecture](Architecture): source layout and protocol design.
- [Migration](Migration): changes from the original extracted code.
- [Contributing](Contributing): checks and release preparation.

## Scope and status

Version 0.3.0 remains experimental. The current backend supports Windows
contact PC/SC only. Historical hardware results came with the imported
project; the refactor requires fresh hardware validation. Unknown ATRs are
probed, but future generations are not guaranteed to work.

This SDK is independent and unofficial. It does not authenticate the card,
read fingerprints, or bypass protected files. It contains no proprietary ICP
credentials or runtime components.

This Wiki is generated from the repository's documentation. See
[Wiki setup](Wiki-Setup) for regeneration and publication instructions.
