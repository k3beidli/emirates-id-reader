//! Decode the card public-data TLV format.

use crate::{Error, ErrorKind, ModifiableData, NonModifiableData};

pub(crate) fn field(data: &[u8], wanted_tag: u16) -> Result<Option<&[u8]>, Error> {
    if data.len() < 4 {
        return Err(Error::new(
            ErrorKind::InvalidData,
            "Truncated card container",
        ));
    }
    let end = 4 + u16::from_be_bytes([data[2], data[3]]) as usize;
    if end > data.len() {
        return Err(Error::new(
            ErrorKind::InvalidData,
            "Card container exceeds available data",
        ));
    }
    let mut found = None;
    let mut offset = 4;
    while offset < end {
        if offset + 4 > end {
            return Err(Error::new(ErrorKind::InvalidData, "Truncated field header"));
        }
        let tag = u16::from_be_bytes([data[offset], data[offset + 1]]);
        let length = u16::from_be_bytes([data[offset + 2], data[offset + 3]]) as usize;
        let value_start = offset + 4;
        let value_end = value_start
            .checked_add(length)
            .ok_or_else(|| Error::new(ErrorKind::InvalidData, "Field length overflow"))?;
        if value_end > end {
            return Err(Error::new(
                ErrorKind::InvalidData,
                format!("Field {tag:04X} is truncated"),
            ));
        }
        if tag == wanted_tag {
            if found.is_some() {
                return Err(Error::new(
                    ErrorKind::InvalidData,
                    format!("Duplicate field {tag:04X}"),
                ));
            }
            found = Some(&data[value_start..value_end]);
        }
        offset = value_end;
    }
    Ok(found)
}

pub(crate) fn text(data: &[u8], tag: u16) -> Result<Option<String>, Error> {
    let Some(value) = field(data, tag)? else {
        return Ok(None);
    };
    let value = std::str::from_utf8(value)
        .map_err(|_| {
            Error::new(
                ErrorKind::InvalidData,
                format!("Field {tag:04X} is not valid UTF-8"),
            )
        })?
        .trim_matches('\0')
        .trim();
    Ok((!value.is_empty()).then(|| value.to_string()))
}

fn required_text(data: &[u8], tag: u16, label: &str) -> Result<String, Error> {
    text(data, tag)?.ok_or_else(|| {
        Error::new(
            ErrorKind::InvalidData,
            format!("Required {label} field {tag:04X} is missing"),
        )
    })
}

pub(crate) fn required_ascii_digits(
    data: &[u8],
    tag: u16,
    label: &str,
    expected_length: usize,
) -> Result<String, Error> {
    let value = required_text(data, tag, label)?;
    if value.len() != expected_length || !value.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err(Error::new(
            ErrorKind::InvalidData,
            format!("{label} field {tag:04X} has an invalid format"),
        ));
    }
    Ok(value)
}

pub(crate) fn bcd(data: &[u8], tag: u16) -> Result<Option<String>, Error> {
    let Some(value) = field(data, tag)? else {
        return Ok(None);
    };
    if value.is_empty() || value.iter().all(|byte| *byte == 0 || *byte == 0xFF) {
        return Ok(None);
    }
    let mut result = String::with_capacity(value.len() * 2);
    for byte in value {
        let high = byte >> 4;
        let low = byte & 0x0F;
        if high > 9 || low > 9 {
            return Err(Error::new(
                ErrorKind::InvalidData,
                format!("Field {tag:04X} is not packed BCD"),
            ));
        }
        result.push(char::from(b'0' + high));
        result.push(char::from(b'0' + low));
    }
    Ok(Some(result))
}

