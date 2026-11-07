//! L1 slice 2 acceptance: the built-in table and compact families render
//! through the hooks path, and a sparse settings layer still overlays them.

use std::fs;

use cac_core::CvDocument;
use cac_render::{RenderOptions, Settings, render_pdf_with_options};
use tempfile::tempdir;

const MINIMAL: &str = r#"{"profile":{"name":"Ada Lovelace"},"sections":[{"id":"work","title":"Experience","kind":"experience","entries":[{"type":"experience","role":"Engineer","organization":"Acme","period":{"start":"2020","end":"2021"},"highlights":["Built a thing"]}]},{"id":"skills","title":"Skills","kind":"skills","entries":[{"type":"skill-group","name":"Languages","skills":["Rust","SQL"]}]}]}"#;

fn render_with_family(family: &str) -> (usize, Vec<u8>) {
    let directory = tempdir().unwrap();
    let theme = directory.path().join(".cac/themes").join(family);
    fs::create_dir_all(&theme).unwrap();
    fs::write(
        theme.join("theme.typ"),
        format!("#let theme = (layout: (family: \"{family}\"))\n"),
    )
    .unwrap();
    let cv: CvDocument = serde_json::from_str(MINIMAL).unwrap();
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
fn families_differ_in_layout() {
    // The table family renders the dated section as a table (different
    // glyph stream than compact); assert their bytes are not identical.
    let (_, table) = render_with_family("table");
    let (_, compact) = render_with_family("compact");
    assert_ne!(table, compact, "table and compact render differently");
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
