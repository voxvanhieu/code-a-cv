use super::*;

#[test]
fn defaults_resolve_and_validate() {
    let resolved = defaults();
    assert_eq!(resolved.page.paper, Paper::A4);
    assert_eq!(resolved.page.margin.top, Length::new(18.0, LengthUnit::Mm));
    assert_eq!(resolved.fonts.body, "Libertinus Serif");
    assert_eq!(resolved.fonts.mono, "DejaVu Sans Mono");
    assert_eq!(resolved.text.size, Length::new(10.0, LengthUnit::Pt));
    assert_eq!(resolved.text.leading, Length::new(0.65, LengthUnit::Em));
    assert_eq!(
        resolved.text.paragraph_gap,
        Length::new(1.2, LengthUnit::Em)
    );
    assert_eq!(resolved.text.align, TextAlign::Justified);
    assert_eq!(resolved.colors.text, "#000000");
    assert_eq!(resolved.colors.muted, "#aaaaaa");
    // The rule color defaults to the accent token.
    assert_eq!(resolved.colors.rule, resolved.colors.accent);
    assert_eq!(resolved.headings.name.weight, Weight::Bold);
    assert_eq!(
        resolved.headings.name.size, None,
        "the name inherits the body size"
    );
    assert_eq!(
        resolved.headings.name.space_after,
        Length::new(1.2, LengthUnit::Em)
    );
    assert_eq!(
        resolved.headings.section.space_after,
        Length::new(0.65, LengthUnit::Em)
    );
    assert_eq!(
        resolved.headings.entry.space_after,
        Length::new(0.65, LengthUnit::Em)
    );
    assert_eq!(resolved.spacing.section, Length::new(1.2, LengthUnit::Em));
    assert_eq!(resolved.spacing.entry, Length::new(1.2, LengthUnit::Em));
    assert_eq!(resolved.spacing.item, None, "item spacing defaults to auto");
    assert_eq!(resolved.lists.bullet, "•");
    assert!(resolved.links.underline);
    assert!(
        resolved.entry.keep_together,
        "entries are kept together by default"
    );
    assert_eq!(resolved.locale, "en");
}

#[test]
fn defaults_round_trip_through_json() {
    let resolved = defaults();
    let json = serde_json::to_string(&resolved).unwrap();
    let restored: ResolvedStyle = serde_json::from_str(&json).unwrap();
    assert_eq!(restored, resolved);
}

#[test]
fn legacy_map_covers_all_twenty_style_keys() {
    let legacy_keys: Vec<&str> = LEGACY_SETTINGS_MAP.iter().map(|entry| entry.old).collect();
    assert_eq!(
        legacy_keys,
        [
            "page.paper",
            "page.margin",
            "page.margins.top",
            "page.margins.bottom",
            "page.margins.left",
            "page.margins.right",
            "typography.font",
            "typography.heading_font",
            "typography.font_size",
            "typography.line_spacing",
            "style.accent_color",
            "style.body_alignment",
            "style.header_alignment",
            "style.link_underline",
            "style.highlight_bullet",
            "spacing.list_item",
            "spacing.section",
            "spacing.section_heading",
            "spacing.entry",
            "pagination.allow_entry_page_break",
        ]
    );
    // Project settings are not style keys and must not appear in the map.
    for key in ["root", "naming", "theme", "themeProject"] {
        assert!(
            legacy_key(key).is_none(),
            "`{key}` is a project key, not a style key"
        );
    }
}

#[test]
fn legacy_map_targets_use_known_groups() {
    const GROUPS: [&str; 11] = [
        "page", "fonts", "colors", "text", "header", "headings", "spacing", "lists", "links",
        "entry", "locale",
    ];
    for entry in LEGACY_SETTINGS_MAP {
        let group = entry.new.split('.').next().expect("new key has a group");
        assert!(
            GROUPS.contains(&group),
            "`{}` targets unknown group `{group}`",
            entry.old
        );
    }
    // The inverted mapping is flagged so the shim flips the boolean.
    let page_break = legacy_key("pagination.allow_entry_page_break").unwrap();
    assert_eq!(page_break.note, "inverted");
    assert_eq!(page_break.new, "entry.keep_together");
}

