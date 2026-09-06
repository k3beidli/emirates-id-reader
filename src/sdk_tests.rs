use crate::{CardGeneration, DataGroupStatus, Error, ErrorKind, Gender, Language, ReadOptions};
use std::collections::BTreeMap;

fn container(fields: &[(u16, &[u8])]) -> Vec<u8> {
    let mut data = vec![0x70, 0, 0, 0];
    for (tag, value) in fields {
        data.extend_from_slice(&tag.to_be_bytes());
        data.extend_from_slice(&(value.len() as u16).to_be_bytes());
        data.extend_from_slice(value);
    }
    let length = (data.len() - 4) as u16;
    data[2..4].copy_from_slice(&length.to_be_bytes());
    data
}

// Synthetic card implementing SELECT and READ BINARY. Never uses real card dumps.
struct FakeCard {
    files: BTreeMap<u16, Result<Vec<u8>, Error>>,
    selected: u16,
    selections: Vec<u16>,
    directory_error: Option<Error>,
    application_selections: usize,
    empty_after_first_chunk: bool,
}

impl FakeCard {
    fn new() -> Self {
        Self {
            files: BTreeMap::from([
                (
                    0x0201,
                    Ok(container(&[
                        (0xE101, b"000000000000000"),
                        (0xE102, b"000000000"),
                    ])),
                ),
                (
                    0x0203,
                    Ok(container(&[
                        (0xE30B, b"SYNTHETIC,TEST,,HOLDER"),
                        (0xA309, "اسم,تجريبي".as_bytes()),
                        (0xE30C, b"M"),
                    ])),
                ),
                (
                    0x0202,
                    Ok(container(&[(0x6203, &[0xFF, 0xD8, 0xFF, 0xD9])])),
                ),
                (0x0205, Ok(container(&[(0xE526, b"SYNTHETIC")]))),
                (0x0207, Ok(container(&[(0x6732, &[1, 2, 3])]))),
            ]),
            selected: 0,
            selections: vec![],
            directory_error: None,
            application_selections: 0,
            empty_after_first_chunk: false,
        }
    }

    fn exchange(&mut self, command: &[u8]) -> Result<Vec<u8>, Error> {
        match command[1] {
            0xA4 if command[2] == 4 => {
                self.application_selections += 1;
                Ok(vec![])
            }
            0xA4 => {
                let file = u16::from_be_bytes([command[5], command[6]]);
                self.selections.push(file);
                if file == 0x0200 {
                    return self.directory_error.clone().map_or(Ok(vec![]), Err);
                }
                match self.files.get(&file) {
                    Some(Ok(_)) => {
                        self.selected = file;
                        Ok(vec![])
                    }
                    Some(Err(error)) => Err(error.clone()),
                    None => Err(Error::apdu(0x6A82)),
                }
            }
            0xB0 => {
                let offset = u16::from_be_bytes([command[2], command[3]]) as usize;
                if self.empty_after_first_chunk && offset > 0 {
                    return Ok(vec![]);
                }
                let file = self.files[&self.selected].as_ref().unwrap();
                let end = (offset + command[4] as usize).min(file.len());
                assert!(offset <= end, "offset past EOF");
                Ok(file[offset..end].to_vec())
            }
            _ => panic!("Unexpected APDU"),
        }
    }

    fn read(
        &mut self,
        options: ReadOptions,
        generation: CardGeneration,
    ) -> Result<crate::EmiratesIdData, Error> {
        crate::protocol::Reader {
            exchange: |command: &[u8]| self.exchange(command),
        }
        .read("Synthetic reader", generation, options)
    }
}

