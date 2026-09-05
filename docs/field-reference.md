# Field reference

Generated from the public Rustdoc comments in `src/data.rs`. Edit those comments and run `python scripts/build_wiki.py` to refresh this page.

`Option` fields can be absent. Consult the containing group's read status before using them. All accessors and fields refer to the same owned snapshot.

## EmiratesIdData

| Field | Rust type | Meaning |
| --- | --- | --- |
| `reader_name` | `String` | Name of the Windows PC/SC reader. |
| `card_generation` | `CardGeneration` | Chip generation classified from the ATR. |
| `id_number` | `String` | Required 15-character Emirates ID number. |
| `card_number` | `String` | Required card number/serial stored by the ID application. |
| `photo_jpeg` | `Option<Vec<u8>>` | JPEG photograph bytes, when requested and accessible. |
| `holder_signature_image` | `Option<Vec<u8>>` | Holder-signature image payload, when requested and available. |
| `non_modifiable` | `NonModifiableData` | Core identity data. |
| `modifiable` | `ModifiableData` | Optional changeable and V2 extension data. |
| `read_status` | `ReadStatus` | Per-group access outcome for this read. |

## NonModifiableData

| Field | Rust type | Meaning |
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

| Field | Rust type | Meaning |
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

| Field | Rust type | Meaning |
| --- | --- | --- |
| `identity` | `DataGroupStatus` | Required identifiers file. |
| `photo` | `DataGroupStatus` | Optional photograph file. |
| `non_modifiable` | `DataGroupStatus` | Required core identity file. |
| `modifiable` | `DataGroupStatus` | Optional occupation/residency/passport/education file. |
| `holder_signature_image` | `DataGroupStatus` | Optional V2 signature-image file. |