#[test]
fn style_overrides_parse_the_reviewed_vocabulary() {
    let source = r##"{
        "page": { "paper": "us-letter", "margin": { "top": "12.7mm", "bottom": "12.7mm" } },
        "fonts": { "body": "New Computer Modern" },
        "colors": { "accent": "#1f4e79", "rule": "accent" },
        "text": { "size": "10pt", "align": "justified" },
        "header": { "align": "center", "rule": "line" },
        "headings": { "section": { "size": "12pt", "weight": "regular", "color": "accent", "transform": "caps", "rule": "line" } },
        "spacing": { "section": "1.2em", "item": "auto" },
        "lists": { "bullet": "–" },
        "links": { "underline": false },
        "entry": { "date_position": "left", "subtitle_style": "muted", "keep_together": false },
        "locale": "vi"
    }"##;
    let sheet: StyleOverrides = serde_json::from_str(source).unwrap();
    assert_eq!(sheet.page.as_ref().unwrap().paper, Some(Paper::UsLetter));
    assert_eq!(
        sheet
            .colors
            .as_ref()
            .unwrap()
            .accent
            .as_ref()
            .unwrap()
            .as_hex(),
        Some("#1f4e79")
    );
    assert_eq!(
        sheet
            .headings
            .as_ref()
            .unwrap()
            .section
            .as_ref()
            .unwrap()
            .transform,
        Some(HeadingTransform::Caps)
    );
    assert_eq!(
        sheet.entry.as_ref().unwrap().date_position,
        Some(DatePosition::Left)
    );
    assert_eq!(sheet.locale.as_deref(), Some("vi"));

    // `auto` item spacing resolves to None.
    let item = sheet.spacing.as_ref().unwrap().item.as_ref().unwrap();
    assert_eq!(item.parse_or_auto().unwrap(), None);
    assert!(item.parse().is_err(), "`auto` is not a length");

    // Unknown keys are rejected with the offending key named.
    let error =
        serde_json::from_str::<StyleOverrides>(r#"{"spacing": {"gutter": "1em"}}"#).unwrap_err();
    assert!(
        error.to_string().contains("gutter"),
        "error names the unknown key: {error}"
    );
}

#[test]
fn theme_manifest_parses_the_classic_blue_example() {
    let source = r##"
        [theme]
        name = "classic-blue"
        description = "The classic centered CV with blue headings"
        author = "Code a CV contributors"
        license = "MIT"
        extends = "classic"

        [style.colors]
        accent = "#1f4e79"

        [style.headings.section]
        color = "accent"

        [options]
        date_width = "30mm"
    "##;
    let manifest: ThemeManifest = toml::from_str(source).unwrap();
    assert_eq!(manifest.theme.name, "classic-blue");
    assert_eq!(manifest.theme.extends.as_deref(), Some("classic"));
    assert_eq!(
        manifest.theme.kind, None,
        "absent kind defaults to universal"
    );
    assert_eq!(
        manifest
            .style
            .colors
            .as_ref()
            .unwrap()
            .accent
            .as_ref()
            .unwrap()
            .as_hex(),
        Some("#1f4e79")
    );
    assert_eq!(
        manifest.options.get("date_width"),
        Some(&OptionValue::Text("30mm".into()))
    );
}

#[test]
fn form_manifest_parses_regions_and_requires() {
    let source = r##"
        [theme]
        name = "rirekisho"
        kind = "form"
        description = "JIS-style 履歴書, two A4 pages"

        [style]
        locale = "ja"

        [style.colors]
        accent = "#333333"

        [layout]
        photo = { region = "header-right", shape = "circle", size = "32mm" }

        [layout.regions]
        main = ["education", "experience", "*"]

        [layout.options]
        levels = "label"
        max_pages = 2
        ats = "friendly"

        [requires]
        packs = ["jp-rirekisho"]
        sections = ["education", "experience", "certifications"]
        profile = ["photo", "birth_date", "furigana"]
    "##;
    let manifest: ThemeManifest = toml::from_str(source).unwrap();
    assert_eq!(manifest.theme.kind, Some(ThemeKind::Form));
    assert_eq!(manifest.style.locale.as_deref(), Some("ja"));
    let photo = manifest.layout.photo.as_ref().unwrap();
    assert_eq!(photo.region, "header-right");
    assert_eq!(photo.shape, Some(PhotoShape::Circle));
    assert_eq!(
        photo.size.as_ref().unwrap().parse().unwrap().unit,
        LengthUnit::Mm
    );
    assert_eq!(
        manifest.layout.regions.get("main").unwrap(),
        &[
            "education".to_string(),
            "experience".to_string(),
            "*".to_string()
        ]
    );
    let options = &manifest.layout.options;
    assert_eq!(options.levels, Some(LevelStyle::Label));
    assert_eq!(options.max_pages, Some(2));
    assert_eq!(options.ats, Some(AtsMode::Friendly));
    let requires = manifest.requires.as_ref().unwrap();
    assert_eq!(requires.packs, ["jp-rirekisho"]);
    assert_eq!(requires.profile, ["photo", "birth_date", "furigana"]);
}

