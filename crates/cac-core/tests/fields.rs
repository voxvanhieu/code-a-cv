//! Contract tests for the field table, open fields, and the core extension
//! types: photo, headline, address, skill level, language, reference, score.
//!
//! These types are the C1 contract: parsers, writers, and renderers code
//! against them in later tasks, so the serde shapes here are load-bearing.

use cac_core::{
    Address, CORE_FIELDS, CvDocument, EducationEntry, EntryKind, FieldMap, FieldParser, FieldScope,
    FieldValue, LanguageEntry, Profile, ReferenceEntry, Skill, SkillLevel, core_field,
};
use serde_json::json;

#[test]
fn profile_accepts_and_round_trips_extension_fields() {
    let document: CvDocument = serde_json::from_value(json!({
        "profile": {
            "name": "Ada Lovelace",
            "photo": "photo.jpg",
            "headline": "Analytical engines specialist",
            "address": {
                "street": "1977 Coleman Blvd",
                "city": "Vancouver",
                "region": "BC",
                "postal_code": "V6J 2H5",
                "country": "Canada"
            }
        }
    }))
    .unwrap();
    let profile = &document.profile;
    assert_eq!(profile.photo.as_deref(), Some("photo.jpg"));
    assert_eq!(
        profile.headline.as_deref(),
        Some("Analytical engines specialist")
    );
    assert_eq!(
        profile.address,
        Some(Address {
            street: Some("1977 Coleman Blvd".into()),
            city: Some("Vancouver".into()),
            region: Some("BC".into()),
            postal_code: Some("V6J 2H5".into()),
            country: Some("Canada".into()),
        })
    );

    // Absent extension fields serialize away, so legacy output is unchanged.
    let minimal: Profile = serde_json::from_value(json!({ "name": "Ada Lovelace" })).unwrap();
    let value = serde_json::to_value(&minimal).unwrap();
    assert_eq!(value, json!({ "name": "Ada Lovelace" }));
}

#[test]
fn partial_address_round_trips() {
    let document: CvDocument = serde_json::from_value(json!({
        "profile": {
            "name": "Ada Lovelace",
            "address": { "city": "Tokyo", "country": "Japan" }
        }
    }))
    .unwrap();
    let address = document.profile.address.as_ref().unwrap();
    assert_eq!(address.city.as_deref(), Some("Tokyo"));
    assert_eq!(address.street, None);
    let restored: CvDocument =
        serde_json::from_value(serde_json::to_value(&document).unwrap()).unwrap();
    assert_eq!(restored, document);
}

#[test]
fn legacy_document_shape_still_deserializes() {
    // A pre-0.4 document carries none of the new fields; it must keep parsing.
    let document: CvDocument = serde_json::from_value(json!({
        "profile": {
            "name": "Ada Lovelace",
            "email": "ada@example.com",
            "phone": "+44 20 7000 0000",
            "location": "London",
            "website": "https://ada.example.com",
            "summary": "Pioneer of computing",
            "contacts": [{ "label": "GitHub", "href": "https://github.com/ada" }]
        },
        "sections": []
    }))
    .unwrap();
    assert_eq!(document.profile.name, "Ada Lovelace");
    assert!(document.profile.photo.is_none());
    assert!(document.profile.headline.is_none());
    assert!(document.profile.address.is_none());
}

#[test]
fn education_score_round_trips() {
    let entry: EducationEntry = serde_json::from_value(json!({
        "qualification": "BSc Computer Science",
        "institution": "University of London",
        "score": "3.9 GPA"
    }))
    .unwrap();
    assert_eq!(entry.score.as_deref(), Some("3.9 GPA"));
    let without_score: EducationEntry =
        serde_json::from_value(json!({ "qualification": "BSc" })).unwrap();
    assert_eq!(without_score.score, None);
    let value = serde_json::to_value(&without_score).unwrap();
    assert!(
        value.get("score").is_none(),
        "absent score must not serialize"
    );
}

#[test]
fn entry_open_fields_round_trip_in_order() {
    // Parse from text, the way a real cv.json is read: `from_value` would
    // route through serde_json's sorted `Map` and lose document order.
    let source = r#"{
        "profile": { "name": "Ada Lovelace" },
        "sections": [{
            "id": "extra",
            "title": "Extra",
            "entries": [{
                "type": "custom",
                "heading": "Facts",
                "fields": {
                    "gpa": "3.9",
                    "furigana": "エイダ",
                    "dependents": 2,
                    "willing-to-relocate": true,
                    "birth-date": "1988-05-31"
                }
            }]
        }]
    }"#;
    let document: CvDocument = serde_json::from_str(source).unwrap();
    let entry = &document.sections[0].entries[0];
    let keys: Vec<&str> = entry
        .fields
        .iter()
        .map(|field| field.key.as_str())
        .collect();
    assert_eq!(
        keys,
        [
            "gpa",
            "furigana",
            "dependents",
            "willing-to-relocate",
            "birth-date"
        ]
    );
    assert_eq!(
        entry.fields.get("gpa"),
        Some(&FieldValue::Text("3.9".into()))
    );
    assert_eq!(
        entry.fields.get("dependents"),
        Some(&FieldValue::Integer(2))
    );
    assert_eq!(
        entry.fields.get("willing-to-relocate"),
        Some(&FieldValue::Boolean(true))
    );
    assert_eq!(
        entry.fields.get("birth-date"),
        Some(&FieldValue::Date(cac_core::DatePoint::Full(
            chrono::NaiveDate::from_ymd_opt(1988, 5, 31).unwrap()
        )))
    );

    // Reinserting an existing key replaces the value in place.
    let mut fields = FieldMap::new();
    fields.insert("gpa", FieldValue::Text("3.9".into()));
    fields.insert("note", FieldValue::Text("kept".into()));
    fields.insert("GPA", FieldValue::Text("4.0".into()));
    assert_eq!(fields.len(), 2);
    assert_eq!(fields.get("gpa"), Some(&FieldValue::Text("4.0".into())));
    let ordered: Vec<&str> = fields.iter().map(|field| field.key.as_str()).collect();
    assert_eq!(ordered, ["gpa", "note"]);
}

