//! The S1 acceptance: every settings.json in docs/examples resolves to the
//! same style sheet through the legacy shim as its modern spelling would,
//! deprecated keys carry advisories naming their replacements, and unknown
//! keys report their path.

use cac_render::{Settings, StyleOverrides, resolve_layered, shim};

fn resolve_settings(source: &str) -> cac_render::ResolvedStyle {
    let settings = serde_json::from_str::<Settings>(source).expect("settings parse");
    let shimmed = shim(&settings);
    resolve_layered(None, &shimmed).expect("resolve").0
}

#[test]
fn complete_cv_settings_shim_to_the_same_sheet_as_the_new_spelling() {
    let legacy = std::fs::read_to_string("../../docs/examples/complete-cv/settings.json")
        .expect("example settings");
    let shimmed = resolve_settings(&legacy);
    let modern = resolve_settings(
        r##"{
            "$schema": ".cac/settings.schema.json",
            "root": "Ada_Lovelace.md",
            "naming": "Ada_Lovelace_CompleteCV",
            "theme": "classic",
            "style": {
                "page": { "paper": "a4", "margin": { "top": "16mm", "right": "16mm", "bottom": "16mm", "left": "16mm" } },
                "fonts": { "body": "Libertinus Serif", "heading": "Libertinus Serif" },
                "colors": { "text": "#000000", "muted": "#aaaaaa", "accent": "#000000" },
                "text": { "size": "9.5pt", "leading": "0.6em", "paragraph_gap": "1.2em", "align": "justified" },
                "header": { "align": "left", "rule": "none" },
                "headings": { "name": {}, "section": { "size": null, "weight": "bold", "color": "text", "transform": "none", "rule": "none", "space_before": "0pt", "space_after": "0.65em" }, "entry": {} },
                "spacing": { "section": "1.1em", "entry": "0.9em", "item": null },
                "lists": { "bullet": "•" },
                "links": { "underline": true },
                "entry": { "date_position": "right", "subtitle_style": "italic", "keep_together": true },
                "locale": "en"
            }
        }"##,
    );
    assert_eq!(
        shimmed, modern,
        "the shim must land on the modern spelling's sheet"
    );
}

#[test]
fn customized_layout_settings_shim_to_the_same_sheet_as_the_new_spelling() {
    let legacy = std::fs::read_to_string("../../docs/examples/customized-layout/settings.json")
        .expect("example settings");
    let shimmed = resolve_settings(&legacy);
    let modern = resolve_settings(
        r##"{
            "$schema": ".cac/settings.schema.json",
            "root": "cv.yaml",
            "naming": "Ada_Lovelace_CustomizedLayout",
            "theme": "table-layout",
            "style": {
                "page": { "paper": "a4", "margin": { "top": "16mm", "right": "16mm", "bottom": "16mm", "left": "16mm" } },
                "colors": { "accent": "#1f4e79" },
                "text": { "size": "9.5pt", "align": "left" },
                "header": { "align": "left" },
                "headings": { "section": { "space_after": "0.5em" } }
            }
        }"##,
    );
    assert_eq!(
        shimmed, modern,
        "the shim must land on the modern spelling's sheet"
    );
}

#[test]
fn plain_examples_resolve_to_the_defaults() {
    for directory in [
        "basic-markdown",
        "job-specific-cv",
        "json-resume-import",
        "format-conversion",
        "structured-formats",
    ] {
        let source =
            std::fs::read_to_string(format!("../../docs/examples/{directory}/settings.json"))
                .expect("example settings");
        let resolved = resolve_settings(&source);
        assert_eq!(
            resolved,
            cac_render::defaults(),
            "{directory} has no style keys"
        );
    }
}

#[test]
fn deprecated_keys_carry_advisories_naming_their_replacement() {
    let settings = serde_json::from_str::<Settings>(
        r##"{"typography":{"font_size":"10pt"},"style":{"accent_color":"#1f4e79"}}"##,
    )
    .unwrap();
    let shimmed = shim(&settings);
    assert!(
        shimmed.warnings.iter().any(
            |warning| warning.contains("typography.font_size") && warning.contains("text.size")
        )
    );
    assert!(
        shimmed
            .warnings
            .iter()
            .any(|warning| warning.contains("style.accent_color")
                && warning.contains("colors.accent"))
    );
}

#[test]
fn an_unknown_style_key_reports_its_path() {
    let error = serde_json::from_str::<StyleOverrides>(r##"{"spacing": {"gutter": "1em"}}"##)
        .expect_err("unknown keys are errors")
        .to_string();
    assert!(error.contains("gutter"), "{error}");
}

#[test]
fn the_new_spelling_wins_over_a_deprecated_leaf() {
    let settings = serde_json::from_str::<Settings>(
        r##"{"style": {"accent_color": "#111111", "colors": {"accent": "#222222"}}}"##,
    )
    .unwrap();
    let shimmed = shim(&settings);
    let shim = cac_render::shim(&settings);
    let _ = shim;
    let resolved = resolve_layered(None, &shimmed).unwrap().0;
    assert_eq!(resolved.colors.accent, "#222222", "the modern group wins");
}

#[test]
fn theme_toml_layers_over_defaults_and_settings_win() {
    let theme: StyleOverrides =
        toml::from_str("[colors]\naccent = \"#1f4e79\"\n\n[header]\nalign = \"center\"\n").unwrap();
    let settings: StyleOverrides =
        serde_json::from_str(r##"{"header": {"align": "left"}}"##).unwrap();
    let resolved = cac_render::resolve(&theme, &settings).unwrap();
    assert_eq!(resolved.colors.accent, "#1f4e79", "theme over defaults");
    assert_eq!(
        resolved.header.align,
        cac_render::HeaderAlign::Left,
        "settings over theme"
    );
    assert_eq!(
        resolved.text.size,
        cac_render::Length::new(10.0, cac_render::LengthUnit::Pt),
        "defaults fill the rest"
    );
}

#[test]
fn toml_and_json_spellings_merge_alike() {
    let from_toml: StyleOverrides =
        toml::from_str("[colors]\naccent = \"#1f4e79\"\n\n[text]\nalign = \"left\"\n").unwrap();
    let from_json: StyleOverrides =
        serde_json::from_str(r##"{"colors": {"accent": "#1f4e79"}, "text": {"align": "left"}}"##)
            .unwrap();
    assert_eq!(from_toml, from_json, "the style table is format-agnostic");
}