#[test]
fn full_read_and_borrowed_accessors_work_for_all_generation_classifications() {
    for generation in [
        CardGeneration::V1,
        CardGeneration::V2,
        CardGeneration::Unknown,
    ] {
        let data = FakeCard::new()
            .read(ReadOptions::all(), generation)
            .unwrap();
        assert_eq!(data.card_generation, generation);
        // The raw accessors keep returning the stored value, separators included.
        assert_eq!(data.get_name(), Some("SYNTHETIC,TEST,,HOLDER"));
        assert_eq!(data.get_name_in(Language::Arabic), Some("اسم,تجريبي"));
        assert_eq!(data.get_gender(), Some("M"));
        assert_eq!(
            data.identity().full_name_english.as_deref(),
            Some("SYNTHETIC,TEST,,HOLDER")
        );
        assert_eq!(
            data.get_formatted_name().as_deref(),
            Some("SYNTHETIC TEST HOLDER")
        );
        assert_eq!(data.gender(), Some(Gender::Male));
        assert_eq!(data.get_photo(), Some(&[0xFF, 0xD8, 0xFF, 0xD9][..]));
        assert_eq!(data.get_signature(), Some(&[1, 2, 3][..]));
        assert_eq!(data.get_id_number(), "000000000000000");
        assert_eq!(
            data.extended().passport_number.as_deref(),
            Some("SYNTHETIC")
        );
        assert_eq!(data.read_status.photo, DataGroupStatus::Read);
        assert_eq!(
            data.get_photo().unwrap().as_ptr(),
            data.photo_jpeg.as_ref().unwrap().as_ptr()
        );
    }
}

#[test]
fn name_fallback_does_not_affect_language_specific_lookup() {
    let mut card = FakeCard::new();
    card.files
        .insert(0x0203, Ok(container(&[(0xA309, "اسم,تجريبي".as_bytes())])));
    let data = card
        .read(ReadOptions::identity_only(), CardGeneration::V1)
        .unwrap();
    assert_eq!(data.get_name(), Some("اسم,تجريبي"));
    assert_eq!(data.get_name_in(Language::English), None);
    assert_eq!(data.get_formatted_name().as_deref(), Some("اسم تجريبي"));
    assert_eq!(data.get_formatted_name_in(Language::English), None);
    assert_eq!(data.name_components_in(Language::English).count(), 0);
}

#[test]
fn name_components_preserve_empty_positions_that_formatting_drops() {
    let data = FakeCard::new()
        .read(ReadOptions::identity_only(), CardGeneration::V1)
        .unwrap();
    assert_eq!(
        data.name_components_in(Language::English)
            .collect::<Vec<_>>(),
        ["SYNTHETIC", "TEST", "", "HOLDER"]
    );
    assert_eq!(
        data.name_components_in(Language::Arabic)
            .collect::<Vec<_>>(),
        ["اسم", "تجريبي"]
    );
    assert_eq!(
        data.get_formatted_name_in(Language::English).as_deref(),
        Some("SYNTHETIC TEST HOLDER")
    );
    assert_eq!(
        data.get_formatted_name_in(Language::Arabic).as_deref(),
        Some("اسم تجريبي")
    );
}

#[test]
fn separator_only_names_keep_positions_but_have_no_formatted_value() {
    let mut card = FakeCard::new();
    card.files.insert(
        0x0203,
        Ok(container(&[
            (0xE30B, b" , ,, "),
            (0xA309, "اسم,تجريبي".as_bytes()),
        ])),
    );
    let data = card
        .read(ReadOptions::identity_only(), CardGeneration::V1)
        .unwrap();
    // The decoder trims the stored value; the separators it keeps are the structure.
    assert_eq!(data.get_name_in(Language::English), Some(", ,,"));
    assert_eq!(
        data.name_components_in(Language::English)
            .collect::<Vec<_>>(),
        ["", "", "", ""]
    );
    assert_eq!(data.get_formatted_name_in(Language::English), None);
    // The wider fallback: no usable English value, so Arabic is formatted instead.
    assert_eq!(data.get_formatted_name().as_deref(), Some("اسم تجريبي"));
}

