//! Card generations, read options, and decoded public data.

use serde::Serialize;

/// Emirates ID chip generation as defined by the official ICP SDK guides.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CardGeneration {
    /// First-generation Emirates ID chip.
    V1,
    /// Second-generation Emirates ID chip.
    V2,
    /// The card selected the Emirates ID application but its ATR is not in the
    /// published V1/V2 list. Reading is still attempted for forward compatibility.
    Unknown,
}

impl CardGeneration {
    /// Classifies an ATR using the V1/V2 values published by ICP.
    pub fn from_atr(atr: &[u8]) -> Self {
        const V1_ATRS: [&[u8]; 2] = [
            &[
                0x3B, 0x6A, 0x00, 0x00, 0x80, 0x65, 0xA2, 0x01, 0x30, 0x01, 0x3D, 0x72, 0xD6, 0x41,
            ],
            &[
                0x3B, 0x6A, 0x00, 0x00, 0x80, 0x65, 0xA2, 0x01, 0x31, 0x01, 0x3D, 0x72, 0xD6, 0x41,
            ],
        ];
        const V2_ATRS: [&[u8]; 2] = [
            &[
                0x3B, 0x7A, 0x95, 0x00, 0x00, 0x80, 0x65, 0xA2, 0x01, 0x30, 0x01, 0x3D, 0x72, 0xD6,
                0x41,
            ],
            &[
                0x3B, 0x7A, 0x95, 0x00, 0x00, 0x80, 0x65, 0xA2, 0x01, 0x31, 0x01, 0x3D, 0x72, 0xD6,
                0x41,
            ],
        ];

        if V1_ATRS.contains(&atr) {
            Self::V1
        } else if V2_ATRS.contains(&atr) {
            Self::V2
        } else {
            Self::Unknown
        }
    }
}

/// Controls expensive optional reads. [`ReadOptions::default`] reads every
/// public group, preserving the behaviour of [`crate::CardSession::read`].
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ReadOptions {
    /// Read the photograph elementary file.
    pub photo: bool,
    /// Read occupation, residency, passport, and education data.
    pub modifiable_data: bool,
    /// Read the V2 holder-signature image file.
    pub holder_signature_image: bool,
}

impl ReadOptions {
    /// Requests every supported public data group.
    pub const fn all() -> Self {
        Self {
            photo: true,
            modifiable_data: true,
            holder_signature_image: true,
        }
    }

    /// Fast path for matching/check-in workflows: identifiers and core identity
    /// fields only, without large binary images or extended data.
    pub const fn identity_only() -> Self {
        Self {
            photo: false,
            modifiable_data: false,
            holder_signature_image: false,
        }
    }
}

