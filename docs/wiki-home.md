# Emirates ID Reader SDK

A Rust SDK for reading public identity fields, photographs, and signature
payloads from Emirates ID contact chips through native PC/SC.

The public API is designed around connecting once, reading an owned snapshot,
and accessing values with `get_name()`, `get_photo()`, and other borrowed
getters. V1 and V2 use the same data model; optional fields and group statuses
describe what the individual card exposes.

## Start here

- [Getting started](getting-started.md): prerequisites, installation, first read.
- [API reference](api-reference.md): session methods, getters, options, serialization.
- [Field reference](field-reference.md): every decoded public field and its type.
- [Application integration](integration.md): UI workers, images, removal, ownership.
- [Error handling](error-handling.md): statuses and recovery behavior.
- [Card generations](card-generations.md): V1/V2 classification and field availability.
- [Troubleshooting](troubleshooting.md): setup and common failures.
- [Security](security.md): data handling and authenticity boundaries.
- [Testing](testing.md): automated coverage and hardware validation status.
- [Architecture](architecture.md): source layout and protocol design.
- [Migration](migration.md): changes from the original extracted code.
- [Contributing](../CONTRIBUTING.md): checks and release preparation.

## Scope and status

Version 0.4.0 remains experimental. The native backend supports Windows, Linux, and macOS contact PC/SC.
See [platform setup](platforms.md). Historical hardware results came with the imported
project; the refactor requires fresh hardware validation. Unknown ATRs are
probed, but future generations are not guaranteed to work.

This SDK is independent and unofficial. It does not authenticate the card,
read fingerprints, or bypass protected files. It contains no proprietary ICP
credentials or runtime components.

This Wiki is generated from the repository's documentation. See
[Wiki setup](wiki-setup.md) for regeneration and publication instructions.