#[test]
fn names_without_separators_yield_a_single_component() {
    let mut card = FakeCard::new();
    card.files
        .insert(0x0203, Ok(container(&[(0xE30B, b"SYNTHETIC HOLDER")])));
    let data = card
        .read(ReadOptions::identity_only(), CardGeneration::V1)
        .unwrap();
    assert_eq!(
        data.name_components_in(Language::English)
            .collect::<Vec<_>>(),
        ["SYNTHETIC HOLDER"]
    );
    assert_eq!(
        data.get_formatted_name().as_deref(),
        Some("SYNTHETIC HOLDER")
    );
}

#[test]
fn gender_codes_are_interpreted_without_discarding_unknown_values() {
    assert_eq!(Gender::from_code("M"), Gender::Male);
    assert_eq!(Gender::from_code("m"), Gender::Male);
    assert_eq!(Gender::from_code("F"), Gender::Female);
    assert_eq!(Gender::from_code("f"), Gender::Female);
    assert_eq!(Gender::Male.code(), "M");
    assert_eq!(Gender::Female.code(), "F");
    // A lowercase code is interpreted, but code() reports the canonical form.
    assert_eq!(Gender::from_code("m").code(), "M");
    // An unknown code is preserved verbatim, never collapsed into None.
    assert_eq!(
        Gender::from_code("X"),
        Gender::Unrecognized(String::from("X"))
    );
    assert_eq!(Gender::from_code("X").code(), "X");

    let mut card = FakeCard::new();
    card.files.insert(
        0x0203,
        Ok(container(&[(0xE30B, b"SYNTHETIC"), (0xE30C, b"X")])),
    );
    let data = card
        .read(ReadOptions::identity_only(), CardGeneration::V1)
        .unwrap();
    assert_eq!(data.get_gender(), Some("X"));
    assert_eq!(data.gender(), Some(Gender::Unrecognized(String::from("X"))));

    let mut card = FakeCard::new();
    card.files
        .insert(0x0203, Ok(container(&[(0xE30B, b"SYNTHETIC")])));
    let data = card
        .read(ReadOptions::identity_only(), CardGeneration::V1)
        .unwrap();
    assert_eq!(data.get_gender(), None);
    assert_eq!(data.gender(), None);
}

#[test]
fn id_number_formatting_groups_valid_digits_and_passes_other_values_through() {
    let mut data = FakeCard::new()
        .read(ReadOptions::identity_only(), CardGeneration::V1)
        .unwrap();
    assert_eq!(data.get_id_number(), "000000000000000");
    assert_eq!(data.formatted_id_number(), "000-0000-0000000-0");
    data.id_number = String::from("784198512345671");
    assert_eq!(data.formatted_id_number(), "784-1985-1234567-1");
    // The field is public, so the formatter must survive values a read cannot produce.
    for replacement in ["", "7841985", "78419851234567", "7841985123456712", "abc"] {
        data.id_number = String::from(replacement);
        assert_eq!(data.formatted_id_number(), replacement);
    }
    data.id_number = String::from("٧٨٤١٩٨٥١٢٣٤٥٦٧١");
    assert_eq!(data.formatted_id_number(), "٧٨٤١٩٨٥١٢٣٤٥٦٧١");
}

#[test]
fn read_options_select_only_requested_files() {
    let mut fake = FakeCard::new();
    let data = fake
        .read(ReadOptions::identity_only(), CardGeneration::V1)
        .unwrap();
    assert_eq!(fake.selections, [0x0200, 0x0201, 0x0203]);
    assert_eq!(data.get_photo(), None);
    assert_eq!(data.read_status.photo, DataGroupStatus::NotRequested);
    assert_eq!(data.read_status.modifiable, DataGroupStatus::NotRequested);
    assert_eq!(
        data.read_status.holder_signature_image,
        DataGroupStatus::NotRequested
    );
    let mut fake = FakeCard::new();
    fake.read(
        ReadOptions::identity_only().with_photo(true),
        CardGeneration::V2,
    )
    .unwrap();
    assert_eq!(fake.selections, [0x0200, 0x0201, 0x0203, 0x0202]);
    let mut fake = FakeCard::new();
    fake.read(
        ReadOptions::all()
            .with_photo(false)
            .with_modifiable_data(false)
            .with_holder_signature_image(false),
        CardGeneration::V2,
    )
    .unwrap();
    assert_eq!(fake.selections, [0x0200, 0x0201, 0x0203]);
}

