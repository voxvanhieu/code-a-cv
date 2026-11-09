//! V1 slice 2: open fields render through the default family in both
//! output formats; documents without fields are unchanged (parity holds
//! via the determinism suites).

use cac_render::{RenderOptions, Settings, render_html_for_sheet, render_pdf_with_options};

const FIELDS_CV: &str = r#"{
    "profile": {
        "name": "Ada Lovelace",
        "fields": { "birth-date": "1988-05-31" }
    },
    "sections": [{
        "id": "extra",
        "title": "Extra",
        "entries": [{
            "type": "custom",
            "heading": "Facts",
            "fields": { "furigana": "Miyoji Name", "dependents": 2 }
        }]
    }]
}"#;

#[test]
fn open_fields_render_in_pdf_and_html() {
    let cv: cac_core::CvDocument = serde_json::from_str(FIELDS_CV).unwrap();
    let rendered = render_pdf_with_options(
        &cv,
        &RenderOptions {
            settings: Settings::default(),
            ..RenderOptions::default()
        },
    )
    .unwrap();
    assert!(rendered.bytes.starts_with(b"%PDF-"));

    let html = render_html_for_sheet(&cv, &cac_render::defaults());
    assert!(html.contains("Furigana"), "entry field keys become labels");
    assert!(html.contains("Miyoji Name"), "the value renders");
    assert!(html.contains("Dependents"), "integer values render");
    assert!(
        html.contains("Birth Date"),
        "profile fields render in the header"
    );
}

#[test]
fn documents_without_fields_are_unchanged() {
    let cv: cac_core::CvDocument =
        serde_json::from_str(r#"{"profile":{"name":"Ada"},"sections":[]}"#).unwrap();
    let html = render_html_for_sheet(&cv, &cac_render::defaults());
    // The stylesheet always carries the rule; no field ROWS without fields.
    assert!(
        !html.contains("class=\"field\"><span"),
        "no field rows without fields"
    );
}
