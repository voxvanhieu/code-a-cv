use cac_core::{DatePoint, EntryKind, Inline, SectionKind};
use cac_io::{InputFormat, parse, parse_markdown, to_markdown_checked};

const FLEXIBLE: &str = include_str!("fixtures/flexible.md");

#[test]
fn explicit_metadata_preserves_titles_contacts_and_optional_fields() {
    let cv = parse_markdown(FLEXIBLE).unwrap();
    assert_eq!(cv.profile.name, "Nguyễn Анна");
    assert_eq!(cv.profile.location.as_deref(), Some("Hà Nội"));
    assert_eq!(cv.profile.contacts.len(), 2);
    assert_eq!(
        cv.profile.contacts[1].href.as_ref().unwrap().as_str(),
        "mailto:work@example.com"
    );
    assert!(
        cv.profile
            .summary
            .as_ref()
            .unwrap()
            .plain()
            .contains("@company")
    );
    assert!(
        cv.profile
            .summary
            .as_ref()
            .unwrap()
            .plain()
            .contains("soft-wrapped summary.")
    );
    let career = &cv.sections[0];
    assert_eq!(career.kind, SectionKind::Experience);
    assert!(matches!(career.entries[0].kind, EntryKind::Prose(_)));
    let EntryKind::Experience(role) = &career.entries[1].kind else {
        panic!()
    };
    assert_eq!(role.role.plain(), "Director, Data and AI");
    assert_eq!(role.organization.plain(), "Acme, Inc.");
    assert_eq!(role.location.as_deref(), Some("Remote"));
    assert_eq!(
        role.period.as_ref().unwrap().start,
        Some(DatePoint::Year(2022))
    );
    assert_eq!(role.period.as_ref().unwrap().end, None);
    assert!(career.entries[1].tags.contains("a,b"));
    assert!(
        career.entries[1]
            .content
            .as_ref()
            .unwrap()
            .plain()
            .ends_with("A paragraph after the list.")
    );
    assert!(matches!(career.entries[3].kind, EntryKind::Project(_)));
    assert_eq!(cv.sections[3].id, "skills");
    assert_eq!(cv.sections[4].id, "skills-2");
}

#[test]
fn flexible_content_round_trips_through_every_native_format() {
    let cv = parse_markdown(FLEXIBLE).unwrap();
    let markdown = to_markdown_checked(&cv).unwrap();
    assert_eq!(parse_markdown(&markdown).unwrap(), cv);
    assert_eq!(
        to_markdown_checked(&parse_markdown(&markdown).unwrap()).unwrap(),
        markdown
    );
    for (source, format) in [
        (serde_json::to_string(&cv).unwrap(), InputFormat::Json),
        (serde_yaml_ng::to_string(&cv).unwrap(), InputFormat::Yaml),
        (toml::to_string(&cv).unwrap(), InputFormat::Toml),
    ] {
        assert_eq!(parse(&source, format).unwrap(), cv, "{format:?}");
    }
}

#[test]
fn omissions_and_blank_optional_values_build_in_all_native_formats() {
    for source in [
        "# Anonymous\n",
        "# Anonymous\nEmail:\nPhone:\nLocation:\nWebsite:\n\n## Experience\n\n### Consultant\nOrganization:\nPeriod:\n",
    ] {
        let cv = parse_markdown(source).unwrap();
        assert!(cv.profile.email.is_none());
        assert!(cv.profile.phone.is_none());
        assert_eq!(
            parse_markdown(&to_markdown_checked(&cv).unwrap()).unwrap(),
            cv
        );
    }
    for (source, format) in [
        (
            r#"{"profile":{"name":"A"},"sections":[{"id":"e","title":"E","entries":[{"type":"experience","role":"Consultant"},{"type":"education","qualification":"Self-taught"}]}]}"#,
            InputFormat::Json,
        ),
        (
            "profile:\n  name: A\nsections:\n- id: e\n  title: E\n  entries:\n  - type: experience\n    role: Consultant\n  - type: education\n    qualification: Self-taught\n",
            InputFormat::Yaml,
        ),
        (
            "[profile]\nname = 'A'\n[[sections]]\nid = 'e'\ntitle = 'E'\n[[sections.entries]]\ntype = 'experience'\nrole = 'Consultant'\n[[sections.entries]]\ntype = 'education'\nqualification = 'Self-taught'\n",
            InputFormat::Toml,
        ),
    ] {
        let cv = parse(source, format).unwrap();
        assert!(
            cv.sections[0]
                .entries
                .iter()
                .all(|entry| entry.kind.heading().1.unwrap().is_empty())
        );
    }
}

