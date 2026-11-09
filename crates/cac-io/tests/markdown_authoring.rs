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
        // A typo'd key alone is prose, not an error; check advises on it.
        ("Organzation: Acme", "PROSE"),
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
        ("Kind: project\nURL: example.com", "invalid URL"),
        // Not a payload of experience entries: the line stays prose.
        ("URL: https://example.com", "PROSE"),
    ] {
        if expected == "PROSE" {
            // A key outside the kind's scope is not a field; the paragraph
            // stays prose and check will advise, not error.
            let cv =
                parse_markdown(&format!("# A\n\n## Experience\n\n### Work\n{body}\n")).unwrap();
            assert_eq!(
                cv.sections[0].entries[0].content.as_ref().unwrap().plain(),
                body
            );
            continue;
        }
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
fn field_blocks_depend_on_content_never_on_blank_lines() {
    // Rule F: a paragraph is a field block by its lines alone, so a known
    // key is a field wherever a formatter puts it. A lone unknown key is
    // prose, and stays prose after any formatting.
    let cv = parse_markdown("# A\n\nEmail: this sentence is prose.\n\n## Experience\n\n### Work\n\nImpact: improved reliability.\n").unwrap();
    assert_eq!(cv.profile.email.as_deref(), Some("this sentence is prose."));
    assert_eq!(
        cv.sections[0].entries[0].content.as_ref().unwrap().plain(),
        "Impact: improved reliability."
    );
    // The same fields parse identically with a formatter's blank lines.
    let formatted = parse_markdown("# A\n\nEmail: this sentence is prose.\n\n## Experience\n\n### Work\n\nImpact: improved reliability.\n").unwrap();
    assert_eq!(formatted, cv);
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

#[test]
fn a_key_with_a_stray_space_before_the_colon_parses_without_panicking() {
    // `Email : ada@x.com` normalized once: the key is known, the slot is
    // filled, and no `unreachable!` fires.
    let cv = parse_markdown("# Ada\n\nEmail : ada@x.com\n").unwrap();
    assert_eq!(cv.profile.email.as_deref(), Some("ada@x.com"));
}

#[test]
fn lookalike_list_lines_stay_with_their_own_entries() {
    // `- Go` must not inherit `- Google Cloud`'s position because the text
    // is a substring of it.
    let cv = parse_markdown(
        "# A\n\n## Skills\n\n### Cloud\n\n- Google Cloud\n\n### Languages\n\n- Go\n",
    )
    .unwrap();
    let skills: Vec<Vec<String>> = cv.sections[0]
        .entries
        .iter()
        .map(|entry| match &entry.kind {
            cac_core::EntryKind::SkillGroup(group) => {
                group.skills.iter().map(|skill| skill.plain()).collect()
            }
            _ => Vec::new(),
        })
        .collect();
    assert_eq!(
        skills,
        [vec!["Google Cloud".to_owned()], vec!["Go".to_owned()]]
    );
}

#[test]
fn an_explicit_summary_and_summary_prose_are_a_duplicate_error() {
    let error = parse_markdown("# A\n\nSummary: Pioneer.\n\nI also write.\n")
        .unwrap_err()
        .to_string();
    assert!(error.contains("duplicate summary"), "{error}");
    // Either spelling alone keeps working.
    let cv = parse_markdown("# A\n\nSummary: Pioneer.\n").unwrap();
    assert_eq!(cv.profile.summary.as_ref().unwrap().plain(), "Pioneer.");
    let cv = parse_markdown("# A\n\nI also write.\n").unwrap();
    assert_eq!(
        cv.profile.summary.as_ref().unwrap().plain(),
        "I also write."
    );
}

#[test]
fn profile_only_keys_under_an_entry_stay_prose() {
    // `Summary:` is not an experience field; the line keeps its text.
    let cv = parse_markdown(
        "# A\n\n## Experience\n\n### Work\n\nKind: experience\nOrganization: Acme\nPeriod: 2020–2021\n\nSummary: Led the migration to Rust.\n",
    )
    .unwrap();
    assert_eq!(
        cv.sections[0].entries[0].content.as_ref().unwrap().plain(),
        "Summary: Led the migration to Rust."
    );
    assert!(cv.sections[0].entries[0].fields.is_empty());
}

#[test]
fn setext_headings_and_closing_hashes_keep_their_text() {
    let cv = parse_markdown("# Ada\n===\n\n## Experience ##\n\n### C#\n\n- Ships\n").unwrap();
    assert_eq!(cv.profile.name, "Ada");
    assert_eq!(cv.sections[0].title, "Experience");
    assert_eq!(cv.sections[0].entries[0].kind.heading().0.plain(), "C#");
}

#[test]
fn comments_inside_fenced_code_survive_and_text_after_a_comment_close_keeps() {
    // The fence shields the comment-looking literal; the trailing words
    // after `-->` are content a formatter must keep.
    // The fenced literal reaches phase two intact — the code block is
    // rejected loudly instead of being silently emptied by comment
    // stripping.
    let error = parse_markdown("# A\n\n```\n<!-- not a directive -->\n```\n")
        .unwrap_err()
        .to_string();
    assert!(error.contains("code blocks are not supported"), "{error}");
    // Text after a comment close on the same line stays content.
    let cv = parse_markdown("# A\n\n<!-- a plain note --> with trailing text\n").unwrap();
    assert_eq!(
        cv.profile.summary.as_ref().unwrap().plain(),
        "with trailing text"
    );
}

#[test]
fn address_segments_keep_their_slots_when_leading_parts_are_absent() {
    let cv = parse(
        r#"{"profile":{"name":"A","address":{"city":"Tokyo","country":"Japan"}},"sections":[]}"#,
        InputFormat::Json,
    )
    .unwrap();
    let written = to_markdown_checked(&cv).unwrap();
    assert!(written.contains("Address: , Tokyo, , , Japan"), "{written}");
    let reparsed = parse_markdown(&written).unwrap();
    assert_eq!(reparsed, cv);
}

#[test]
fn tags_round_trip_without_escaping_artifacts() {
    // Markdown normalizes tag spelling to lowercase (tags are build
    // selectors), so the round trip uses normalized tags; `c#` still
    // proves the hash survives unescaped.
    let cv = parse(r#"{"profile":{"name":"A"},"sections":[{"id":"s","title":"Experience","kind":"experience","tags":["c#","backend"],"entries":[]}]}"#, InputFormat::Json).unwrap();
    let written = to_markdown_checked(&cv).unwrap();
    // Tags render in their canonical (sorted) order.
    assert!(written.contains("Tags: backend, c#"), "{written}");
    assert_eq!(parse_markdown(&written).unwrap(), cv);
    // Uppercase tags convert honestly: the checked writer refuses rather
    // than silently rewriting the spelling.
    let cv = parse(r#"{"profile":{"name":"A"},"sections":[{"id":"s","title":"Experience","kind":"experience","tags":["Backend"],"entries":[]}]}"#, InputFormat::Json).unwrap();
    assert!(to_markdown_checked(&cv).is_err());
}

#[test]
fn numeric_proficiency_round_trips_through_markdown() {
    let cv = parse(r#"{"profile":{"name":"A"},"sections":[{"id":"langs","title":"Languages","kind":"language","entries":[{"type":"language","name":"German","proficiency":4}]}]}"#, InputFormat::Json).unwrap();
    let written = to_markdown_checked(&cv).unwrap();
    assert!(written.contains("Proficiency: 4"), "{written}");
    assert_eq!(parse_markdown(&written).unwrap(), cv);
}
