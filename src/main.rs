use emirates_id_reader::{CardSession, ReadOptions};

fn report_optional(label: &str, value: Option<&str>) {
    match value {
        Some(value) => println!("{label}: present ({} characters)", value.chars().count()),
        None => println!("{label}: absent"),
    }
}

fn probe() -> Result<(), String> {
    let session = CardSession::connect_first().map_err(|error| error.to_string())?;
    println!("Reader: {}", session.reader_name());
    println!("Card: connected");
    println!("ATR: {}", session.atr_hex());
    Ok(())
}

fn read(redacted: bool, identity_only: bool) -> Result<(), String> {
    let session = CardSession::connect_first().map_err(|error| error.to_string())?;
    let card = if identity_only {
        session.read_with_options(ReadOptions::identity_only())
    } else {
        session.read()
    }
    .map_err(|error| error.to_string())?;
    println!("Reader: {}", card.reader_name);
    println!("Toolkit-free direct card read: successful");
    println!("Card generation: {:?}", card.card_generation);
    if redacted {
        println!(
            "ID number: read successfully ({} characters)",
            card.id_number.chars().count()
        );
        println!(
            "Card number: read successfully ({} characters)",
            card.card_number.chars().count()
        );
        let non_modifiable = &card.non_modifiable;
        report_optional("ID type", non_modifiable.id_type.as_deref());
        report_optional("Issue date", non_modifiable.issue_date.as_deref());
        report_optional("Expiry date", non_modifiable.expiry_date.as_deref());
        report_optional("Arabic title", non_modifiable.title_arabic.as_deref());
        report_optional(
            "Arabic full name",
            non_modifiable.full_name_arabic.as_deref(),
        );
        report_optional("English title", non_modifiable.title_english.as_deref());
        report_optional(
            "English full name",
            non_modifiable.full_name_english.as_deref(),
        );
        report_optional("Gender", non_modifiable.gender.as_deref());
        report_optional(
            "Arabic nationality",
            non_modifiable.nationality_arabic.as_deref(),
        );
        report_optional(
            "English nationality",
            non_modifiable.nationality_english.as_deref(),
        );
        report_optional(
            "Nationality code",
            non_modifiable.nationality_code.as_deref(),
        );
        report_optional("Date of birth", non_modifiable.date_of_birth.as_deref());
        report_optional(
            "Arabic place of birth",
            non_modifiable.place_of_birth_arabic.as_deref(),
        );
        report_optional(
            "English place of birth",
            non_modifiable.place_of_birth_english.as_deref(),
        );

        let modifiable = &card.modifiable;
        report_optional("Occupation code", modifiable.occupation_code.as_deref());
        report_optional("Arabic occupation", modifiable.occupation_arabic.as_deref());
        report_optional(
            "English occupation",
            modifiable.occupation_english.as_deref(),
        );
        report_optional("Family ID", modifiable.family_id.as_deref());
        report_optional(
            "Arabic occupation type",
            modifiable.occupation_type_arabic.as_deref(),
        );
        report_optional(
            "English occupation type",
            modifiable.occupation_type_english.as_deref(),
        );
        report_optional(
            "Occupation field code",
            modifiable.occupation_field_code.as_deref(),
        );
        report_optional(
            "Arabic company name",
            modifiable.company_name_arabic.as_deref(),
        );
        report_optional(
            "English company name",
            modifiable.company_name_english.as_deref(),
        );
        report_optional(
            "Marital status code",
            modifiable.marital_status_code.as_deref(),
        );
        report_optional("Husband ID number", modifiable.husband_id_number.as_deref());
        report_optional("Sponsor type code", modifiable.sponsor_type_code.as_deref());
        report_optional(
            "Sponsor unified number",
            modifiable.sponsor_unified_number.as_deref(),
        );
        report_optional("Sponsor name", modifiable.sponsor_name.as_deref());
        report_optional(
            "Residency type code",
            modifiable.residency_type_code.as_deref(),
        );
        report_optional("Residency number", modifiable.residency_number.as_deref());
        report_optional(
            "Residency expiry date",
            modifiable.residency_expiry_date.as_deref(),
        );
        report_optional("Passport number", modifiable.passport_number.as_deref());
        report_optional(
            "Passport type code",
            modifiable.passport_type_code.as_deref(),
        );
        report_optional(
            "Passport country code",
            modifiable.passport_country_code.as_deref(),
        );
        report_optional(
            "Arabic passport country",
            modifiable.passport_country_arabic.as_deref(),
        );
        report_optional(
            "English passport country",
            modifiable.passport_country_english.as_deref(),
        );
        report_optional(
            "Passport issue date",
            modifiable.passport_issue_date.as_deref(),
        );
        report_optional(
            "Passport expiry date",
            modifiable.passport_expiry_date.as_deref(),
        );
        report_optional(
            "Qualification level code",
            modifiable.qualification_level_code.as_deref(),
        );
        report_optional(
            "Arabic qualification level",
            modifiable.qualification_level_arabic.as_deref(),
        );
        report_optional(
            "English qualification level",
            modifiable.qualification_level_english.as_deref(),
        );
        report_optional(
            "Arabic degree description",
            modifiable.degree_description_arabic.as_deref(),
        );
        report_optional(
            "English degree description",
            modifiable.degree_description_english.as_deref(),
        );
        report_optional(
            "Field of study code",
            modifiable.field_of_study_code.as_deref(),
        );
        report_optional(
            "Arabic field of study",
            modifiable.field_of_study_arabic.as_deref(),
        );
        report_optional(
            "English field of study",
            modifiable.field_of_study_english.as_deref(),
        );
        report_optional(
            "Arabic place of study",
            modifiable.place_of_study_arabic.as_deref(),
        );
        report_optional(
            "English place of study",
            modifiable.place_of_study_english.as_deref(),
        );
        report_optional(
            "Date of graduation",
            modifiable.date_of_graduation.as_deref(),
        );
        report_optional(
            "Arabic mother name",
            modifiable.mother_full_name_arabic.as_deref(),
        );
        report_optional(
            "English mother name",
            modifiable.mother_full_name_english.as_deref(),
        );
        println!(
            "Photo: {} bytes",
            card.photo_jpeg.as_deref().unwrap_or(&[]).len()
        );
        println!(
            "Holder signature image: {} bytes",
            card.holder_signature_image.as_deref().unwrap_or(&[]).len()
        );
        println!("Group status: {:?}", card.read_status);
    } else {
        println!("ID number: {}", card.id_number);
        println!("Card number: {}", card.card_number);
        println!(
            "English full name: {}",
            card.non_modifiable
                .full_name_english
                .as_deref()
                .unwrap_or("")
        );
        println!(
            "Arabic full name: {}",
            card.non_modifiable
                .full_name_arabic
                .as_deref()
                .unwrap_or("")
        );
    }
    Ok(())
}

fn main() -> Result<(), String> {
    let mut args = std::env::args().skip(1);
    match args.next().as_deref() {
        None | Some("probe") => probe(),
        Some("read") => {
            let mut redacted = false;
            let mut identity_only = false;
            for argument in args {
                match argument.as_str() {
                    "--redacted" => redacted = true,
                    "--identity-only" => identity_only = true,
                    _ => return Err(format!("Unknown read option '{argument}'")),
                }
            }
            read(redacted, identity_only)
        }
        Some(command) => Err(format!(
            "Unknown command '{command}'. Use 'probe' or 'read [--redacted] [--identity-only]'."
        )),
    }
}
