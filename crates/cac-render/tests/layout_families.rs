//! L1 slice 2 acceptance: the built-in table and compact families render
//! through the hooks path, and a sparse settings layer still overlays them.

use std::fs;

use cac_core::CvDocument;
use cac_render::{RenderOptions, Settings, render_pdf_with_options};
use tempfile::tempdir;

const MINIMAL_PNG: [u8; 69] = [
    0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44, 0x52,
    0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x02, 0x00, 0x00, 0x00, 0x90, 0x77, 0x53,
    0xDE, 0x00, 0x00, 0x00, 0x0C, 0x49, 0x44, 0x41, 0x54, 0x78, 0x9C, 0x63, 0xF8, 0xFF, 0xFF, 0x3F,
    0x00, 0x05, 0xFE, 0x02, 0xFE, 0x0D, 0xEF, 0x46, 0xB8, 0x00, 0x00, 0x00, 0x00, 0x49, 0x45, 0x4E,
    0x44, 0xAE, 0x42, 0x60, 0x82,
];

const MINIMAL: &str = r#"{"profile":{"name":"Ada Lovelace"},"sections":[{"id":"work","title":"Experience","kind":"experience","entries":[{"type":"experience","role":"Engineer","organization":"Acme","period":{"start":"2020","end":"2021"},"highlights":["Built a thing"]}]},{"id":"skills","title":"Skills","kind":"skills","entries":[{"type":"skill-group","name":"Languages","skills":["Rust","SQL"]}]}]}"#;

fn render_with_family(family: &str) -> (usize, Vec<u8>) {
    render_with_family_cv(family, MINIMAL)
}

fn render_with_family_cv(family: &str, cv_json: &str) -> (usize, Vec<u8>) {
    let directory = tempdir().unwrap();
    let theme = directory.path().join(".cac/themes").join(family);
    fs::create_dir_all(&theme).unwrap();
    fs::write(
        theme.join("theme.typ"),
        format!("#let theme = (layout: (family: \"{family}\"))\n"),
    )
    .unwrap();
    let cv: CvDocument = serde_json::from_str(cv_json).unwrap();
    let settings: Settings = serde_json::from_str(&format!(r#"{{"theme":"{family}"}}"#)).unwrap();
    let rendered = render_pdf_with_options(
        &cv,
        &RenderOptions {
            project_dir: Some(directory.path().join(".cac")),
            user_dir: None,
            settings,
        },
    )
    .unwrap();
    let pages = rendered.pages;
    (pages, rendered.bytes)
}

#[test]
fn table_and_compact_families_render_the_corpus() {
    for family in ["table", "compact", "classic"] {
        let (pages, bytes) = render_with_family(family);
        assert!(bytes.starts_with(b"%PDF-"), "{family} renders a PDF");
        assert!(pages >= 1, "{family} renders at least one page");
    }
}

#[test]
fn sidebar_family_renders_two_columns_with_languages() {
    let sidebar_cv = r#"{"profile":{"name":"Ada Lovelace"},"sections":[
        {"id":"experience","title":"Experience","kind":"experience","entries":[
            {"type":"experience","role":"Engineer","organization":"Acme","period":{"start":"2020","end":"2021"},"highlights":["Built a thing"]}]},
        {"id":"languages","title":"Languages","kind":"custom","entries":[
            {"type":"language","name":"Japanese","proficiency":"Native"},
            {"type":"language","name":"English","proficiency":"C1"}]}]}"#;
    let (pages, bytes) = render_with_family_cv("sidebar", sidebar_cv);
    assert!(bytes.starts_with(b"%PDF-"), "sidebar renders a PDF");
    assert!(pages >= 1, "sidebar renders at least one page");
    let (_, classic) = render_with_family_cv("classic", sidebar_cv);
    assert_ne!(bytes, classic, "sidebar differs from classic");
}