pub(crate) fn date(data: &[u8], tag: u16) -> Result<Option<String>, Error> {
    let Some(value) = field(data, tag)? else {
        return Ok(None);
    };
    if value.is_empty() || value.iter().all(|byte| *byte == 0 || *byte == 0xFF) {
        return Ok(None);
    }
    if value.len() != 4 {
        return Err(Error::new(
            ErrorKind::InvalidData,
            format!("Date {tag:04X} is not four bytes"),
        ));
    }
    let digits = value
        .iter()
        .map(|byte| {
            let high = byte >> 4;
            let low = byte & 0x0F;
            (high <= 9 && low <= 9).then(|| format!("{high}{low}"))
        })
        .collect::<Option<Vec<_>>>()
        .ok_or_else(|| {
            Error::new(
                ErrorKind::InvalidData,
                format!("Date {tag:04X} is not packed BCD"),
            )
        })?;
    let year: u16 = format!("{}{}", digits[0], digits[1]).parse().unwrap();
    let month: u8 = digits[2].parse().unwrap();
    let day: u8 = digits[3].parse().unwrap();
    let leap = year % 4 == 0 && (year % 100 != 0 || year % 400 == 0);
    let days = match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if leap => 29,
        2 => 28,
        _ => 0,
    };
    if year == 0 || day == 0 || day > days {
        return Err(Error::new(
            ErrorKind::InvalidData,
            format!("Date {tag:04X} is not a calendar date"),
        ));
    }
    Ok(Some(format!("{year:04}-{month:02}-{day:02}")))
}

pub(crate) fn decode_non_modifiable(data: &[u8]) -> Result<NonModifiableData, Error> {
    Ok(NonModifiableData {
        id_type: text(data, 0xE305)?,
        issue_date: date(data, 0x4306)?,
        expiry_date: date(data, 0x4307)?,
        title_arabic: text(data, 0xA308)?,
        full_name_arabic: text(data, 0xA309)?,
        title_english: text(data, 0xE30A)?,
        full_name_english: text(data, 0xE30B)?,
        gender: text(data, 0xE30C)?,
        nationality_arabic: text(data, 0xA30D)?,
        nationality_english: text(data, 0xE336)?,
        nationality_code: text(data, 0xE30E)?,
        date_of_birth: date(data, 0x430F)?,
        place_of_birth_arabic: text(data, 0xA337)?,
        place_of_birth_english: text(data, 0xE338)?,
    })
}

pub(crate) fn decode_modifiable(data: &[u8]) -> Result<ModifiableData, Error> {
    Ok(ModifiableData {
        occupation_code: bcd(data, 0x2515)?,
        occupation_arabic: text(data, 0xA539)?,
        occupation_english: text(data, 0xE53A)?,
        family_id: text(data, 0xE520)?,
        occupation_type_arabic: text(data, 0xA521)?,
        occupation_type_english: text(data, 0xE522)?,
        occupation_field_code: bcd(data, 0x2523)?,
        company_name_arabic: text(data, 0xA524)?,
        company_name_english: text(data, 0xE525)?,
        marital_status_code: bcd(data, 0x2516)?,
        husband_id_number: text(data, 0xE517)?,
        sponsor_type_code: bcd(data, 0x2518)?,
        sponsor_unified_number: bcd(data, 0x2519)?,
        sponsor_name: text(data, 0xA51A)?,
        residency_type_code: bcd(data, 0x251B)?,
        residency_number: text(data, 0xE51C)?,
        residency_expiry_date: date(data, 0x451D)?,
        passport_number: text(data, 0xE526)?,
        passport_type_code: bcd(data, 0x2527)?,
        passport_country_code: text(data, 0xE528)?,
        passport_country_arabic: text(data, 0xA53B)?,
        passport_country_english: text(data, 0xE53C)?,
        passport_issue_date: date(data, 0x4529)?,
        passport_expiry_date: date(data, 0x452A)?,
        qualification_level_code: bcd(data, 0x252B)?,
        qualification_level_arabic: text(data, 0xA53D)?,
        qualification_level_english: text(data, 0xE53E)?,
        degree_description_arabic: text(data, 0xA52C)?,
        degree_description_english: text(data, 0xE52D)?,
        field_of_study_code: bcd(data, 0x252E)?,
        field_of_study_arabic: text(data, 0xA53F)?,
        field_of_study_english: text(data, 0xE540)?,
        place_of_study_arabic: text(data, 0xA52F)?,
        place_of_study_english: text(data, 0xE530)?,
        date_of_graduation: date(data, 0x4531)?,
        mother_full_name_arabic: text(data, 0xA510)?,
        mother_full_name_english: text(data, 0xE511)?,
    })
}