#[test]
fn skill_dual_shape_and_level_validation() {
    let plain: Skill = serde_json::from_str("\"Rust\"").unwrap();
    assert_eq!(plain.level, None);
    assert_eq!(serde_json::to_value(&plain).unwrap(), json!("Rust"));

    let leveled: Skill = serde_json::from_value(json!({ "name": "Rust", "level": 4 })).unwrap();
    assert_eq!(leveled.level, Some(SkillLevel::Numeric(4)));
    let named: Skill = serde_json::from_value(json!({ "name": "French", "level": "C1" })).unwrap();
    assert_eq!(named.level, Some(SkillLevel::Named("C1".into())));

    assert!(serde_json::from_value::<SkillLevel>(json!(6)).is_err());
    assert!(serde_json::from_value::<Skill>(json!({ "level": 4 })).is_err());
}

#[test]
fn language_entry_round_trips() {
    let entry: LanguageEntry = serde_json::from_value(json!({
        "name": "Français",
        "proficiency": "C1"
    }))
    .unwrap();
    assert_eq!(entry.proficiency, Some(SkillLevel::Named("C1".into())));
    let restored: LanguageEntry =
        serde_json::from_value(serde_json::to_value(&entry).unwrap()).unwrap();
    assert_eq!(restored, entry);
    assert!(
        serde_json::from_value::<LanguageEntry>(json!({ "name": "x", "grade": "C1" })).is_err()
    );
}

#[test]
fn reference_entry_round_trips() {
    let entry: ReferenceEntry = serde_json::from_value(json!({
        "name": "Christine Faulkner",
        "organization": "Nova Industries",
        "role": "CTO",
        "email": "christine@example.com",
        "phone": "+1 555 0100",
        "note": "Managed me for four years"
    }))
    .unwrap();
    let restored: ReferenceEntry =
        serde_json::from_value(serde_json::to_value(&entry).unwrap()).unwrap();
    assert_eq!(restored, entry);
    assert!(
        serde_json::from_value::<ReferenceEntry>(json!({ "name": "x", "alias": "y" })).is_err()
    );
}

#[test]
fn field_table_covers_the_extension_fields() {
    let photo = core_field("photo").expect("photo is a core field");
    assert!(photo.applies_to(FieldScope::Profile));
    assert_eq!(photo.json_resume, Some("basics.image"));

    let level = core_field("level").expect("level is a core field");
    assert!(level.applies_to(FieldScope::SkillItem));
    assert_eq!(level.parser, FieldParser::Level);

    let score = core_field("score").expect("score is a core field");
    assert!(score.applies_to(FieldScope::Entries(&[cac_core::EntryKindName::Education])));
    assert!(!score.applies_to(FieldScope::Entries(&[cac_core::EntryKindName::Experience])));
    assert_eq!(score.json_resume, Some("education[].score"));

    let proficiency = core_field("proficiency").expect("proficiency is a core field");
    assert!(proficiency.applies_to(FieldScope::Entries(&[cac_core::EntryKindName::Language])));

    // `kind` and `tags` may appear under sections and entries alike.
    let kind = core_field("kind").expect("kind is a core field");
    assert!(kind.applies_to(FieldScope::Section));
    assert!(kind.applies_to(FieldScope::Entries(&[cac_core::EntryKindName::Experience])));

    // The table covers every field the legacy parser accepted, so the
    // parser's allow-list can later be generated from it.
    for key in [
        "email",
        "phone",
        "location",
        "website",
        "summary",
        "period",
        "date",
        "url",
        "organization",
        "qualification",
        "institution",
        "publisher",
        "highlights",
        "kind",
        "id",
        "tags",
        "content",
        "name",
        "role",
        "title",
        "skills",
    ] {
        assert!(
            core_field(key).is_some(),
            "`{key}` must be in the field table"
        );
    }
    assert!(
        CORE_FIELDS.len() >= 29,
        "the table covers the legacy and extension fields"
    );
}

#[test]
fn entry_kind_names_carry_kebab_spellings() {
    use cac_core::EntryKindName;
    assert_eq!(EntryKindName::SkillGroup.as_str(), "skill-group");
    assert_eq!(EntryKindName::Language.as_str(), "language");
    assert_eq!(EntryKindName::Reference.as_str(), "reference");
    assert_eq!(EntryKindName::Text.as_str(), "text");
}

#[test]
fn entry_kind_enum_still_accepts_legacy_kinds() {
    let document: CvDocument = serde_json::from_value(json!({
        "profile": { "name": "Ada Lovelace" },
        "sections": [{
            "id": "work",
            "title": "Work",
            "kind": "experience",
            "entries": [{
                "type": "experience",
                "role": "Director",
                "organization": "Analytical Engines",
                "period": { "start": "2023-01", "end": "present" }
            }]
        }]
    }))
    .unwrap();
    let entry = &document.sections[0].entries[0];
    assert!(matches!(entry.kind, EntryKind::Experience(_)));
}