impl Default for ReadOptions {
    fn default() -> Self {
        Self::all()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
/// Result of reading one optional card data group.
pub enum DataGroupStatus {
    /// The file was available and read.
    Read,
    /// The caller disabled the group through [`ReadOptions`].
    NotRequested,
    /// The card reports that the file is absent.
    NotAvailable,
    /// The card requires authentication or secure messaging for the file.
    Protected,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
/// Access outcome for every elementary-file group used by a read.
pub struct ReadStatus {
    /// Required identifiers file.
    pub identity: DataGroupStatus,
    /// Optional photograph file.
    pub photo: DataGroupStatus,
    /// Required core identity file.
    pub non_modifiable: DataGroupStatus,
    /// Optional occupation/residency/passport/education file.
    pub modifiable: DataGroupStatus,
    /// Optional V2 signature-image file.
    pub holder_signature_image: DataGroupStatus,
}

#[derive(Clone, Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
/// Core identity values which are not normally changed after card issuance.
pub struct NonModifiableData {
    /// Card/document type code.
    pub id_type: Option<String>,
    /// Issue date as `YYYY-MM-DD`.
    pub issue_date: Option<String>,
    /// Expiry date as `YYYY-MM-DD`.
    pub expiry_date: Option<String>,
    /// Arabic honorific/title.
    pub title_arabic: Option<String>,
    /// Full Arabic name.
    pub full_name_arabic: Option<String>,
    /// English honorific/title.
    pub title_english: Option<String>,
    /// Full English name.
    pub full_name_english: Option<String>,
    /// Gender/sex code stored by the card.
    pub gender: Option<String>,
    /// Arabic nationality description.
    pub nationality_arabic: Option<String>,
    /// English nationality description when supplied by the card generation.
    pub nationality_english: Option<String>,
    /// Three-character nationality code.
    pub nationality_code: Option<String>,
    /// Date of birth as `YYYY-MM-DD`.
    pub date_of_birth: Option<String>,
    /// Arabic place of birth; a V2 extension.
    pub place_of_birth_arabic: Option<String>,
    /// English place of birth; a V2 extension.
    pub place_of_birth_english: Option<String>,
}

#[derive(Clone, Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
/// Changeable and extended public fields. Many are V2-only or holder-dependent.
pub struct ModifiableData {
    /// Occupation code, preserving leading zeroes.
    pub occupation_code: Option<String>,
    /// Arabic occupation description.
    pub occupation_arabic: Option<String>,
    /// English occupation description.
    pub occupation_english: Option<String>,
    /// Family identifier.
    pub family_id: Option<String>,
    /// Arabic occupation type.
    pub occupation_type_arabic: Option<String>,
    /// English occupation type.
    pub occupation_type_english: Option<String>,
    /// Occupation-field code.
    pub occupation_field_code: Option<String>,
    /// Arabic employer/company name.
    pub company_name_arabic: Option<String>,
    /// English employer/company name.
    pub company_name_english: Option<String>,
    /// Marital-status code.
    pub marital_status_code: Option<String>,
    /// Husband's Emirates ID number when populated.
    pub husband_id_number: Option<String>,
    /// Sponsor-type code.
    pub sponsor_type_code: Option<String>,
    /// Sponsor unified number.
    pub sponsor_unified_number: Option<String>,
    /// Sponsor name.
    pub sponsor_name: Option<String>,
    /// Residency-type code.
    pub residency_type_code: Option<String>,
    /// Residency number.
    pub residency_number: Option<String>,
    /// Residency expiry date as `YYYY-MM-DD`.
    pub residency_expiry_date: Option<String>,
    /// Passport number.
    pub passport_number: Option<String>,
    /// Passport-type code.
    pub passport_type_code: Option<String>,
    /// Passport-country code.
    pub passport_country_code: Option<String>,
    /// Arabic passport-country description.
    pub passport_country_arabic: Option<String>,
    /// English passport-country description.
    pub passport_country_english: Option<String>,
    /// Passport issue date as `YYYY-MM-DD`.
    pub passport_issue_date: Option<String>,
    /// Passport expiry date as `YYYY-MM-DD`.
    pub passport_expiry_date: Option<String>,
    /// Qualification-level code.
    pub qualification_level_code: Option<String>,
    /// Arabic qualification-level description.
    pub qualification_level_arabic: Option<String>,
    /// English qualification-level description.
    pub qualification_level_english: Option<String>,
    /// Arabic degree description.
    pub degree_description_arabic: Option<String>,
    /// English degree description.
    pub degree_description_english: Option<String>,
    /// Field-of-study code.
    pub field_of_study_code: Option<String>,
    /// Arabic field-of-study description.
    pub field_of_study_arabic: Option<String>,
    /// English field-of-study description.
    pub field_of_study_english: Option<String>,
    /// Arabic place-of-study description.
    pub place_of_study_arabic: Option<String>,
    /// English place-of-study description.
    pub place_of_study_english: Option<String>,
    /// Graduation date as `YYYY-MM-DD`.
    pub date_of_graduation: Option<String>,
    /// Mother's Arabic name (V1 stores the first name; V2 may store the full name).
    pub mother_full_name_arabic: Option<String>,
    /// Mother's English name (V1 stores the first name; V2 may store the full name).
    pub mother_full_name_english: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
/// Complete in-memory result of one public-data read.
pub struct EmiratesIdData {
    /// Name of the Windows PC/SC reader.
    pub reader_name: String,
    /// Chip generation classified from the ATR.
    pub card_generation: CardGeneration,
    /// Required 15-character Emirates ID number.
    pub id_number: String,
    /// Required card number/serial stored by the ID application.
    pub card_number: String,
    /// JPEG photograph bytes, when requested and accessible.
    pub photo_jpeg: Option<Vec<u8>>,
    /// Holder-signature image payload, when requested and available.
    pub holder_signature_image: Option<Vec<u8>>,
    /// Core identity data.
    pub non_modifiable: NonModifiableData,
    /// Optional changeable and V2 extension data.
    pub modifiable: ModifiableData,
    /// Per-group access outcome for this read.
    pub read_status: ReadStatus,
}

/// These values are documented as public, but their card files reject a plain
/// unauthenticated read with status 6982, so this clean SDK deliberately skips them.
pub const PROTECTED_AND_SKIPPED_FIELDS: &[&str] = &[
    "home address details",
    "work address details",
    "resident phone number",
    "mobile phone number",
    "email address",
];
