//! The S3 acceptance: HTML reads the resolved style sheet (CSS custom
//! properties, a theme accent visible) and prints dates through the shared
//! locale-aware formatter, so HTML and PDF spell dates identically.

use cac_io::{InputFormat, parse};
use cac_render::{Settings, render_html_for_sheet, resolved_style_for};

#[test]
fn html_and_view_spell_dates_identically() {
    let cv = parse(
        include_str!("../../cac-io/tests/fixtures/flexible.md"),
        InputFormat::Markdown,
    )
    .unwrap();
    let html = render_html_for_sheet(&cv, &cac_render::defaults());
    // The view's en spellings are the PDF's: both come from the shared
    // formatter (view/dates.rs), so the same strings appear in both.
    assert!(
        html.contains("May 2023"),
        "publication date spelled May 2023"
    );
    assert!(html.contains("2022"), "open period start spelled 2022");
    assert!(
        !html.contains(">2023-05<"),
        "raw ISO dates no longer leak into the HTML"
    );
}

#[test]
fn a_theme_accent_shows_up_in_the_html() {
    let cv: cac_core::CvDocument =
        serde_json::from_str(r#"{"profile":{"name":"Ada Lovelace"},"sections":[]}"#).unwrap();
    let settings: Settings =
        serde_json::from_str(r##"{"style":{"accent_color":"#1f4e79"}}"##).unwrap();
    let resolved = resolved_style_for(&settings, "classic").unwrap();
    let html = render_html_for_sheet(&cv, &resolved);
    assert!(
        html.contains("--color-accent:#1f4e79"),
        "the accent color lands in the CSS variables"
    );
    assert!(
        html.contains("var(--color-accent)"),
        "the accent is referenced by the stylesheet"
    );
}

#[test]
fn sheet_locale_drives_the_html_dates() {
    let cv: cac_core::CvDocument = serde_json::from_str(
        r#"{"profile":{"name":"Ada"},"sections":[{"id":"e","title":"Experience","kind":"experience","entries":[{"type":"experience","role":"Role","period":{"start":"2022-10","end":"present"}}]}]}"#,
    )
    .unwrap();
    let mut sheet = cac_render::defaults();
    sheet.locale = "vi".into();
    let html = render_html_for_sheet(&cv, &sheet);
    assert!(html.contains("Tháng Mười 2022 – Nay"), "vi dates: {html}");
}

#[test]
fn resolve_matches_the_layering_the_pdf_uses() {
    // The same layering as the PDF path: family under settings.
    let settings: Settings =
        serde_json::from_str(r#"{"typography":{"font_size":"11pt"}}"#).unwrap();
    let resolved = resolved_style_for(&settings, "classic").unwrap();
    assert_eq!(
        resolved.text.size.to_string(),
        "11pt",
        "settings beat the family"
    );
    assert_eq!(
        resolved.fonts.body, "New Computer Modern",
        "the family fills fonts"
    );
    let resolved_plain = resolved_style_for(&Settings::default(), "other").unwrap();
    assert_eq!(resolved_plain.fonts.body, "Libertinus Serif");
}
