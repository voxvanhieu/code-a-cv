//! D1 acceptance: two builds with a fixed document_date are byte-identical,
//! and the world's clock returns the setting.

use cac_core::CvDocument;
use cac_render::{RenderOptions, Settings, render_pdf_with_options};

const CV: &str = r#"{"profile":{"name":"Ada Lovelace"},"sections":[]}"#;

fn build(settings: &str) -> Vec<u8> {
    let cv: CvDocument = serde_json::from_str(CV).unwrap();
    let settings: Settings = serde_json::from_str(settings).unwrap();
    render_pdf_with_options(
        &cv,
        &RenderOptions {
            settings,
            ..RenderOptions::default()
        },
    )
    .unwrap()
    .bytes
}

#[test]
fn two_builds_with_a_fixed_date_are_byte_identical() {
    let first = build(r#"{"document_date":"2026-01-15"}"#);
    let second = build(r#"{"document_date":"2026-01-15"}"#);
    assert_eq!(first, second, "fixed-date builds are byte-identical");
    assert!(first.starts_with(b"%PDF-"));
}

#[test]
fn an_invalid_document_date_is_rejected() {
    let settings: Settings = serde_json::from_str(r#"{"document_date":"January 15"}"#).unwrap();
    assert!(
        Settings::validate(settings).is_err(),
        "non-ISO dates are rejected at validation"
    );
}

#[test]
fn the_classic_layout_does_not_print_dates() {
    // The classic header carries no date, so the bytes are equal; the
    // assertion pins that both builds succeed. The printer-path coverage
    // lives in `a_theme_printing_the_clock_sees_the_document_date`.
    let a = build(r#"{"document_date":"2026-01-15"}"#);
    let b = build(r#"{"document_date":"2027-01-15"}"#);
    assert_eq!(a, b);
    assert!(!a.is_empty());
}

#[test]
fn a_theme_printing_the_clock_sees_the_document_date() {
    // The date genuinely reaches the world: a theme that prints
    // today() produces different bytes for different dates.
    use std::fs;
    use tempfile::tempdir;
    let cv: CvDocument = serde_json::from_str(CV).unwrap();
    let build_with = |date: &str| {
        let directory = tempdir().unwrap();
        let theme = directory.path().join(".cac/themes/dated");
        fs::create_dir_all(&theme).unwrap();
        fs::write(
            theme.join("theme.typ"),
            "#let theme = (layout: (family: \"classic\", header: (ctx) => align(right)[Printed #datetime.today().display()]))\n",
        )
        .unwrap();
        let settings: Settings =
            serde_json::from_str(&format!(r#"{{"theme":"dated","document_date":"{date}"}}"#))
                .unwrap();
        render_pdf_with_options(
            &cv,
            &RenderOptions {
                settings,
                project_dir: Some(directory.path().join(".cac")),
                ..RenderOptions::default()
            },
        )
        .unwrap()
        .bytes
    };
    let a = build_with("2026-01-15");
    let b = build_with("2027-01-15");
    assert_ne!(a, b, "the printed date differs");
    assert!(a.starts_with(b"%PDF-") && b.starts_with(b"%PDF-"));
}
