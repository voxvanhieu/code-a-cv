//! M2 acceptance: a jp-rirekisho-style pack types a fixture's open fields;
//! undeclared keys are advisories, mistyped declared values are errors.
//!
//! Markdown authors write every open field as text, so the validator
//! coerces text through the declared kind: `Dependents: 2` passes an
//! `integer` field, `Remote: yes` a `boolean` field.
//!
//! Requiredness resolves against the model. Required **profile** keys check
//! the typed profile slots (`photo`, `email`, …). A required **entry** key
//! is checked only on entries that can carry it: core keys use the core
//! field table's entry scopes; pack-only keys are required on an entry only
//! when the entry's section already uses the key (a sibling declares it) —
//! so a required `commute` stays quiet on a Skills section but flags an
//! experience entry that skips the field its siblings carry.

use cac_check::packs::check_packs;
use cac_check::{Diagnostic, Severity};
use cac_core::{FieldPack, PackScope, PackType};
use cac_io::packs::parse_pack;
use cac_io::{InputFormat, parse};

const PACK_TOML: &str = r#"
name = "jp-rirekisho"

[fields.profile.photo]
label = "Photo"
type = "text"
required = true

[fields.profile.email]
label = "Email"
type = "text"
required = true

[fields.entry.commute]
label = "通勤時間"
type = "text"
required = true

[fields.entry.organization]
label = "勤務先"
type = "text"
required = true

[fields.entry.dependents]
label = "扶養家族（配偶者を除く）"
type = "integer"

[fields.entry.commute_time]
label = "通勤時間"
type = "length"

[fields.entry.remote]
label = "Remote work"
type = "boolean"

[fields.entry.available_from]
label = "Available from"
type = "date"
"#;

fn jp_pack() -> FieldPack {
    parse_pack(PACK_TOML, "fields.toml").unwrap()
}

fn cv_from_json(source: &str) -> cac_core::CvDocument {
    parse(source, InputFormat::Json).unwrap()
}

/// A profile with the required slots filled, plus one custom entry carrying
/// the given open fields.
fn cv_with(fields: &str) -> cac_core::CvDocument {
    let source = format!(
        r#"{{"profile":{{"name":"Ada","email":"ada@x.io","photo":"photo.jpg"}},"sections":[{{"id":"s","title":"S","entries":[{{"type":"custom","heading":"Facts","fields":{{ {fields} }}}}]}}]}}"#
    );
    cv_from_json(&source)
}

/// A markdown CV: the name block, then the given sections.
fn markdown_cv(sections: &str) -> cac_core::CvDocument {
    let source = format!("# Ada\nEmail: ada@x.io\nPhoto: photo.jpg\n\n{sections}");
    parse(&source, InputFormat::Markdown).unwrap()
}

fn errors(diagnostics: &[Diagnostic]) -> Vec<&Diagnostic> {
    diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.severity == Severity::Error)
        .collect()
}

fn count_code(diagnostics: &[Diagnostic], code: &str) -> usize {
    diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.code == code)
        .count()
}

#[test]
fn pack_toml_loads_the_declared_name_and_typed_fields() {
    let pack = parse_pack(PACK_TOML, "fields.toml").unwrap();
    assert_eq!(pack.name, "jp-rirekisho", "the declared name, not the path");
    let dependents = pack
        .field(PackScope::Entry, "Dependents")
        .expect("declared");
    assert_eq!(dependents.kind, PackType::Integer);
    assert!(dependents.label.contains("扶養"));
    assert!(
        pack.field(PackScope::Entry, "commute-time").is_some(),
        "`commute_time` and `commute-time` are one field"
    );
}

#[test]
fn mistyped_declared_value_is_an_error_and_undeclared_is_advisory() {
    let pack = jp_pack();
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
    assert!(
        !errors(&diagnostics)
            .iter()
            .any(|d| d.message.contains("commute")),
        "a required pack-only field stays quiet while no sibling uses it: {diagnostics:?}"
    );
}

#[test]
fn typed_values_and_coerced_markdown_text_pass_clean() {
    let pack = jp_pack();
    // JSON carries typed values directly.
    let cv = cv_with(
        r#""dependents": 2, "remote": true, "available-from": "2026-04", "commute": "40 minutes by train""#,
    );
    let diagnostics = check_packs(&cv, &[pack]);
    assert!(
        errors(&diagnostics).is_empty(),
        "correct typed values pass: {diagnostics:?}"
    );
}

#[test]
fn markdown_text_is_coerced_to_the_declared_kind() {
    // `Dependents: 2` is stored as text by the Markdown parser; the pack
    // says integer, so the validator coerces before it judges.
    let pack = jp_pack();
    let cv = markdown_cv(
        "## Facts\nKind: custom\n\n### Personal\nKind: custom\nDependents: 2\nRemote: yes\nAvailable From: 2026-04\nCommute Time: 32mm\nCommute: 40 minutes\n",
    );
    let diagnostics = check_packs(&cv, &[pack]);
    assert!(
        errors(&diagnostics).is_empty(),
        "markdown text coerces to the declared kinds: {diagnostics:?}"
    );
    assert!(
        diagnostics.is_empty(),
        "every declared key is covered, no advisories: {diagnostics:?}"
    );
}

