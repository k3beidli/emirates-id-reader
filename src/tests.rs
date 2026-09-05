use crate::CardGeneration;
use crate::apdu::exchange_apdu;
use crate::decode::{
    bcd, date, decode_modifiable, decode_non_modifiable, field, required_ascii_digits, text,
};
use std::collections::VecDeque;

fn container(tag: u16, fields: &[(u16, &[u8])]) -> Vec<u8> {
    let payload_length = fields
        .iter()
        .map(|(_, value)| 4 + value.len())
        .sum::<usize>();
    let mut data = Vec::with_capacity(4 + payload_length);
    data.extend_from_slice(&tag.to_be_bytes());
    data.extend_from_slice(&(payload_length as u16).to_be_bytes());
    for (field_tag, value) in fields {
        data.extend_from_slice(&field_tag.to_be_bytes());
        data.extend_from_slice(&(value.len() as u16).to_be_bytes());
        data.extend_from_slice(value);
    }
    data
}

#[test]
fn parses_two_byte_tags_and_lengths() {
    let data = [
        0x70, 0x03, 0x00, 0x11, 0xE3, 0x0B, 0x00, 0x05, b'A', b'L', b'I', b'C', b'E', 0xA3, 0x09,
        0x00, 0x04, 0xD8, 0xA3, 0xD9, 0x85,
    ];
    assert_eq!(field(&data, 0xE30B).unwrap(), Some(&b"ALICE"[..]));
    assert_eq!(text(&data, 0xE30B).unwrap().as_deref(), Some("ALICE"));
}

#[test]
fn decodes_card_date_layout_to_iso() {
    let data = [
        0x70, 0x03, 0x00, 0x08, 0x43, 0x0F, 0x00, 0x04, 0x19, 0x95, 0x07, 0x28,
    ];
    assert_eq!(date(&data, 0x430F).unwrap().as_deref(), Some("1995-07-28"));
}

#[test]
fn decodes_packed_bcd_codes() {
    let data = [0x70, 0x05, 0x00, 0x05, 0x25, 0x15, 0x00, 0x01, 0x42];
    assert_eq!(bcd(&data, 0x2515).unwrap().as_deref(), Some("42"));
}

#[test]
fn validates_required_identifier_shape_without_echoing_the_value() {
    let valid = container(0x7001, &[(0xE101, b"784123456789012")]);
    assert_eq!(
        required_ascii_digits(&valid, 0xE101, "ID number", 15).unwrap(),
        "784123456789012"
    );

    let invalid = container(0x7001, &[(0xE101, b"784-NOT-AN-ID")]);
    let error = required_ascii_digits(&invalid, 0xE101, "ID number", 15).unwrap_err();
    assert!(!error.message.contains("784-NOT-AN-ID"));
}

#[test]
fn decodes_typed_modifiable_fields() {
    let data = container(
        0x7005,
        &[
            (0x2515, &[0x42]),
            (0xE53A, b"Engineer"),
            (0xE520, b"FAMILY01"),
            (0xE526, b"P1234567"),
            (0x452A, &[0x20, 0x31, 0x12, 0x09]),
        ],
    );
    let decoded = decode_modifiable(&data).unwrap();
    assert_eq!(decoded.occupation_code.as_deref(), Some("42"));
    assert_eq!(decoded.occupation_english.as_deref(), Some("Engineer"));
    assert_eq!(decoded.family_id.as_deref(), Some("FAMILY01"));
    assert_eq!(decoded.passport_number.as_deref(), Some("P1234567"));
    assert_eq!(decoded.passport_expiry_date.as_deref(), Some("2031-12-09"));
}

