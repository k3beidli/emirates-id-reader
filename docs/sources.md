# Sources and acknowledgments

This independent SDK uses the publicly available references below. It ships no
proprietary toolkit files, credentials, or runtime components. Citations also appear beside the claims they support; this
page identifies each document precisely.

Official document links were checked on 2026-09-06. Document publication dates
are separate from this link-check date.

## Official specifications

Published by the **Federal Authority for Identity, Citizenship, Customs & Port
Security (ICP)** on its
[ID Card Toolkit documentation index](https://icp.gov.ae/en/id-card-benefits/sdk-toolkit/documentation/).

### C++ Developer guide

[EIDA Toolkit v2.5 — Developer's guide for C++](https://icp.gov.ae/ica_files/documentations/cplusplus_developer_guide.pdf)
is a 66-page guide authored by the Emirates Identity Authority (EIDA) and hosted
by ICP under the label *C++ Developer guide*.

Its document-details table is dated 10 April 2012. The revision history records
version 1.0 for Toolkit 2.5 on 2 May 2012; these are distinct dates in the file.

Used for:

- **§5.7 "Read Card ATR"** (p. 16), the source of the four published ATR values
  and their reset types, in [V1/V2 compatibility](card-generations.md). The same
  section states that a matching ATR is not sufficient to establish that a card
  is genuine, which is the basis for the authenticity limit described in
  [security and access boundaries](security.md).
- The toolkit's separate genuineness and secure-messaging operations
  (§5.24–5.26), which this SDK does not implement.

This SDK does not call the toolkit. It talks to the card through native PC/SC
and uses the guide for the documented card families and toolkit behavior.
It is not a complete specification of this SDK's direct PC/SC implementation.

### Fields Stored in UAE ID Card V1

[Fields Stored in UAE ID Card V1](https://icp.gov.ae/wp-content/uploads/2020/10/Fields_Stored_in_UAE_ID_Card_V1.docx)
(Word document).

The V1 field inventory used for the V1 column of the
[field compatibility matrix](card-generations.md). It also documents six comma
separators in each full-name field and the `M`, `F`, and `X` sex codes. See
[names](names.md) and [codes and identifiers](codes-and-identifiers.md).

### Fields Stored in UAE ID Card V2

[Fields Stored in UAE ID Card V2](https://icp.gov.ae/ica_files/documentations/Fields_Stored_in_UAE_ID_Card_V2.docx)
(Word document).

The V2 field inventory, describing the passport, place-of-birth, education,
expanded occupation, signature-image, and family-book additions.

### SDK FAQ

[SDK FAQ](https://icp.gov.ae/en/id-card-benefits/sdk-toolkit/sdk-faq/).

Supporting material on toolkit behavior, cited alongside the V1 field list in
[V1/V2 compatibility](card-generations.md).

## Software

- [pcsc-rust](https://github.com/bluetech/pcsc-rust) provides the PC/SC
  bindings, covering WinSCard on Windows, pcsc-lite on Linux, and the system
  PCSC framework on macOS.
- [Serde](https://serde.rs/) derives the snapshot's serialization.
- GitHub's documentation on
  [wiki pages](https://docs.github.com/en/communities/documenting-your-project-with-wikis/adding-or-editing-wiki-pages)
  and [sidebars and footers](https://docs.github.com/en/communities/documenting-your-project-with-wikis/creating-a-footer-or-sidebar-for-your-wiki)
  describes the publication mechanics used in
  [documentation maintenance](wiki-setup.md).

## Acknowledgments

The reader began as application-specific code extracted into a standalone
project at commit `14e415a`, and the SDK refactor starts at version 0.3.0. The
hardware results recorded in [testing](testing.md) were supplied with that
import; they describe the earlier implementation and are labelled as such.

## Trademarks and scope

Emirates ID, EIDA, and ICP are the property of their respective owners. This
project is not affiliated with, endorsed by, or supported by ICP. The
original documents and toolkit binaries are not bundled with the SDK. The
references describe card data and toolkit behavior; they do not imply ICP
endorsement of this implementation.
