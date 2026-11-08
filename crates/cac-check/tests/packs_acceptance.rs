//! M2 acceptance: a jp-rirekisho-style pack types a fixture's open fields;
//! undeclared keys are advisories, mistyped declared values are errors.

use cac_check::Severity;
use cac_check::packs::check_packs;
use cac_core::FieldPack;
use cac_io::packs::load_pack_from_str;

const PACK_TOML: &str = r#"
name = "jp-rirekisho"

[fields.profile.image]
label = "Photo"
type = "text"
required = true

[fields.entry.commute]
label = "通勤時間"
type = "text"

[fields.entry.dependents]
label = "扶養家族（配偶者を除く）"
type = "integer"

[fields.entry.commute_time]
label = "通勤時間"
type = "length"
"#;

fn cv_with(fields: &str) -> cac_core::CvDocument {
    cv_with_profile("Ada", fields)
}

fn cv_with_profile(name: &str, fields: &str) -> cac_core::CvDocument {
    let source = format!(
        r#"{{"profile":{{"name":"{name}","photo":"photo.jpg"}},"sections":[{{"id":"s","title":"S","entries":[{{"type":"custom","heading":"Facts","fields":{{ {fields} }}}}]}}]}}"#
    );
    cac_io::parse(&source, cac_io::InputFormat::Json).unwrap()
}

#[test]
fn pack_toml_loads_typed_fields() {
    let pack = load_pack_from_str("jp-rirekisho", PACK_TOML).unwrap();
    assert_eq!(pack.name, "jp-rirekisho");
    let dependents = pack
        .field(cac_core::PackScope::Entry, "Dependents")
        .expect("declared");
    assert_eq!(dependents.kind, cac_core::PackType::Integer);
    assert!(dependents.label.contains("扶養"));
}

#[test]
fn mistyped_declared_value_is_an_error_and_undeclared_is_advisory() {
    let pack: FieldPack = load_pack_from_str("jp", PACK_TOML).unwrap();
    let cv = cv_with(r#""dependents": "two", "favorite-color": "blue""#);
    let diagnostics = check_packs(&cv, &[pack]);
    let has_error = diagnostics
        .iter()
        .any(|d| d.severity == Severity::Error && d.message.contains("dependents"));
    let has_advisory = diagnostics
        .iter()
        .any(|d| d.message.contains("favorite-color"));
    assert!(has_error, "mistyped dependents errors: {diagnostics:?}");
    assert!(
        has_advisory,
        "undeclared favorite-color advises: {diagnostics:?}"
    );
}

#[test]
fn correct_types_and_declared_commute_pass_clean() {
    let pack = load_pack_from_str("jp", PACK_TOML).unwrap();
    let cv = cv_with(r#""dependents": 2, "commute": "40 minutes by train""#);
    let diagnostics = check_packs(&cv, &[pack]);
    assert!(
        !diagnostics.iter().any(|d| d.severity == Severity::Error),
        "correct values pass: {diagnostics:?}"
    );
}

#[test]
fn required_profile_photo_is_reported_when_missing() {
    let pack = load_pack_from_str("jp", PACK_TOML).unwrap();
    let cv = cv_with_profile("Ada", "");
    // cv_with_profile carries a photo; strip it for the missing case.
    let source = r#"{"profile":{"name":"Ada"},"sections":[]}"#;
    let missing = cac_io::parse(source, cac_io::InputFormat::Json).unwrap();
    let _ = cv;
    let diagnostics = check_packs(&missing, &[pack]);
    assert!(
        diagnostics.iter().any(
            |d| d.severity == Severity::Error && d.message.contains("requires profile `image`")
        )
    );
}

#[test]
fn no_packs_means_no_diagnostics() {
    let cv = cv_with(r#""anything": "goes""#);
    assert!(check_packs(&cv, &[]).is_empty());
}