#[test]
fn sidebar_photo_slot_reads_the_project_jail() {
    // A 1x1 PNG next to the CV; the photo slot resolves it inside the
    // project root and renders it.
    let directory = tempdir().unwrap();
    let theme = directory.path().join(".cac/themes").join("sidebar");
    fs::create_dir_all(&theme).unwrap();
    fs::write(
        theme.join("theme.typ"),
        "#let theme = (layout: (family: \"sidebar\"))\n",
    )
    .unwrap();
    fs::write(directory.path().join("cv.json"), MINIMAL).unwrap();
    let png: [u8; 69] = [
        0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44,
        0x52, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x02, 0x00, 0x00, 0x00, 0x90,
        0x77, 0x53, 0xDE, 0x00, 0x00, 0x00, 0x0C, 0x49, 0x44, 0x41, 0x54, 0x78, 0x9C, 0x63, 0xF8,
        0xFF, 0xFF, 0x3F, 0x00, 0x05, 0xFE, 0x02, 0xFE, 0x0D, 0xEF, 0x46, 0xB8, 0x00, 0x00, 0x00,
        0x00, 0x49, 0x45, 0x4E, 0x44, 0xAE, 0x42, 0x60, 0x82,
    ];
    fs::write(directory.path().join("photo.png"), png).unwrap();
    let cv: CvDocument =
        serde_json::from_str(r#"{"profile":{"name":"Ada","photo":"photo.png"},"sections":[]}"#)
            .unwrap();
    let settings: Settings = serde_json::from_str(r#"{"theme":"sidebar"}"#).unwrap();
    let rendered = render_pdf_with_options(
        &cv,
        &RenderOptions {
            project_dir: Some(directory.path().join(".cac")),
            user_dir: None,
            settings,
        },
    )
    .unwrap();
    assert!(rendered.bytes.starts_with(b"%PDF-"));
}

#[test]
fn families_differ_in_layout() {
    // The table family renders the dated section as a table (different
    // glyph stream than compact); assert their bytes are not identical.
    let (_, table) = render_with_family("table");
    let (_, compact) = render_with_family("compact");
    assert_ne!(table, compact, "table and compact render differently");
}

#[test]
fn the_form_family_rejects_unsupported_sections_by_name() {
    let directory = tempdir().unwrap();
    let theme = directory.path().join(".cac/themes").join("form");
    fs::create_dir_all(&theme).unwrap();
    fs::write(
        theme.join("theme.typ"),
        r#"
#import "/.cac/base.typ" as base
#let theme = (layout: (
  family: "form",
  requires: (sections: ("education",), profile: ("photo",)),
  allowed: ("education", "experience", "languages", "certifications"),
))
"#,
    )
    .unwrap();
    let cv: CvDocument = serde_json::from_str(
        r#"{"profile":{"name":"Ada","photo":"photo.png"},"sections":[
            {"id":"edu","title":"Education","kind":"education","entries":[
                {"type":"education","qualification":"MSc","institution":"TIT",
                 "period":{"start":"2018","end":"2020"},"highlights":["Thesis"]}]},
            {"id":"projects","title":"Projects","kind":"projects","entries":[
                {"type":"project","name":"Toolkit","period":{"start":"2021"},"highlights":["Built it"]}]}
        ]}"#,
    )
    .unwrap();
    fs::write(directory.path().join("photo.png"), MINIMAL_PNG).unwrap();
    let settings: Settings = serde_json::from_str(r#"{"theme":"form"}"#).unwrap();
    let error = render_pdf_with_options(
        &cv,
        &RenderOptions {
            project_dir: Some(directory.path().join(".cac")),
            user_dir: None,
            settings,
        },
    )
    .unwrap_err();
    assert!(
        error.to_string().contains("`Projects`"),
        "the rejection names the section: {error}"
    );
}

#[test]
fn a_conforming_document_renders_through_the_form_family() {
    let directory = tempdir().unwrap();
    let theme = directory.path().join(".cac/themes").join("form");
    fs::create_dir_all(&theme).unwrap();
    fs::write(
        theme.join("theme.typ"),
        r#"
#let theme = (layout: (
  family: "form",
  requires: (sections: ("education",), profile: ("photo",)),
  allowed: ("education", "experience", "languages", "certifications"),
))
"#,
    )
    .unwrap();
    let cv: CvDocument = serde_json::from_str(
        r#"{"profile":{"name":"Ada","photo":"photo.png"},"sections":[
            {"id":"edu","title":"Education","kind":"education","entries":[
                {"type":"education","qualification":"MSc","institution":"TIT",
                 "period":{"start":"2018","end":"2020"},"highlights":["Thesis done"]}]}
        ]}"#,
    )
    .unwrap();
    fs::write(directory.path().join("photo.png"), MINIMAL_PNG).unwrap();
    let settings: Settings = serde_json::from_str(r#"{"theme":"form"}"#).unwrap();
    let rendered = render_pdf_with_options(
        &cv,
        &RenderOptions {
            project_dir: Some(directory.path().join(".cac")),
            user_dir: None,
            settings,
        },
    )
    .unwrap();
    assert!(rendered.bytes.starts_with(b"%PDF-"));
    assert!(rendered.pages >= 1);
}

#[test]
fn sparse_settings_still_override_a_family() {
    let directory = tempdir().unwrap();
    let theme = directory.path().join(".cac/themes").join("compact");
    fs::create_dir_all(&theme).unwrap();
    fs::write(
        theme.join("theme.typ"),
        "#let theme = (layout: (family: \"compact\"))\n",
    )
    .unwrap();
    let cv: CvDocument = serde_json::from_str(MINIMAL).unwrap();
    let settings: Settings =
        serde_json::from_str(r#"{"theme":"compact","typography":{"font_size":"11pt"}}"#).unwrap();
    let rendered = render_pdf_with_options(
        &cv,
        &RenderOptions {
            project_dir: Some(directory.path().join(".cac")),
            user_dir: None,
            settings,
        },
    )
    .unwrap();
    assert!(rendered.bytes.starts_with(b"%PDF-"));
}