#[test]
fn metadata_typos_conflicts_and_incompatible_fields_report_lines() {
    for (body, expected) in [
        ("Organzation: Acme", "unknown entry field"),
        ("Kind: imaginary", "invalid entry Kind"),
        ("Kind: skills", "invalid entry Kind"),
        ("Period: 2023-15–Present", "invalid date"),
        ("Date: 03/04/2023", "invalid Date"),
        ("Date: Present", "invalid Date"),
        ("Period: Present–Present", "invalid period"),
        ("Period: 2024–2020", "invalid period"),
        ("Period: –", "invalid period"),
        ("Date: 2023\nPeriod: 2020–2024", "conflict"),
        ("Period: 2020–2024\n2021–2024", "duplicate"),
        ("Organization: One\nOrganization: Two", "duplicate"),
        ("Kind: skill-group\nPeriod: 2020–2024", "not supported"),
        ("Kind: publication\nPeriod: 2020–2024", "not supported"),
        ("URL: https://example.com", "not supported"),
        ("Kind: project\nURL: example.com", "invalid URL"),
    ] {
        let error = parse_markdown(&format!("# A\n\n## Experience\n\n### Work\n{body}\n"))
            .unwrap_err()
            .to_string();
        assert!(error.contains(expected), "{body}: {error}");
        assert!(error.contains("line "), "{error}");
    }
}

#[test]
fn legacy_dates_are_not_discarded_for_skills_or_publications() {
    for section in ["Skills", "Publications"] {
        assert!(
            parse_markdown(&format!("# A\n\n## {section}\n\n### Entry\n2020–2023\n"))
                .unwrap_err()
                .to_string()
                .contains("not supported")
        );
    }
}

#[test]
fn prose_does_not_become_metadata_after_a_blank_line() {
    let cv = parse_markdown("# A\n\nEmail: this sentence is prose.\n\n## Experience\n\n### Work\n\nImpact: improved reliability.\n").unwrap();
    assert!(cv.profile.email.is_none());
    assert_eq!(
        cv.profile.summary.unwrap().plain(),
        "Email: this sentence is prose."
    );
    assert_eq!(
        cv.sections[0].entries[0].content.as_ref().unwrap().plain(),
        "Impact: improved reliability."
    );
}

#[test]
fn localized_titles_and_generated_ids_do_not_guess_semantics() {
    let cv = parse_markdown("# A\n\n## Experience\n\n## Experience\n\n## Other\n<!-- cac:section id=experience-2 kind=education -->\n\n## Customer experience research\n\n## Опыт\n<!-- cac:section kind=experience -->\n").unwrap();
    assert_eq!(
        cv.sections
            .iter()
            .map(|s| s.id.as_str())
            .collect::<Vec<_>>(),
        [
            "experience",
            "experience-3",
            "experience-2",
            "customer-experience-research",
            "опыт"
        ]
    );
    assert_eq!(cv.sections[3].kind, SectionKind::Custom);
    assert_eq!(cv.sections[4].kind, SectionKind::Experience);
    let error = parse_markdown(
        "# A\n## One\n<!-- cac:section id=same -->\n## Two\n<!-- cac:section id=same -->",
    )
    .unwrap_err()
    .to_string();
    assert!(error.contains("line 5") && error.contains("line 3"));
}

#[test]
fn comments_are_ignored_but_reserved_directives_are_validated() {
    let cv = parse_markdown("<!-- an editing note -->\n# A\n\n<!-- multiline\n## not a section\n-->\n\n## Skills\n\n- Rust\n<!-- tags: rust -->\n").unwrap();
    assert_eq!(cv.sections.len(), 1);
    assert!(cv.sections[0].entries[0].tags.contains("rust"));
    for source in [
        "# A\n<!-- cac:unknown -->",
        "# A\n<!-- unfinished",
        "# A\n## S\n<!-- cac:entry kind=prose -->",
        "# A\n## S\n<!-- tags: [bad] -->",
    ] {
        assert!(parse_markdown(source).is_err(), "{source}");
    }
}

#[test]
fn unsupported_blocks_and_unsafe_links_fail_without_losing_content() {
    for content in [
        "![photo](https://example.com/a.png)",
        "```rust\nlet a = 1;\n```",
        "<script>alert(1)</script>",
        "[link](javascript:alert)",
        "[link](relative.html)",
        "| a | b |\n|---|---|\n| c | d |",
        "#### Subheading",
        "> A quotation",
        "[target]: https://example.com",
        r#"[link](https://example.com "tooltip")"#,
    ] {
        let error = parse_markdown(&format!("# A\n\n## Notes\n\n{content}\n"))
            .unwrap_err()
            .to_string();
        assert!(error.contains("line 5"), "{content}: {error}");
    }
}