#[test]
fn markdown_text_that_fails_the_coercion_is_an_error() {
    let pack = jp_pack();
    let cv = markdown_cv(
        "## Facts\nKind: custom\n\n### Personal\nKind: custom\nDependents: two\nRemote: maybe\nCommute: 40 minutes\n",
    );
    let diagnostics = check_packs(&cv, &[pack]);
    let integer_errors = diagnostics
        .iter()
        .any(|d| d.severity == Severity::Error && d.message.contains("not an integer"));
    let boolean_errors = diagnostics
        .iter()
        .any(|d| d.severity == Severity::Error && d.message.contains("not a boolean"));
    assert!(
        integer_errors,
        "text `two` fails the integer pack: {diagnostics:?}"
    );
    assert!(
        boolean_errors,
        "text `maybe` fails the boolean pack: {diagnostics:?}"
    );
}

#[test]
fn required_profile_keys_check_the_model_slots() {
    let pack = jp_pack();
    let complete = cv_from_json(
        r#"{"profile":{"name":"Ada","email":"ada@x.io","photo":"photo.jpg"},"sections":[]}"#,
    );
    assert!(
        check_packs(&complete, &[jp_pack()]).is_empty(),
        "photo and email slots satisfy the pack"
    );
    let missing = cv_from_json(r#"{"profile":{"name":"Ada","photo":"photo.jpg"},"sections":[]}"#);
    let diagnostics = check_packs(&missing, &[pack]);
    assert!(
        diagnostics.iter().any(
            |d| d.severity == Severity::Error && d.message.contains("requires profile `email`")
        ),
        "the email slot is empty: {diagnostics:?}"
    );
    assert!(
        !diagnostics
            .iter()
            .any(|d| d.message.contains("requires profile `photo`")),
        "the photo slot is filled even though no open field says `photo`: {diagnostics:?}"
    );
}

#[test]
fn required_pack_entry_field_flags_only_the_sibling_that_skips_it() {
    // The section-scope rule: a required pack-only entry field is checked
    // only in sections where a sibling entry declares it.
    let pack = jp_pack();
    let cv = markdown_cv(
        "## Experience\nKind: experience\n\n### Engineer\nOrganization: Acme\nPeriod: 2020–2021\nCommute: 40 minutes\n\n### Engineer\nOrganization: Beta\nPeriod: 2021–2022\n\n## Skills\nKind: skill-group\n\n### Spoken\nKind: skill-group\nName: Spoken\n\n- Japanese\n",
    );
    let diagnostics = check_packs(&cv, &[pack]);
    let required = diagnostics
        .iter()
        .filter(|d| d.message.contains("requires entry field `commute`"))
        .collect::<Vec<_>>();
    assert_eq!(
        required.len(),
        1,
        "exactly the Beta entry skips a declared field: {diagnostics:?}"
    );
    assert_eq!(required[0].path, "sections[0].entries[1]");
    assert!(
        !diagnostics.iter().any(
            |d| d.message.contains("requires entry field") && d.path.starts_with("sections[1]")
        ),
        "the Skills section never declares `commute`, so it is not required there: {diagnostics:?}"
    );
}

#[test]
fn required_core_entry_field_is_scoped_to_the_kinds_that_carry_it() {
    // `organization` is a core field scoped to experience entries: the
    // skill group cannot carry it, so requiring it must not fire there.
    let pack = jp_pack();
    let cv = cv_from_json(
        r#"{"profile":{"name":"Ada","email":"ada@x.io","photo":"photo.jpg"},"sections":[
            {"id":"work","title":"Work","entries":[{"type":"experience","role":"Engineer"}]},
            {"id":"skills","title":"Skills","entries":[{"type":"skill-group","name":"Spoken","skills":["Japanese"]}]}
        ]}"#,
    );
    let diagnostics = check_packs(&cv, &[pack]);
    let required = diagnostics
        .iter()
        .filter(|d| d.message.contains("requires entry field `organization`"))
        .collect::<Vec<_>>();
    assert_eq!(
        required.len(),
        1,
        "only the experience entry: {diagnostics:?}"
    );
    assert_eq!(required[0].path, "sections[0].entries[0]");
}

#[test]
fn identical_diagnostics_are_reported_once() {
    let cv = cv_with(r#""dependents": "two""#);
    let twice = check_packs(&cv, &[jp_pack(), jp_pack()]);
    assert_eq!(
        count_code(&twice, "CACPACK2"),
        1,
        "two agreeing packs report one diagnostic: {twice:?}"
    );
    let other = parse_pack(
        &PACK_TOML.replace("jp-rirekisho", "jp-nagoya"),
        "fields.toml",
    )
    .unwrap();
    let distinct = check_packs(&cv, &[jp_pack(), other]);
    assert_eq!(
        count_code(&distinct, "CACPACK2"),
        2,
        "packs with different names report distinct diagnostics: {distinct:?}"
    );
}

#[test]
fn no_packs_means_no_diagnostics() {
    let cv = cv_with(r#""anything": "goes""#);
    assert!(check_packs(&cv, &[]).is_empty());
}
