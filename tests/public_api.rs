use emirates_id_reader::{
    CardSession, DataGroupStatus, EmiratesIdData, Error, ErrorKind, ModifiableData,
    NonModifiableData, ReadOptions,
};

fn snapshot() -> EmiratesIdData {
    let mut identity = NonModifiableData::default();
    identity.full_name_english = Some("SYNTHETIC,, HOLDER".into());
    identity.date_of_birth = Some("2000-02-29".into());
    identity.gender = Some("m".into());
    let mut extended = ModifiableData::default();
    extended.passport_number = Some("TEST-PASSPORT".into());
    EmiratesIdData::builder("784200000000001", "123456789")
        .reader_name("SYNTHETIC-READER")
        .identity(identity)
        .extended(extended)
        .photo(vec![0xFF, 0xD8, 0xFF, 1])
        .signature(b"TEST-SIGNATURE".to_vec())
        .build()
        .unwrap()
}

#[test]
fn defaults_do_not_request_optional_groups() {
    assert_eq!(ReadOptions::default(), ReadOptions::identity_only());
    assert_ne!(ReadOptions::default(), ReadOptions::all());
}

#[test]
fn public_builder_preserves_raw_values_and_borrowed_access() {
    let card = snapshot();
    assert_eq!(card.name(), Some("SYNTHETIC,, HOLDER"));
    assert_eq!(card.formatted_name().as_deref(), Some("SYNTHETIC HOLDER"));
    assert_eq!(card.gender_code(), Some("m"));
    assert_eq!(card.id_number(), "784200000000001");
    assert_eq!(card.formatted_id_number(), "784-2000-0000000-1");
    assert_eq!(card.read_status().modifiable, DataGroupStatus::Read);
    assert_eq!(
        card.extended().passport_number.as_deref(),
        Some("TEST-PASSPORT")
    );
}

#[test]
fn debug_redacts_nested_records_and_payloads() {
    let card = snapshot();
    for text in [
        format!("{card:?}"),
        format!("{card:#?}"),
        format!("{:?}", card.identity()),
        format!("{:?}", card.extended()),
    ] {
        assert!(text.contains("REDACTED"));
        for secret in [
            "SYNTHETIC",
            "HOLDER",
            "784200000000001",
            "123456789",
            "TEST-PASSPORT",
            "TEST-SIGNATURE",
            "2000-02-29",
            "255",
        ] {
            assert!(!text.contains(secret), "debug exposed a synthetic secret");
        }
    }
    assert!(
        !format!(
            "{:?}",
            EmiratesIdData::builder("784200000000001", "123456789")
        )
        .contains("784200000000001")
    );
}

#[test]
fn public_builder_rejects_invalid_identifiers_dates_and_photos() {
    for (id, serial) in [
        ("invalid", "123456789"),
        ("784200000000001", "123"),
        ("٧84200000000001", "123456789"),
    ] {
        assert_eq!(
            EmiratesIdData::builder(id, serial)
                .build()
                .unwrap_err()
                .kind,
            ErrorKind::InvalidArgument
        );
    }
    for date in [
        "1900-02-29",
        "2000-04-31",
        "0000-01-01",
        "2000-13-01",
        "2000-02-00",
        "2000-2-29",
        "💥2000-01",
    ] {
        let mut identity = NonModifiableData::default();
        identity.issue_date = Some(date.into());
        assert!(
            EmiratesIdData::builder("784200000000001", "123456789")
                .identity(identity)
                .build()
                .is_err()
        );
        let mut extended = ModifiableData::default();
        extended.passport_expiry_date = Some(date.into());
        assert!(
            EmiratesIdData::builder("784200000000001", "123456789")
                .extended(extended)
                .build()
                .is_err()
        );
    }
    assert!(
        EmiratesIdData::builder("784200000000001", "123456789")
            .photo(vec![1, 2, 3])
            .build()
            .is_err()
    );
}

#[test]
fn disabled_builder_groups_cannot_retain_supplied_payloads() {
    let mut extended = ModifiableData::default();
    extended.passport_number = Some("TEST-PASSPORT".into());
    let card = EmiratesIdData::builder("784200000000001", "123456789")
        .photo(vec![1])
        .signature(vec![2])
        .extended(extended)
        .optional_statuses(
            DataGroupStatus::Protected,
            DataGroupStatus::NotRequested,
            DataGroupStatus::NotAvailable,
        )
        .build()
        .unwrap();
    assert_eq!(card.photo(), None);
    assert_eq!(card.signature(), None);
    assert_eq!(card.extended().passport_number, None);
    assert_eq!(card.read_status().photo, DataGroupStatus::Protected);
}

#[test]
fn public_types_remain_send_sync_and_debug() {
    fn check<T: Send + Sync + std::fmt::Debug>() {}
    check::<CardSession>();
    check::<EmiratesIdData>();
    check::<Error>();
}

#[cfg(feature = "serde")]
#[test]
fn serialization_keeps_snapshot_shape_and_personal_values_explicit() {
    let value = serde_json::to_value(snapshot()).unwrap();
    assert_eq!(
        value["nonModifiable"]["fullNameEnglish"],
        "SYNTHETIC,, HOLDER"
    );
    assert_eq!(value["nonModifiable"]["gender"], "m");
    assert_eq!(value["idNumber"], "784200000000001");
    assert_eq!(value["readStatus"]["photo"], "read");
    assert_eq!(value["photoJpeg"], serde_json::json!([255, 216, 255, 1]));
    assert!(value.get("formattedName").is_none());
    assert_eq!(value.as_object().unwrap().len(), 9);
}
