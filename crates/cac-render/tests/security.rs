//! Rendering-safety regressions: markup injection through open-field keys,
//! CSS/HTML injection through style strings, and the theme file jail.

use std::fs;

use cac_core::CvDocument;
use cac_render::{
    RenderOptions, Settings, defaults, render_html, render_html_for_sheet, render_pdf_with_options,
};
use tempfile::tempdir;

#[test]
fn entry_field_keys_are_html_escaped() {
    // A field key is author data; its label renders as text even though it
    // is capitalized, never as a live element. (Deserialization rejects
    // such keys outright — see cac-core — this is the renderer's own
    // defense for values built in memory.)
    let mut fields = cac_core::FieldMap::new();
    fields.insert(
        "<img src=x onerror=alert(1)>",
        cac_core::FieldValue::Text("v".into()),
    );
    let profile = cac_core::Profile {
        name: "Ada".into(),
        ..Default::default()
    };
    let entry = cac_core::Entry {
        kind: cac_core::EntryKind::Custom(cac_core::CustomEntry {
            heading: cac_core::RichText::literal("N"),
            period: None,
            date: None,
            highlights: Vec::new(),
        }),
        content: None,
        fields,
        tags: Default::default(),
        origin: cac_core::Origin::default(),
    };
    let section = cac_core::Section {
        id: "s".into(),
        title: "S".into(),
        kind: cac_core::SectionKind::Custom,
        tags: Default::default(),
        entries: vec![entry],
    };
    let cv = CvDocument {
        profile,
        sections: vec![section],
    };
    let html = render_html(&cv);
    assert!(!html.contains("<img"), "live element in output: {html}");
    assert!(html.contains("&lt;img"), "label not escaped: {html}");
}

#[test]
fn font_names_cannot_break_out_of_the_style_sheet() {
    // A hostile font name from settings (or a theme's `[style]`) must not
    // close the style element and inject script.
    let cv: CvDocument =
        serde_json::from_str(r#"{"profile":{"name":"Ada"},"sections":[]}"#).unwrap();
    let mut sheet = defaults();
    sheet.fonts.body = "x\";}</style><script>alert(2)</script><style>".into();
    let html = render_html_for_sheet(&cv, &sheet);
    assert!(
        !html.contains("<script>alert(2)</script>"),
        "script survived in output: {html}"
    );
    assert!(!html.contains("</style><script>"), "{html}");
}

#[test]
fn theme_code_cannot_read_files_outside_the_asset_allowlist() {
    // The only project files a theme may read are the renderer-collected
    // assets (the document's photo). Everything else is refused before the
    // filesystem is touched, so a downloaded theme cannot embed `.env` or
    // `.git/config` contents in its output.
    let directory = tempdir().unwrap();
    fs::write(directory.path().join("secret.txt"), "TOPSECRET").unwrap();
    fs::write(directory.path().join(".env"), "TOKEN=TOPSECRET").unwrap();
    let theme = directory.path().join(".cac/themes/evil");
    fs::create_dir_all(&theme).unwrap();
    fs::write(
        theme.join("theme.typ"),
        "#(read(\"/secret.txt\"))#(read(\"/.env\"))",
    )
    .unwrap();
    let settings: Settings = serde_json::from_str(r#"{"theme":"evil"}"#).unwrap();
    let cv: CvDocument =
        serde_json::from_str(r#"{"profile":{"name":"Ada"},"sections":[]}"#).unwrap();
    let result = render_pdf_with_options(
        &cv,
        &RenderOptions {
            project_dir: Some(directory.path().join(".cac")),
            user_dir: None,
            settings,
        },
    );
    let error = result.expect_err("reading outside the jail must fail");
    let message = error.to_string();
    assert!(!message.contains("TOPSECRET"), "secret leaked: {message}");
}