#[test]
fn optional_statuses_are_preserved_but_transport_errors_fail_the_read() {
    for (status, expected) in [
        (0x6982, DataGroupStatus::Protected),
        (0x6985, DataGroupStatus::Protected),
        (0x6A82, DataGroupStatus::NotAvailable),
        (0x6A83, DataGroupStatus::NotAvailable),
    ] {
        let mut fake = FakeCard::new();
        for file in [0x0202, 0x0205, 0x0207] {
            fake.files.insert(file, Err(Error::apdu(status)));
        }
        let data = fake.read(ReadOptions::all(), CardGeneration::V2).unwrap();
        assert_eq!(data.read_status.photo, expected);
        assert_eq!(data.read_status.modifiable, expected);
        assert_eq!(data.read_status.holder_signature_image, expected);
    }
    let mut fake = FakeCard::new();
    fake.files
        .insert(0x0202, Err(Error::new(ErrorKind::CardRemoved, "Removed")));
    assert_eq!(
        fake.read(ReadOptions::all(), CardGeneration::V2)
            .unwrap_err()
            .kind,
        ErrorKind::CardRemoved
    );
}

#[test]
fn required_file_errors_are_never_downgraded_to_optional_status() {
    let mut fake = FakeCard::new();
    fake.files.insert(0x0203, Err(Error::apdu(0x6982)));
    assert_eq!(
        fake.read(ReadOptions::all(), CardGeneration::V1)
            .unwrap_err()
            .status_word,
        Some(0x6982)
    );
    assert!(!fake.selections.contains(&0x0202));
}

#[test]
fn falls_back_to_application_root_only_when_directory_is_absent() {
    let mut fake = FakeCard::new();
    fake.directory_error = Some(Error::apdu(0x6A82));
    fake.read(ReadOptions::identity_only(), CardGeneration::V1)
        .unwrap();
    assert_eq!(fake.application_selections, 2);
    for error in [
        Error::apdu(0x6982),
        Error::new(ErrorKind::CardRemoved, "Removed"),
    ] {
        let mut fake = FakeCard::new();
        fake.directory_error = Some(error.clone());
        assert_eq!(
            fake.read(ReadOptions::all(), CardGeneration::V2)
                .unwrap_err()
                .kind,
            error.kind
        );
        assert_eq!(fake.application_selections, 1);
    }
}

#[test]
fn malformed_images_fail_but_empty_fields_are_absent() {
    for payload in [
        vec![0x70, 0x02, 0, 4, 0x62],
        container(&[(0x6203, b"not jpeg")]),
    ] {
        let mut fake = FakeCard::new();
        fake.files.insert(0x0202, Ok(payload));
        assert_eq!(
            fake.read(ReadOptions::all(), CardGeneration::V2)
                .unwrap_err()
                .kind,
            ErrorKind::InvalidData
        );
    }
    let mut fake = FakeCard::new();
    fake.files.insert(0x0202, Ok(container(&[(0x6203, &[])])));
    let data = fake.read(ReadOptions::all(), CardGeneration::V2).unwrap();
    assert_eq!(data.get_photo(), None);
    assert_eq!(data.read_status.photo, DataGroupStatus::Read);
    fake.files.insert(0x0207, Ok(vec![0x70, 7, 0, 4, 0x67]));
    assert!(fake.read(ReadOptions::all(), CardGeneration::V2).is_err());
}