#[test]
fn multiline_and_nested_bullets_keep_order_and_numbering() {
    let cv = parse_markdown(FLEXIBLE).unwrap();
    let body = cv.sections[0].entries[1].content.as_ref().unwrap();
    assert!(
        body.0
            .iter()
            .any(|node| matches!(node, Inline::List { start: Some(3), .. }))
    );
    let EntryKind::Experience(entry) = &cv.sections[0].entries[2].kind else {
        panic!()
    };
    assert_eq!(
        entry.highlights[0].plain(),
        "Advised small teams. Wrapped this achievement over two source lines."
    );
}

#[test]
fn singleton_contacts_never_overwrite_and_repeated_labeled_contacts_are_allowed() {
    for source in [
        "# A\nEmail: a@example.com\nEmail: b@example.com",
        "# A\n\na@example.com · b@example.com",
        "# A\n\nhttps://one.example · https://two.example",
    ] {
        assert!(
            parse_markdown(source)
                .unwrap_err()
                .to_string()
                .contains("duplicate")
        );
    }
    let cv = parse_markdown("# A\nContact: [One](mailto:a@example.com)\nContact: [Two](mailto:b@example.com)\nPhone: 090 123 4567\n").unwrap();
    assert_eq!(cv.profile.contacts.len(), 2);
    assert_eq!(cv.profile.phone.as_deref(), Some("090 123 4567"));
}

#[test]
fn canonical_export_preserves_every_entry_kind_in_any_section() {
    let cv = parse(r#"{"profile":{"name":"A # & <B>"},"sections":[{"id":"skills","title":"Skills","kind":"skills","entries":[{"type":"text","body":"plain"},{"type":"custom","heading":"custom"},{"type":"experience","role":"Director, Data","organization":"A, B","location":"Remote"},{"type":"education","qualification":"Study"},{"type":"publication","title":"Paper","publisher":"Press","url":"https://example.com/paper","date":"2024"},{"type":"project","name":"Tool","url":"https://example.com/tool"},{"type":"skill-group","name":"Rust"},{"type":"prose","body":"Paragraph one.\n\nParagraph two."}]}]}"#, InputFormat::Json).unwrap();
    assert_eq!(
        parse_markdown(&to_markdown_checked(&cv).unwrap()).unwrap(),
        cv
    );
}

#[test]
fn json_resume_import_rejects_invalid_dates_and_export_rejects_information_loss() {
    for date in ["2023-15", "nonsense"] {
        let source = format!(
            r#"{{"basics":{{"name":"A"}},"work":[{{"name":"Company","position":"Role","startDate":"{date}"}}]}}"#
        );
        assert!(parse(&source, InputFormat::JsonResume).is_err());
    }
    let source = r#"{"basics":{"name":"A"},"work":[{"name":"Company","position":"Role","startDate":"2023"}]}"#;
    let cv = parse(source, InputFormat::JsonResume).unwrap();
    assert!(
        cv.sections[0].entries[0]
            .kind
            .period()
            .unwrap()
            .end
            .is_none()
    );
    assert!(cac_io::export_json_resume_checked(&cv).is_ok());
    let full = parse_markdown(FLEXIBLE).unwrap();
    assert!(
        cac_io::export_json_resume_checked(&full)
            .unwrap_err()
            .to_string()
            .contains("cannot preserve")
    );
}

#[test]
fn contact_shorthand_does_not_consume_links_or_url_sentences_as_email_or_website() {
    for summary in [
        "[Work](mailto:work@example.com)",
        "https://example.com/path is where I write.",
    ] {
        let cv = parse_markdown(&format!("# A\n\n{summary}\n")).unwrap();
        assert!(cv.profile.email.is_none());
        assert!(cv.profile.website.is_none());
        assert!(cv.profile.summary.is_some());
        assert_eq!(
            parse_markdown(&to_markdown_checked(&cv).unwrap()).unwrap(),
            cv
        );
    }
}

#[test]
fn repeated_direct_entry_tags_stay_on_the_entry() {
    let cv = parse_markdown("# A\n\n## Skills\n\n- Rust\n<!-- tags: one -->\n<!-- tags: two -->\n")
        .unwrap();
    assert!(cv.sections[0].tags.is_empty());
    assert_eq!(cv.sections[0].entries[0].tags.len(), 2);
    assert_eq!(
        parse_markdown(&to_markdown_checked(&cv).unwrap()).unwrap(),
        cv
    );
}

#[test]
fn json_resume_multiline_plain_summary_exports_without_losing_text() {
    let source =
        r#"{"basics":{"name":"A","summary":"First *literal* line.\n\nSecond paragraph."}}"#;
    let cv = parse(source, InputFormat::JsonResume).unwrap();
    assert_eq!(
        cv.profile.summary.as_ref().unwrap().plain(),
        "First *literal* line.\n\nSecond paragraph."
    );
    assert_eq!(
        parse_markdown(&to_markdown_checked(&cv).unwrap()).unwrap(),
        cv
    );
}
