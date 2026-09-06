# Photos and signatures

Photographs and signature images are optional read groups. `session.read_all()`
requests both; `ReadOptions::identity_only()` skips them. Either image can be
absent or inaccessible on an individual card.

```rust,no_run
use emirates_id_reader::{CardSession, ReadOptions};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let session = CardSession::connect_first()?;
    let card = session.read_with_options(ReadOptions::identity_only().with_photo(true))?;

    if let Some(jpeg) = card.photo() {
        // Hand the bytes to your image decoder; copy only if the UI needs ownership.
        let _ = jpeg;
    }
    Ok(())
}
```

## Accessors and options

| Method | Returns | Value |
| --- | --- | --- |
| `photo()` | `Option<&[u8]>` | JPEG photograph bytes |
| `signature()` | `Option<&[u8]>` | Holder-signature payload |

| Option | Effect |
| --- | --- |
| `ReadOptions::identity_only()` | Neither image is read |
| `.with_photo(true)` | Adds the photograph |
| `.with_holder_signature_image(true)` | Adds the signature image |
| `ReadOptions::all()`, `session.read_all()` | Reads both |

Both accessors borrow the snapshot and trigger no chip access. Neither writes a
file: storing or transmitting an image is always an explicit application step.

## What the library validates

A non-empty photo payload must carry the JPEG prefix and a well-formed TLV
structure. The library does not decode the pixels: dimensions, colour space, and
whether the image opens at all remain your decoder's responsibility, and it must
handle failure.

The signature payload is returned opaque. Its format is card-dependent and the
library asserts no MIME type, so probe the bytes rather than assuming JPEG.

## When an image is missing

`None` alone does not say why. Read the group status:

| `read_status` value | Meaning |
| --- | --- |
| `Read` | The file was read; a `None` accessor means the field was absent or empty |
| `NotRequested` | The option was disabled |
| `NotAvailable` | The card reports no such file |
| `Protected` | The file needs authentication or secure messaging |

Check `card.read_status().photo` and `card.read_status().holder_signature_image`.
The handled absent-file and access-refusal responses produce group statuses.
Other failures, including transport errors and malformed image fields, fail the
whole read. Retrying with unchanged access conditions does not unlock a
`Protected` image; this library provides no authentication operation.

The signature image is documented in V2 and was also returned by the tested V1
card. Inspect the group status rather than treating generation as an access rule.
See [V1/V2 compatibility](Card-Generations).

## Related

- [Application integration](Application-Integration) for displaying images in a UI
- [Errors and read statuses](Error-Handling) for the full status model
- [Security and access boundaries](Security) for what is never requested
