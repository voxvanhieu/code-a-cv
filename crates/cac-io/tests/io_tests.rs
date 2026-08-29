use cac_core::{EntryKind, Inline};
use cac_io::{InputFormat, STARTER_MARKDOWN, parse, schema_json, to_markdown};

#[test]
fn starter_markdown_parses_into_typed_entries() {
    let cv = parse(STARTER_MARKDOWN, InputFormat::Markdown).unwrap();

    assert_eq!(cv.profile.name, "Ada Lovelace");
    assert_eq!(cv.profile.email.as_deref(), Some("ada@example.com"));
    assert_eq!(
        cv.profile.location.as_deref(),
        Some("London, United Kingdom")
    );
    assert!(matches!(
        cv.sections[0].entries[0].kind,
        EntryKind::Education(_)
    ));
    assert!(matches!(
        cv.sections[1].entries[0].kind,
        EntryKind::Experience(_)
    ));
}

#[test]
fn markdown_handles_adversarial_renderer_characters() {
    let source = "# A & B\n\na@example.com\n\n## Experience\n\n### C# Engineer, R&D 100%\n2020–Present\n\n- Maintained `~/path` and \\ tools by **35%**\n";
    let cv = parse(source, InputFormat::Markdown).unwrap();
    let highlight = cv.sections[0].entries[0].kind.highlights().first().unwrap();

    assert_eq!(highlight.plain(), "Maintained ~/path and \\ tools by 35%");
    assert!(
        highlight
            .0
            .iter()
            .any(|node| matches!(node, Inline::Code(_)))
    );
}

#[test]
fn markdown_round_trip_preserves_visible_content() {
    let cv = parse(STARTER_MARKDOWN, InputFormat::Markdown).unwrap();
    let reparsed = parse(&to_markdown(&cv), InputFormat::Markdown).unwrap();

    assert_eq!(reparsed.profile, cv.profile);
    assert_eq!(reparsed.sections, cv.sections);
}

#[test]
fn ambiguous_markdown_reports_the_line() {
    let error = parse(
        "# Ada\n\n## Experience\nunknown text\n",
        InputFormat::Markdown,
    )
    .unwrap_err();

    assert!(error.to_string().contains("line 4"));
}

#[test]
fn structured_formats_round_trip() {
    let cv = parse(STARTER_MARKDOWN, InputFormat::Markdown).unwrap();
    let json = serde_json::to_string(&cv).unwrap();
    let yaml = serde_yaml_ng::to_string(&cv).unwrap();
    let toml = toml::to_string(&cv).unwrap();

    assert_eq!(parse(&json, InputFormat::Json).unwrap(), cv);
    assert_eq!(parse(&yaml, InputFormat::Yaml).unwrap(), cv);
    assert_eq!(parse(&toml, InputFormat::Toml).unwrap(), cv);
}

#[test]
fn schema_describes_the_document_root() {
    let schema: serde_json::Value = serde_json::from_str(&schema_json().unwrap()).unwrap();

    assert_eq!(schema["title"], "CvDocument");
    assert!(schema["properties"]["profile"].is_object());
}

#[test]
fn markdown_round_trip_preserves_tags_on_bullet_entries() {
    let source = "# Ada\n\nada@example.com\n\n## Skills\n\n- Rust, C#, and R&D\n<!-- tags: backend, dotnet -->\n";
    let cv = parse(source, InputFormat::Markdown).unwrap();
    let reparsed = parse(&to_markdown(&cv), InputFormat::Markdown).unwrap();

    assert_eq!(reparsed.sections, cv.sections);
    assert!(reparsed.sections[0].entries[0].tags.contains("dotnet"));
}
