# Emirates ID card generations

The names **V1** and **V2** in this SDK refer to the chip generations used by
the official ICP SDK documentation. They do not refer only to the artwork
printed on the card, the issue date, or whether the card is still valid.

`CardGeneration` is deliberately separate from field availability. The ATR
identifies the documented chip family; the optional values and `ReadStatus`
report what that individual card actually exposes. For example, a
hardware-tested, unexpired V1 card returned extended public fields plus its
photograph and holder-signature image. Callers must therefore inspect the data
and group status instead of treating the V1/V2 label as a field-access policy.

## Detection

`CardSession::card_generation()` compares the card's ATR with the values
published in the ICP developer guide:

| Generation | Published ATR |
| --- | --- |
| V1 | `3B 6A 00 00 80 65 A2 01 30 01 3D 72 D6 41` |
| V1 | `3B 6A 00 00 80 65 A2 01 31 01 3D 72 D6 41` |
| V2 | `3B 7A 95 00 00 80 65 A2 01 30 01 3D 72 D6 41` |
| V2 | `3B 7A 95 00 00 80 65 A2 01 31 01 3D 72 D6 41` |

An unrecognised ATR is reported as `CardGeneration::Unknown`; the SDK still
attempts the normal application and file probes for forward compatibility.
ATR recognition identifies a documented card family but is not proof that a
card is genuine.

Source: [ICP C++ Developer Guide](https://icp.gov.ae/ica_files/documentations/cplusplus_developer_guide.pdf), section 5.7.

## Data stored in V1

The ICP V1 field specification lists:

- ID number, card number, issue date and expiry date
- Arabic and English title and full name
- sex, nationality and date of birth
- mother's first name in Arabic and English
- photograph
- occupation, marital status and husband's ID number
- sponsor type, number and name
- residency type and number
- document type and family ID
- two fingerprints

Stored does not mean anonymously readable. In particular, ICP states that the
photograph on old cards is protected by secure messaging, and biometrics are
outside this SDK's public-data scope. The SDK never attempts fingerprint
access. An inaccessible optional file is represented in `ReadStatus` instead
of failing the entire public-data read.

Source: [Fields Stored in UAE ID Card V1](https://icp.gov.ae/wp-content/uploads/2020/10/Fields_Stored_in_UAE_ID_Card_V1.docx) and the [ICP SDK FAQ](https://icp.gov.ae/en/id-card-benefits/sdk-toolkit/sdk-faq/).

## Data added in V2

The V2 field document describes additional groups:

- passport number, type, country, issue date and expiry date
- Arabic and English place of birth
- qualification, degree, field/place of study and graduation date
- expanded occupation type/field and company names
- mother's full name, holder signature image and sponsor unified number
- family-book records
- detailed home/work address, phone numbers and email

The developer guide says applications using the proprietary toolkit must not
request its V2-only extension API from a V1 card. This Rust SDK does not call
that toolkit API: it probes the public elementary files and models every value
as optional. This both preserves one stable data model and handles later V1
cards that expose some fields associated with the extended layout.

Source: [Fields Stored in UAE ID Card V2](https://icp.gov.ae/ica_files/documentations/Fields_Stored_in_UAE_ID_Card_V2.docx).

## Field compatibility matrix

`Yes` means the official card-generation documents describe the field. `No`
means the published V1 document does not list it, not that every V1 card is
physically incapable of exposing it. The table does not override a card's
security policy or guarantee that a holder has a non-empty value.

| SDK field/group | V1 | V2 | Notes |
| --- | --- | --- | --- |
| `id_number` | Yes | Yes | Required by the SDK |
| `card_number` | Yes | Yes | Required by the SDK |
| `photo_jpeg` | Stored, protected on old cards | Yes | Optional read; consult `read_status.photo` |
| `id_type` | Yes | Yes | Called document type in the V1 field list |
| `issue_date`, `expiry_date` | Yes | Yes | Packed BCD, returned as ISO dates |
| Arabic/English titles | Yes | Yes | May be blank |
| Arabic/English full names | Yes | Yes | Comma-separated name components; `get_formatted_name()` joins them for display |
| `gender` | Yes | Yes | Stored code returned unchanged; `gender()` interprets it |
| Arabic nationality | Yes | Yes | Description |
| nationality code | Yes | Yes | V1's English nationality entry is three bytes |
| English nationality description | No separate V1 description | Yes | V2 extension |
| `date_of_birth` | Yes | Yes | Packed BCD, returned as ISO date |
| Arabic/English place of birth | No | Yes | V2 extension |
| occupation code | Yes | Yes | Leading zeroes preserved |
| Arabic/English occupation descriptions | No | Yes | V2 extension |
| occupation type/field | No | Yes | V2 extension |
| Arabic/English company name | No | Yes | V2 extension |
| `marital_status_code` | Yes | Yes | Optional holder data |
| `husband_id_number` | Yes | Yes | Optional holder data |
| sponsor type/number/name | Yes | Yes | V2 explicitly adds a unified-number form |
| residency type/number | Yes | Yes | Optional holder data |
| passport fields | No | Yes | Number, type, country, issue and expiry |
| qualification and degree | No | Yes | Codes plus Arabic/English descriptions |
| field/place of study and graduation | No | Yes | V2 extension |
| mother's Arabic/English name | First name | Full name | Exposed through the same SDK fields |
| `holder_signature_image` | No | Yes | Optional V2 file |
| address, phone and email | No | Stored | Protected on the tested V2 card; intentionally not read |
| fingerprints | Stored, protected | Stored, protected | Never accessed by this public-data SDK |
| family-book data | No | Stored separately | Not part of anonymous public-data reading |

## Historical validation supplied with the import

These results describe the imported 0.2 implementation. Fresh hardware
validation of 0.3 is pending; synthetic tests are not hardware evidence.

- V1 and V2 contact reading are hardware-validated with an HID OMNIKEY 3x21
  reader, using cards that return the exact published ATRs.
- The tested V1 card returned `3B 6A 00 ... 31 ...`, while the tested V2 card
  returned `3B 7A 95 ... 31 ...`.
- Both generations passed identity-only and full public-data reads. The tested
  V1 card exposed more fields than the historical V1 field list, so generation
  and actual group availability remain intentionally independent.
- Removal/reinsert lifecycle validation remains outstanding for the V1 card;
  see [testing](testing.md).
