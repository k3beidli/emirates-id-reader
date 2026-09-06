# Sources and acknowledgments

References used by the library. Official document links were checked on 2026-09-06.

## Official specifications

Published by the **Federal Authority for Identity, Citizenship, Customs & Port
Security (ICP)** on its
[ID Card Toolkit documentation index](https://icp.gov.ae/en/id-card-benefits/sdk-toolkit/documentation/).

### C++ Developer guide

[EIDA Toolkit v2.5 — Developer's guide for C++](https://icp.gov.ae/ica_files/documentations/cplusplus_developer_guide.pdf)
was authored by the Emirates Identity Authority (EIDA) and is hosted by ICP.

Its document-details table is dated 10 April 2012. The revision history records
version 1.0 for Toolkit 2.5 on 2 May 2012; these are distinct dates in the file.

Used for:

- **§5.7 "Read Card ATR"** (p. 16), the source of the four published ATR values
  and their reset types, in [V1/V2 compatibility](Card-Generations). The same
  section explains the limits of ATR-based identification.
- The toolkit's separate genuineness and secure-messaging operations
  (§5.24–5.26), which this library does not implement.

The library uses native PC/SC, not the proprietary toolkit API.

### Fields Stored in UAE ID Card V1

[Fields Stored in UAE ID Card V1](https://icp.gov.ae/wp-content/uploads/2020/10/Fields_Stored_in_UAE_ID_Card_V1.docx)
(Word document).

The V1 field inventory used for the V1 column of the
[field compatibility matrix](Card-Generations). It also documents six comma
separators in each full-name field and the `M`, `F`, and `X` sex codes. See
[names](Names) and [codes and identifiers](Codes-And-Identifiers).

### Fields Stored in UAE ID Card V2

[Fields Stored in UAE ID Card V2](https://icp.gov.ae/ica_files/documentations/Fields_Stored_in_UAE_ID_Card_V2.docx)
(Word document).

The V2 field inventory, describing the passport, place-of-birth, education,
expanded occupation, signature-image, and family-book additions.

### SDK FAQ

[SDK FAQ](https://icp.gov.ae/en/id-card-benefits/sdk-toolkit/sdk-faq/).

Supporting material on toolkit behavior, cited alongside the V1 field list in
[V1/V2 compatibility](Card-Generations).

## Software

- [pcsc-rust](https://github.com/bluetech/pcsc-rust) provides the PC/SC
  bindings, covering WinSCard on Windows, pcsc-lite on Linux, and the system
  PCSC framework on macOS.
- [Serde](https://serde.rs/) derives the snapshot's serialization.
- GitHub's documentation on
  [wiki pages](https://docs.github.com/en/communities/documenting-your-project-with-wikis/adding-or-editing-wiki-pages)
  and [sidebars and footers](https://docs.github.com/en/communities/documenting-your-project-with-wikis/creating-a-footer-or-sidebar-for-your-wiki)
  describes the publication mechanics used in
  [documentation maintenance](Wiki-Setup).

## Trademarks and scope

This project is unofficial and is not affiliated with or endorsed by ICP.
Original documents and proprietary toolkit binaries are not bundled.

## Protocol constants

The standalone code supplied for this project already contained the application
identifier, file identifiers, and TLV mappings. Their repository provenance is
[the initial implementation, commit 14e415a](https://github.com/k3beidli/emirates-id-reader/blob/14e415a/src/lib.rs).
The original external source of these byte mappings has not been established.
The ICP field inventories describe fields; they are not cited here as proof of
these exact APDU or TLV values.

| Constant | Current location |
| --- | --- |
| AID `A00000024300130000000101` | `src/protocol.rs`, `select_application` |
| Public directory `0200` | `src/protocol.rs` |
| Files `0201`, `0202`, `0203`, `0205`, `0207` | `src/protocol.rs` |
| Identifier/image TLV tags | `src/protocol.rs` |
| Core and extended TLV tags | `src/decode.rs` |
| V1/V2 ATR bytes | `src/data.rs`; official source: Toolkit guide §5.7 |

Historical hardware results in [Testing](Testing) describe the earlier
implementation and do not replace validation of the current version.