#[test]
fn decodes_v1_common_subset_without_v2_extensions() {
    let non_modifiable = container(
        0x7003,
        &[
            (0xE305, b"01"),
            (0x4306, &[0x20, 0x10, 0x01, 0x02]),
            (0x4307, &[0x20, 0x15, 0x01, 0x02]),
            (0xE30B, b"TEST HOLDER"),
            (0xE30C, b"F"),
            (0xE30E, b"ARE"),
            (0x430F, &[0x19, 0x90, 0x03, 0x04]),
        ],
    );
    let decoded = decode_non_modifiable(&non_modifiable).unwrap();
    assert_eq!(decoded.full_name_english.as_deref(), Some("TEST HOLDER"));
    assert_eq!(decoded.nationality_code.as_deref(), Some("ARE"));
    assert_eq!(decoded.date_of_birth.as_deref(), Some("1990-03-04"));
    assert_eq!(decoded.place_of_birth_english, None);

    let modifiable = container(
        0x7005,
        &[
            (0x2515, &[0x12, 0x34]),
            (0x2516, &[0x01]),
            (0x2518, &[0x02]),
            (0x2519, &[0x12, 0x34, 0x56]),
            (0xA51A, b"TEST SPONSOR"),
            (0x251B, &[0x03]),
            (0xE51C, b"RESIDENCY"),
            (0xE520, b"FAMILY"),
            (0xE511, b"MOTHER"),
        ],
    );
    let decoded = decode_modifiable(&modifiable).unwrap();
    assert_eq!(decoded.occupation_code.as_deref(), Some("1234"));
    assert_eq!(decoded.sponsor_name.as_deref(), Some("TEST SPONSOR"));
    assert_eq!(decoded.residency_number.as_deref(), Some("RESIDENCY"));
    assert_eq!(decoded.mother_full_name_english.as_deref(), Some("MOTHER"));
    assert_eq!(decoded.passport_number, None);
}

#[test]
fn recognizes_published_v1_and_v2_atrs() {
    assert_eq!(
        CardGeneration::from_atr(&[
            0x3B, 0x6A, 0x00, 0x00, 0x80, 0x65, 0xA2, 0x01, 0x31, 0x01, 0x3D, 0x72, 0xD6, 0x41,
        ]),
        CardGeneration::V1
    );
    assert_eq!(
        CardGeneration::from_atr(&[
            0x3B, 0x7A, 0x95, 0x00, 0x00, 0x80, 0x65, 0xA2, 0x01, 0x31, 0x01, 0x3D, 0x72, 0xD6,
            0x41,
        ]),
        CardGeneration::V2
    );
    assert_eq!(
        CardGeneration::from_atr(&[0x3B, 0x00]),
        CardGeneration::Unknown
    );
}

#[test]
fn follows_t0_get_response_continuation() {
    let mut replies = VecDeque::from([(vec![0x6F], 0x6102), (vec![0x84, 0x00], 0x9000)]);
    let mut commands = Vec::new();
    let data = exchange_apdu(&[0x00, 0xA4, 0x04, 0x00, 0x00], |command| {
        commands.push(command.to_vec());
        Ok(replies.pop_front().unwrap())
    })
    .unwrap();

    assert_eq!(data, vec![0x6F, 0x84, 0x00]);
    assert_eq!(commands[1], vec![0x00, 0xC0, 0x00, 0x00, 0x02]);
}

#[test]
fn corrects_case_four_response_length_without_overwriting_lc() {
    let mut replies = VecDeque::from([(Vec::new(), 0x6C10), (vec![0xAA], 0x9000)]);
    let mut commands = Vec::new();
    let data = exchange_apdu(
        &[0x00, 0xA4, 0x04, 0x00, 0x02, 0x01, 0x02, 0x00],
        |command| {
            commands.push(command.to_vec());
            Ok(replies.pop_front().unwrap())
        },
    )
    .unwrap();

    assert_eq!(data, vec![0xAA]);
    assert_eq!(commands[1][4], 0x02);
    assert_eq!(commands[1][7], 0x10);
}

#[test]
fn accepts_valid_data_returned_with_eof_warning() {
    let data = exchange_apdu(&[0x00, 0xB0, 0x00, 0x00, 0xFD], |_| {
        Ok((vec![0x70, 0x01, 0x00, 0x00], 0x6282))
    })
    .unwrap();
    assert_eq!(data, vec![0x70, 0x01, 0x00, 0x00]);
}