#[test]
fn manifest_without_a_theme_name_is_rejected() {
    let error =
        toml::from_str::<ThemeManifest>("[style.colors]\naccent = \"#000000\"\n").unwrap_err();
    assert!(
        error.to_string().contains("theme"),
        "missing table is reported: {error}"
    );
}

#[test]
fn schemas_have_editor_metadata_and_expected_properties() {
    let style = style_schema_document();
    assert_eq!(
        style["$schema"],
        "https://json-schema.org/draft/2020-12/schema"
    );
    assert!(
        style["$id"]
            .as_str()
            .unwrap()
            .ends_with("style.schema.json")
    );
    for property in [
        "page", "fonts", "colors", "text", "header", "headings", "spacing", "lists", "links",
        "entry", "locale",
    ] {
        assert!(
            style["properties"].get(property).is_some(),
            "style schema has `{property}`"
        );
    }
    assert_eq!(
        style["additionalProperties"], false,
        "unknown keys are errors"
    );

    let theme = theme_schema_document();
    for property in ["theme", "style", "options", "layout", "requires"] {
        assert!(
            theme["properties"].get(property).is_some(),
            "theme schema has `{property}`"
        );
    }
    // theme.name is required; JSON Schema 2020-12 lists required names as a
    // sibling of `properties`, with dotted paths for nested objects.
    let required: Vec<String> = theme["required"]
        .as_array()
        .unwrap_or(&Vec::new())
        .iter()
        .map(|value| value.as_str().unwrap_or_default().to_string())
        .collect();
    assert!(
        required
            .iter()
            .any(|path| path == "theme.name" || path == "theme"),
        "theme (or theme.name) is required, got {required:?}"
    );
}

#[test]
fn schema_documents_match_committed_files() {
    let cases = [
        ("style.schema.json", style_schema_document()),
        ("theme.schema.json", theme_schema_document()),
    ];
    for (file, generated) in cases {
        let path = format!("{}/schema/{file}", env!("CARGO_MANIFEST_DIR"));
        let canonical = canonical_schema(&generated);
        match std::fs::read_to_string(&path) {
            Ok(committed) => {
                let committed: serde_json::Value = serde_json::from_str(&committed).unwrap();
                assert_eq!(
                    canonical_schema(&committed),
                    canonical,
                    "`{file}` drifted from the generated schema; regenerate it (see the task notes) or update the contract deliberately"
                );
            }
            Err(_) => {
                std::fs::write(&path, format!("{canonical}\n")).expect("write schema");
                panic!("`{file}` was missing and has been written; commit it");
            }
        }
    }
}

/// Compare semantically: pretty-printed JSON with sorted keys.
fn canonical_schema(value: &serde_json::Value) -> String {
    fn sort(value: &serde_json::Value) -> serde_json::Value {
        match value {
            serde_json::Value::Object(map) => {
                let sorted: serde_json::Map<String, serde_json::Value> = map
                    .iter()
                    .map(|(key, value)| (key.clone(), sort(value)))
                    .collect();
                serde_json::Value::Object(sorted)
            }
            serde_json::Value::Array(items) => {
                serde_json::Value::Array(items.iter().map(sort).collect())
            }
            other => other.clone(),
        }
    }
    serde_json::to_string(&sort(value)).unwrap()
}
