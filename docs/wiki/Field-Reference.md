# Field reference

Look up the fields in a read result, the status of each group, and the options used to request it.

- [Snapshot](#emiratesiddata)
- [Core identity](#nonmodifiabledata)
- [Extended information](#modifiabledata)
- [Group statuses](#readstatus)
- [Read options](#readoptions)
- [Fields not read](#fields-not-read-by-this-library)

`Option` fields can be absent. Check the containing group's read status before interpreting `None`. For examples and formatting rules, see [names](Names), [codes and identifiers](Codes-And-Identifiers), [dates](Dates), [photos and signatures](Photos-And-Signatures), or [extended information](Extended-Information).

## EmiratesIdData

| Field or accessor | Rust type | Meaning |
| --- | --- | --- |
| `reader_name()` | `&str` | Name of the native PC/SC reader. |
| `card_generation()` | `CardGeneration` | Chip generation classified from the ATR. |
| `id_number()` | `&str` | Required 15-character Emirates ID number. |
| `card_number()` | `&str` | Required card number/serial stored by the ID application. |
| `photo()` | `Option<&[u8]>` | JPEG photograph bytes, when requested and accessible. |
| `signature()` | `Option<&[u8]>` | Holder-signature image payload, when requested and available. |
| `identity()` | `&NonModifiableData` | Core identity data. |
| `extended()` | `&ModifiableData` | Optional changeable and V2 extension data. |
| `read_status()` | `&ReadStatus` | Per-group access outcome for this read. |

## NonModifiableData

| Field or accessor | Rust type | Meaning |
| --- | --- | --- |
| `id_type` | `Option<String>` | Card/document type code. |
| `issue_date` | `Option<String>` | Issue date as `YYYY-MM-DD`. |
| `expiry_date` | `Option<String>` | Expiry date as `YYYY-MM-DD`. |
| `title_arabic` | `Option<String>` | Arabic honorific/title. |
| `full_name_arabic` | `Option<String>` | Full Arabic name. |
| `title_english` | `Option<String>` | English honorific/title. |
| `full_name_english` | `Option<String>` | Full English name. |
| `gender` | `Option<String>` | Gender/sex code stored by the card. |
| `nationality_arabic` | `Option<String>` | Arabic nationality description. |
| `nationality_english` | `Option<String>` | English nationality description when supplied by the card generation. |
| `nationality_code` | `Option<String>` | Three-character nationality code. |
| `date_of_birth` | `Option<String>` | Date of birth as `YYYY-MM-DD`. |
| `place_of_birth_arabic` | `Option<String>` | Arabic place of birth; a V2 extension. |
| `place_of_birth_english` | `Option<String>` | English place of birth; a V2 extension. |

## ModifiableData

| Field or accessor | Rust type | Meaning |
| --- | --- | --- |
| `occupation_code` | `Option<String>` | Occupation code, preserving leading zeroes. |
| `occupation_arabic` | `Option<String>` | Arabic occupation description. |
| `occupation_english` | `Option<String>` | English occupation description. |
| `family_id` | `Option<String>` | Family identifier. |
| `occupation_type_arabic` | `Option<String>` | Arabic occupation type. |
| `occupation_type_english` | `Option<String>` | English occupation type. |
| `occupation_field_code` | `Option<String>` | Occupation-field code. |
| `company_name_arabic` | `Option<String>` | Arabic employer/company name. |
| `company_name_english` | `Option<String>` | English employer/company name. |
| `marital_status_code` | `Option<String>` | Marital-status code. |
| `husband_id_number` | `Option<String>` | Husband's Emirates ID number when populated. |
| `sponsor_type_code` | `Option<String>` | Sponsor-type code. |
| `sponsor_unified_number` | `Option<String>` | Sponsor unified number. |
| `sponsor_name` | `Option<String>` | Sponsor name. |
| `residency_type_code` | `Option<String>` | Residency-type code. |
| `residency_number` | `Option<String>` | Residency number. |
| `residency_expiry_date` | `Option<String>` | Residency expiry date as `YYYY-MM-DD`. |
| `passport_number` | `Option<String>` | Passport number. |
| `passport_type_code` | `Option<String>` | Passport-type code. |
| `passport_country_code` | `Option<String>` | Passport-country code. |
| `passport_country_arabic` | `Option<String>` | Arabic passport-country description. |
| `passport_country_english` | `Option<String>` | English passport-country description. |
| `passport_issue_date` | `Option<String>` | Passport issue date as `YYYY-MM-DD`. |
| `passport_expiry_date` | `Option<String>` | Passport expiry date as `YYYY-MM-DD`. |
| `qualification_level_code` | `Option<String>` | Qualification-level code. |
| `qualification_level_arabic` | `Option<String>` | Arabic qualification-level description. |
| `qualification_level_english` | `Option<String>` | English qualification-level description. |
| `degree_description_arabic` | `Option<String>` | Arabic degree description. |
| `degree_description_english` | `Option<String>` | English degree description. |
| `field_of_study_code` | `Option<String>` | Field-of-study code. |
| `field_of_study_arabic` | `Option<String>` | Arabic field-of-study description. |
| `field_of_study_english` | `Option<String>` | English field-of-study description. |
| `place_of_study_arabic` | `Option<String>` | Arabic place-of-study description. |
| `place_of_study_english` | `Option<String>` | English place-of-study description. |
| `date_of_graduation` | `Option<String>` | Graduation date as `YYYY-MM-DD`. |
| `mother_full_name_arabic` | `Option<String>` | Mother's Arabic name (V1 stores the first name; V2 may store the full name). |
| `mother_full_name_english` | `Option<String>` | Mother's English name (V1 stores the first name; V2 may store the full name). |

## ReadStatus

| Field or accessor | Rust type | Meaning |
| --- | --- | --- |
| `identity` | `DataGroupStatus` | Required identifiers file. |
| `photo` | `DataGroupStatus` | Optional photograph file. |
| `non_modifiable` | `DataGroupStatus` | Required core identity file. |
| `modifiable` | `DataGroupStatus` | Optional occupation/residency/passport/education file. |
| `holder_signature_image` | `DataGroupStatus` | Optional signature-image file. |

## ReadOptions

| Field or accessor | Rust type | Meaning |
| --- | --- | --- |
| `photo` | `bool` | Read the photograph elementary file. |
| `modifiable_data` | `bool` | Read occupation, residency, passport, and education data. |
| `holder_signature_image` | `bool` | Read the holder-signature image file when available. |

<a id="fields-not-read-by-this-sdk"></a>


## Fields not read by this library

Not read by this library; access restrictions apply.

Address and contact fields excluded from this library's reads and data model. The earlier implementation encountered access refusal (status 6982) for these files. This library provides no authentication or secure-messaging support.

- home address details
- work address details
- resident phone number
- mobile phone number
- email address

These fields have no getter, snapshot field, or per-group status. `DataGroupStatus::Protected` instead describes a supported optional group whose read was refused. See [errors and read statuses](Error-Handling) and [security and access boundaries](Security).

---

This page is generated from Rustdoc comments in `src/data.rs`. To change it, edit those comments and run `python scripts/build_wiki.py`; see [documentation maintenance](Wiki-Setup).