#[test]
fn multi_chunk_photos_empty_chunks_and_oversized_files() {
    let mut photo = vec![0xAB; 700];
    photo[..3].copy_from_slice(&[0xFF, 0xD8, 0xFF]);
    let mut fake = FakeCard::new();
    fake.files
        .insert(0x0202, Ok(container(&[(0x6203, &photo)])));
    assert_eq!(
        fake.read(ReadOptions::all(), CardGeneration::V2)
            .unwrap()
            .get_photo(),
        Some(photo.as_slice())
    );
    fake.empty_after_first_chunk = true;
    assert_eq!(
        fake.read(ReadOptions::all(), CardGeneration::V2)
            .unwrap_err()
            .kind,
        ErrorKind::InvalidData
    );
    fake.files.insert(0x0202, Ok(vec![0x70, 0x02, 0xFF, 0xFF]));
    assert_eq!(
        fake.read(ReadOptions::all(), CardGeneration::V2)
            .unwrap_err()
            .kind,
        ErrorKind::InvalidData
    );
}

#[test]
fn rejects_trailing_corruption_and_duplicate_requested_fields() {
    let mut bad = container(&[(0xE30B, b"SYNTHETIC")]);
    bad.push(0xFF);
    let length = (bad.len() - 4) as u16;
    bad[2..4].copy_from_slice(&length.to_be_bytes());
    assert_eq!(
        crate::decode::field(&bad, 0xE30B).unwrap_err().kind,
        ErrorKind::InvalidData
    );
    assert!(
        crate::decode::field(&container(&[(0xE30B, b"ONE"), (0xE30B, b"TWO")]), 0xE30B).is_err()
    );
}

#[test]
fn validates_calendar_dates_including_century_leap_years() {
    for bytes in [
        [0x20, 0x23, 0x02, 0x29],
        [0x20, 0x24, 0x13, 0x01],
        [0x20, 0x24, 0x04, 0x31],
        [0x19, 0x00, 0x02, 0x29],
        [0x20, 0x24, 0x01, 0x00],
    ] {
        assert!(crate::decode::date(&container(&[(0x430F, &bytes)]), 0x430F).is_err());
    }
    assert_eq!(
        crate::decode::date(&container(&[(0x430F, &[0x20, 0x00, 0x02, 0x29])]), 0x430F)
            .unwrap()
            .as_deref(),
        Some("2000-02-29")
    );
}

#[test]
fn rejects_invalid_utf8_bcd_and_truncated_containers() {
    assert!(crate::decode::text(&container(&[(0xE30B, &[0xFF])]), 0xE30B).is_err());
    assert!(crate::decode::bcd(&container(&[(0x2515, &[0xAF])]), 0x2515).is_err());
    for bytes in [
        &[][..],
        &[0x70, 0x03, 0x00],
        &[0x70, 0x03, 0x00, 0x04, 0xE3],
    ] {
        assert!(crate::decode::field(bytes, 0xE30B).is_err());
    }
}

#[test]
fn limits_length_correction_and_response_chaining() {
    let mut count = 0;
    let error = crate::apdu::exchange_apdu(&[0, 0xB0, 0, 0, 1], |_| {
        count += 1;
        Ok((vec![], 0x6C10))
    })
    .unwrap_err();
    assert_eq!(count, 2);
    assert_eq!(error.status_word, Some(0x6C10));
    count = 0;
    assert!(
        crate::apdu::exchange_apdu(&[0, 0xB0, 0, 0, 1], |_| {
            count += 1;
            Ok((vec![], 0x6101))
        })
        .is_err()
    );
    assert_eq!(count, 32);
}

#[test]
fn invalid_reader_names_are_rejected_before_touching_pcsc() {
    for name in ["", "reader\0suffix"] {
        assert_eq!(
            crate::CardSession::connect(name).err().unwrap().kind,
            ErrorKind::InvalidArgument
        );
    }
}

#[test]
fn pcsc_errors_keep_machine_readable_kinds() {
    for (code, expected) in [
        (0x8010002Eu32, ErrorKind::NoReader),
        (0x8010000C, ErrorKind::NoCard),
        (0x80100068, ErrorKind::CardRemoved),
        (0x80100069, ErrorKind::CardRemoved),
        (0x80100017, ErrorKind::CardRemoved),
        (0x8010000B, ErrorKind::Pcsc),
    ] {
        assert_eq!(Error::pcsc("Synthetic", code as i32).kind, expected);
    }
}
