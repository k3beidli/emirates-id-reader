//! Card generations, read options, and decoded public data.

#[cfg(feature = "serde")]
use serde::Serialize;

/// Emirates ID chip generation as defined by the official ICP SDK guides.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
#[non_exhaustive]
pub enum CardGeneration {
    /// First-generation Emirates ID chip.
    V1,
    /// Second-generation Emirates ID chip.
    V2,
    /// The ATR is not recognized. Application selection and reading may still
    /// succeed, but this classification alone does not identify an Emirates ID.
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

/// Controls optional reads. The default requests identifiers and core identity only.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[must_use = "reading options have no effect unless passed to a read"]
#[non_exhaustive]
pub struct ReadOptions {
    /// Read the photograph elementary file.
    pub photo: bool,
    /// Read occupation, residency, passport, and education data.
    pub modifiable_data: bool,
    /// Read the holder-signature image file when available.
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
    /// Enables or disables photograph transfer.
    pub const fn with_photo(mut self, enabled: bool) -> Self {
        self.photo = enabled;
        self
    }
    /// Enables or disables occupation, residency, passport, and education transfer.
    pub const fn with_modifiable_data(mut self, enabled: bool) -> Self {
        self.modifiable_data = enabled;
        self
    }
    /// Enables or disables holder-signature image transfer.
    pub const fn with_holder_signature_image(mut self, enabled: bool) -> Self {
        self.holder_signature_image = enabled;
        self
    }
}

impl Default for ReadOptions {
    fn default() -> Self {
        Self::identity_only()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
/// Result of reading one optional card data group.
#[non_exhaustive]
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

#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
/// Access outcome for every elementary-file group used by a read.
#[non_exhaustive]
pub struct ReadStatus {
    /// Required identifiers file.
    pub identity: DataGroupStatus,
    /// Optional photograph file.
    pub photo: DataGroupStatus,
    /// Required core identity file.
    pub non_modifiable: DataGroupStatus,
    /// Optional occupation/residency/passport/education file.
    pub modifiable: DataGroupStatus,
    /// Optional signature-image file.
    pub holder_signature_image: DataGroupStatus,
}

#[derive(Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
/// Core identity values which are not normally changed after card issuance.
#[non_exhaustive]
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

#[derive(Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
/// Changeable and extended public fields. Many are V2-only or holder-dependent.
#[non_exhaustive]
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

#[derive(Clone)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
/// Complete in-memory result of one public-data read.
pub struct EmiratesIdData {
    /// Name of the native PC/SC reader.
    pub(crate) reader_name: String,
    /// Chip generation classified from the ATR.
    pub(crate) card_generation: CardGeneration,
    /// Required 15-character Emirates ID number.
    pub(crate) id_number: String,
    /// Required card number/serial stored by the ID application.
    pub(crate) card_number: String,
    /// JPEG photograph bytes, when requested and accessible.
    pub(crate) photo_jpeg: Option<Vec<u8>>,
    /// Holder-signature image payload, when requested and available.
    pub(crate) holder_signature_image: Option<Vec<u8>>,
    /// Core identity data.
    pub(crate) non_modifiable: NonModifiableData,
    /// Optional changeable and V2 extension data.
    pub(crate) modifiable: ModifiableData,
    /// Per-group access outcome for this read.
    pub(crate) read_status: ReadStatus,
}

/// Address and contact fields excluded from this library's reads and data model.
/// The earlier implementation encountered access refusal (status 6982) for
/// these files. This library provides no authentication or secure-messaging support.
pub const PROTECTED_AND_SKIPPED_FIELDS: &[&str] = &[
    "home address details",
    "work address details",
    "resident phone number",
    "mobile phone number",
    "email address",
];

impl EmiratesIdData {
    /// Returns the reader name recorded for this snapshot.
    pub fn reader_name(&self) -> &str {
        &self.reader_name
    }
    /// Returns the chip generation recorded for this snapshot.
    pub fn card_generation(&self) -> CardGeneration {
        self.card_generation
    }
    /// Borrows the access outcome of each group.
    pub fn read_status(&self) -> &ReadStatus {
        &self.read_status
    }
    /// Starts building a snapshot from caller-supplied data, useful for tests.
    /// No reader is contacted and the values are not authenticated.
    /// Call [`EmiratesIdDataBuilder::build`] to validate identifiers and dates.
    pub fn builder(
        id_number: impl Into<String>,
        card_number: impl Into<String>,
    ) -> EmiratesIdDataBuilder {
        EmiratesIdDataBuilder {
            data: Self {
                reader_name: String::new(),
                card_generation: CardGeneration::Unknown,
                id_number: id_number.into(),
                card_number: card_number.into(),
                photo_jpeg: None,
                holder_signature_image: None,
                non_modifiable: NonModifiableData::default(),
                modifiable: ModifiableData::default(),
                read_status: ReadStatus {
                    identity: DataGroupStatus::Read,
                    non_modifiable: DataGroupStatus::Read,
                    photo: DataGroupStatus::NotRequested,
                    modifiable: DataGroupStatus::NotRequested,
                    holder_signature_image: DataGroupStatus::NotRequested,
                },
            },
        }
    }

    /// Returns the English full name exactly as the card stores it, falling back
    /// to Arabic if English is absent. The card delimits name components with
    /// commas, so this value is not a display string; use
    /// [`Self::formatted_name`] for that, or [`Self::name_components_in`] for
    /// the stored positions. No chip access or allocation occurs. Use
    /// [`Self::name_in`] for an exact language.
    pub fn name(&self) -> Option<&str> {
        self.name_in(Language::English)
            .or_else(|| self.name_in(Language::Arabic))
    }
    /// Returns the full name in the requested language exactly as stored, without
    /// fallback. See [`Self::name`] for why this may contain comma separators.
    pub fn name_in(&self, language: Language) -> Option<&str> {
        match language {
            Language::English => self.non_modifiable.full_name_english.as_deref(),
            Language::Arabic => self.non_modifiable.full_name_arabic.as_deref(),
        }
    }
    /// Borrows JPEG bytes. `None` means absent, empty, inaccessible, or not requested;
    /// inspect [`Self::read_status`] for the group outcome. The library checks the JPEG
    /// prefix, not complete image decodability. No file is created.
    pub fn photo(&self) -> Option<&[u8]> {
        self.photo_jpeg.as_deref()
    }
    /// Borrows the signature payload. Its image format is card-dependent.
    pub fn signature(&self) -> Option<&[u8]> {
        self.holder_signature_image.as_deref()
    }
    /// Returns the 15-digit Emirates ID number, preserving leading zeroes.
    /// Use [`Self::formatted_id_number`] for the grouping printed on the card.
    pub fn id_number(&self) -> &str {
        &self.id_number
    }
    /// Returns the card serial/number.
    pub fn card_number(&self) -> &str {
        &self.card_number
    }
    /// Borrows all core identity fields.
    pub fn identity(&self) -> &NonModifiableData {
        &self.non_modifiable
    }
    /// Borrows occupation, residency, passport, education, and family fields.
    pub fn extended(&self) -> &ModifiableData {
        &self.modifiable
    }
    /// Returns the date of birth as `YYYY-MM-DD`, when populated.
    pub fn date_of_birth(&self) -> Option<&str> {
        self.non_modifiable.date_of_birth.as_deref()
    }
    /// Returns the issue date as `YYYY-MM-DD`, when populated.
    pub fn issue_date(&self) -> Option<&str> {
        self.non_modifiable.issue_date.as_deref()
    }
    /// Returns the expiry date as `YYYY-MM-DD`, when populated.
    pub fn expiry_date(&self) -> Option<&str> {
        self.non_modifiable.expiry_date.as_deref()
    }
    /// Returns the gender code exactly as stored, when populated.
    /// Use [`Self::gender`] for the interpreted value.
    pub fn gender_code(&self) -> Option<&str> {
        self.non_modifiable.gender.as_deref()
    }
    /// Returns the nationality code, when populated.
    pub fn nationality_code(&self) -> Option<&str> {
        self.non_modifiable.nationality_code.as_deref()
    }
    /// Returns the nationality description in the requested language, without fallback.
    pub fn nationality_in(&self, language: Language) -> Option<&str> {
        match language {
            Language::English => self.non_modifiable.nationality_english.as_deref(),
            Language::Arabic => self.non_modifiable.nationality_arabic.as_deref(),
        }
    }
    /// Interprets the stored gender code. An unrecognized code yields
    /// [`Gender::Unrecognized`] rather than `None`, so an unknown value stays
    /// distinguishable from an absent one. [`Self::gender_code`] still returns the
    /// code exactly as the card stored it.
    pub fn gender(&self) -> Option<Gender> {
        self.gender_code().map(Gender::from_code)
    }
    /// Borrows the stored name components for `language`, in card order.
    ///
    /// The card delimits name components with commas. Empty positions are
    /// preserved so callers can see the structure the card stored; the library does
    /// not identify which position holds a given name or a family name. Each
    /// component is trimmed of surrounding whitespace. The iterator yields nothing
    /// when the field is absent, and a value holding no separator yields exactly
    /// one component. Use [`Self::formatted_name_in`] for a display string.
    pub fn name_components_in(&self, language: Language) -> impl Iterator<Item = &str> {
        self.name_in(language)
            .into_iter()
            .flat_map(|name| name.split(',').map(str::trim))
    }
    /// Returns the name in `language` formatted for display, without fallback.
    ///
    /// Comma separators become single spaces, empty positions are dropped, and the
    /// result is trimmed. Capitalization, spelling, diacritics, and component order
    /// are preserved. A field that is absent, or that holds only separators and
    /// whitespace, returns `None`, matching how the decoder treats an empty value.
    pub fn formatted_name_in(&self, language: Language) -> Option<String> {
        let mut formatted = String::new();
        for component in self
            .name_components_in(language)
            .filter(|component| !component.is_empty())
        {
            if !formatted.is_empty() {
                formatted.push(' ');
            }
            formatted.push_str(component);
        }
        (!formatted.is_empty()).then_some(formatted)
    }
    /// Returns the formatted English name, falling back to Arabic if English is
    /// absent. See [`Self::formatted_name_in`] for the formatting rules.
    ///
    /// The fallback is slightly wider than [`Self::name`]: an English field
    /// holding only separators has no formatted value, so Arabic is used, whereas
    /// [`Self::name`] would return the stored separators.
    pub fn formatted_name(&self) -> Option<String> {
        self.formatted_name_in(Language::English)
            .or_else(|| self.formatted_name_in(Language::Arabic))
    }
    /// Returns the Emirates ID number grouped as `784-YYYY-NNNNNNN-C`, the form
    /// printed on the card.
    ///
    /// Identifiers are validated when the snapshot is read or built.
    pub fn formatted_id_number(&self) -> String {
        let digits = self.id_number.as_str();
        if digits.len() != 15 || !digits.bytes().all(|byte| byte.is_ascii_digit()) {
            return digits.to_string();
        }
        format!(
            "{}-{}-{}-{}",
            &digits[..3],
            &digits[3..7],
            &digits[7..14],
            &digits[14..]
        )
    }
}

/// Gender interpreted from the code stored by the card.
///
/// Only the `M` and `F` codes are interpreted. Display labels such as `Male` or
/// `Ø°ÙƒØ±` are deliberately left to the application: they are translations rather
/// than card data, and the card's own `Sex` field prints the code. Use
/// [`Gender::code`] for the value the document shows.
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
#[non_exhaustive]
pub enum Gender {
    /// The card stores the `M` code.
    Male,
    /// The card stores the `F` code.
    Female,
    /// The card stores a code this library does not interpret, preserved as read.
    Unrecognized(String),
}

impl Gender {
    /// Interprets a stored gender code. Matching is ASCII case-insensitive, so
    /// both `m` and `M` yield [`Gender::Male`].
    pub fn from_code(code: &str) -> Self {
        if code.eq_ignore_ascii_case("M") {
            Self::Male
        } else if code.eq_ignore_ascii_case("F") {
            Self::Female
        } else {
            Self::Unrecognized(code.to_string())
        }
    }
    /// Returns the canonical uppercase code, matching the `Sex` field printed on
    /// the card. An unrecognized value is returned exactly as the card stored it,
    /// without case conversion.
    pub fn code(&self) -> &str {
        match self {
            Self::Male => "M",
            Self::Female => "F",
            Self::Unrecognized(code) => code,
        }
    }
}

/// Language selection for bilingual identity accessors.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Language {
    /// English card field.
    English,
    /// Arabic card field.
    Arabic,
}

/// Builds caller-supplied snapshots without accessing hardware.
/// Optional groups begin as `NotRequested`; setters mark populated groups as `Read`.
/// These statuses describe supplied data, not evidence of a hardware read.
#[derive(Debug)]
#[must_use = "call build to create the snapshot"]
pub struct EmiratesIdDataBuilder {
    data: EmiratesIdData,
}
impl EmiratesIdDataBuilder {
    /// Records a reader name without connecting to it.
    pub fn reader_name(mut self, value: impl Into<String>) -> Self {
        self.data.reader_name = value.into();
        self
    }
    /// Records a chip generation without detecting hardware.
    pub fn card_generation(mut self, value: CardGeneration) -> Self {
        self.data.card_generation = value;
        self
    }
    /// Supplies core identity fields.
    pub fn identity(mut self, value: NonModifiableData) -> Self {
        self.data.non_modifiable = value;
        self
    }
    /// Supplies extended fields and marks their group as read.
    pub fn extended(mut self, value: ModifiableData) -> Self {
        self.data.modifiable = value;
        self.data.read_status.modifiable = DataGroupStatus::Read;
        self
    }
    /// Supplies JPEG bytes and marks the photo group as read.
    pub fn photo(mut self, value: Vec<u8>) -> Self {
        self.data.photo_jpeg = Some(value);
        self.data.read_status.photo = DataGroupStatus::Read;
        self
    }
    /// Supplies opaque signature bytes and marks the signature group as read.
    pub fn signature(mut self, value: Vec<u8>) -> Self {
        self.data.holder_signature_image = Some(value);
        self.data.read_status.holder_signature_image = DataGroupStatus::Read;
        self
    }
    /// Records the result of a supported optional group. Required identity groups stay `Read`.
    /// Payloads for groups not marked `Read` are removed during `build`.
    pub fn optional_statuses(
        mut self,
        photo: DataGroupStatus,
        extended: DataGroupStatus,
        signature: DataGroupStatus,
    ) -> Self {
        self.data.read_status.photo = photo;
        self.data.read_status.modifiable = extended;
        self.data.read_status.holder_signature_image = signature;
        self
    }
    /// Validates caller-supplied data and returns an immutable snapshot.
    ///
    /// # Errors
    /// Returns `InvalidArgument` for identifiers of the wrong length or containing
    /// non-ASCII digits, invalid ISO calendar dates, or non-JPEG photo bytes.
    /// Other text and opaque signature bytes are preserved without normalization.
    pub fn build(mut self) -> Result<EmiratesIdData, crate::Error> {
        for (value, length) in [(&self.data.id_number, 15), (&self.data.card_number, 9)] {
            if value.len() != length || !value.bytes().all(|b| b.is_ascii_digit()) {
                return Err(crate::Error::new(
                    crate::ErrorKind::InvalidArgument,
                    "invalid identifier format",
                ));
            }
        }
        if self.data.read_status.photo != DataGroupStatus::Read {
            self.data.photo_jpeg = None;
        }
        if self.data.read_status.modifiable != DataGroupStatus::Read {
            self.data.modifiable = ModifiableData::default();
        }
        if self.data.read_status.holder_signature_image != DataGroupStatus::Read {
            self.data.holder_signature_image = None;
        }
        for date in [
            &self.data.non_modifiable.date_of_birth,
            &self.data.non_modifiable.issue_date,
            &self.data.non_modifiable.expiry_date,
            &self.data.modifiable.residency_expiry_date,
            &self.data.modifiable.passport_issue_date,
            &self.data.modifiable.passport_expiry_date,
            &self.data.modifiable.date_of_graduation,
        ]
        .into_iter()
        .flatten()
        {
            if !crate::decode::valid_iso_date(date) {
                return Err(crate::Error::new(
                    crate::ErrorKind::InvalidArgument,
                    "invalid ISO calendar date",
                ));
            }
        }
        if self
            .data
            .photo_jpeg
            .as_ref()
            .is_some_and(|bytes| !bytes.starts_with(&[0xFF, 0xD8, 0xFF]))
        {
            return Err(crate::Error::new(
                crate::ErrorKind::InvalidArgument,
                "photograph payload is not JPEG",
            ));
        }
        Ok(self.data)
    }
}

impl std::fmt::Debug for NonModifiableData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NonModifiableData")
            .field("fields", &"[REDACTED]")
            .finish()
    }
}

impl std::fmt::Debug for ModifiableData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ModifiableData")
            .field("fields", &"[REDACTED]")
            .finish()
    }
}

impl std::fmt::Debug for EmiratesIdData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EmiratesIdData")
            .field("card_generation", &self.card_generation)
            .field("read_status", &self.read_status)
            .field("personal_data", &"[REDACTED]")
            .field("photo_bytes", &self.photo_jpeg.as_ref().map(Vec::len))
            .field(
                "signature_bytes",
                &self.holder_signature_image.as_ref().map(Vec::len),
            )
            .finish()
    }
}
