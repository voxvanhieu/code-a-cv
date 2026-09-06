use cac_core::{EntryKind, SectionKind};
use cac_io::{InputFormat, parse, to_markdown};

#[test]
fn translated_headings_keep_explicit_identity_and_entry_semantics() {
    let source = "# Nguyễn Анна\n\n## Kinh nghiệm\n<!-- cac:section id=career kind=experience -->\n\n### Engineer, R&D\n2022–Present\n\n- Built **reliable** tools\n";
    let cv = parse(source, InputFormat::Markdown).unwrap();
    assert_eq!(cv.sections[0].id, "career");
    assert_eq!(cv.sections[0].kind, SectionKind::Experience);
    assert!(matches!(
        cv.sections[0].entries[0].kind,
        EntryKind::Experience(_)
    ));
    let mut renamed = cv.clone();
    renamed.sections[0].title = "Опыт работы <&>".into();
    renamed.sections[0].id = "career / kinh nghiệm --> %".into();
    assert_eq!(
        parse(&to_markdown(&renamed), InputFormat::Markdown).unwrap(),
        renamed
    );
}

#[test]
fn optional_annotation_fields_keep_legacy_defaults() {
    for (annotation, id, kind) in [
        ("", "experience", SectionKind::Experience),
        (
            "<!-- cac:section id=career -->",
            "career",
            SectionKind::Experience,
        ),
        (
            "<!-- cac:section kind=custom -->",
            "experience",
            SectionKind::Custom,
        ),
    ] {
        let cv = parse(
            &format!("# Ada\n\n## Experience\n{annotation}\n\n### Work\n"),
            InputFormat::Markdown,
        )
        .unwrap();
        assert_eq!(cv.sections[0].id, id);
        assert_eq!(cv.sections[0].kind, kind);
    }
}

#[test]
fn malformed_duplicate_and_misplaced_annotations_report_lines() {
    for suffix in [
        "<!-- cac:section kind=invalid! -->",
        "<!-- cac:section kind=skills kind=custom -->",
        "<!-- cac:section table=true -->",
        "<!-- cac:section id= -->",
        "<!-- cac:section id=%ZZ -->",
        "<!-- cac:section kind=skills",
        "<!-- cac:section -->",
        "<!-- cac:section kind=skills -->\n<!-- cac:section id=again -->",
        "### Entry\n<!-- cac:section kind=skills -->",
        "- Text\n<!-- cac:section kind=skills -->",
        "<!-- tags: section -->\n<!-- cac:section kind=skills -->",
    ] {
        let error = parse(
            &format!("# Ada\n\n## Section\n{suffix}\n"),
            InputFormat::Markdown,
        )
        .unwrap_err()
        .to_string();
        assert!(
            error.contains("line 4") || error.contains("line 5"),
            "{error}"
        );
    }
    assert!(
        parse(
            "# Ada\n<!-- cac:section kind=custom -->",
            InputFormat::Markdown
        )
        .is_err()
    );
}

#[test]
fn section_fields_support_theme_defined_kinds_and_native_round_trips() {
    let source = "# Nguyễn Анна\n\n## Credentials\nKind: certifications\nId: credentials / quốc tế\n\n### Cloud Engineer\nDate: 2024\n\n- Passed the exam.\n\n## Work\nKind: experience\nId: career\n\n### Engineer\nOrganization: Example\n";
    let cv = parse(source, InputFormat::Markdown).unwrap();
    assert_eq!(cv.sections[0].kind.as_str(), "certifications");
    assert_eq!(cv.sections[0].id, "credentials / quốc tế");
    assert!(matches!(
        cv.sections[0].entries[0].kind,
        EntryKind::Custom(_)
    ));
    assert!(matches!(
        cv.sections[1].entries[0].kind,
        EntryKind::Experience(_)
    ));
    let canonical = cac_io::to_markdown_checked(&cv).unwrap();
    assert!(canonical.contains("Kind: certifications\nId:"));
    assert!(!canonical.contains("cac:section"));
    assert_eq!(parse(&canonical, InputFormat::Markdown).unwrap(), cv);
    assert_eq!(
        cac_io::to_markdown_checked(&parse(&canonical, InputFormat::Markdown).unwrap()).unwrap(),
        canonical
    );
    for (source, format) in [
        (serde_json::to_string(&cv).unwrap(), InputFormat::Json),
        (serde_yaml_ng::to_string(&cv).unwrap(), InputFormat::Yaml),
        (toml::to_string(&cv).unwrap(), InputFormat::Toml),
    ] {
        assert_eq!(parse(&source, format).unwrap(), cv);
    }
}

#[test]
fn section_metadata_boundaries_defaults_and_conflicts_are_clear() {
    let cv = parse(
        "# A\n## Experience\nKind:\nId:\n\nKind: this is ordinary prose.\n",
        InputFormat::Markdown,
    )
    .unwrap();
    assert_eq!(cv.sections[0].kind, SectionKind::Experience);
    assert_eq!(cv.sections[0].id, "experience");
    assert!(
        cv.sections[0].entries[0]
            .kind
            .heading()
            .0
            .plain()
            .contains("Kind:")
    );
    for suffix in [
        "Kind: awards\nKind: education",
        "Id: one\nid: two",
        "Kind: bad value",
        "Knd: awards",
        "<!-- cac:section kind=custom -->\nKind: awards",
        "Id: same\n\n## Two\nId: same",
    ] {
        assert!(
            parse(&format!("# A\n## One\n{suffix}\n"), InputFormat::Markdown).is_err(),
            "{suffix}"
        );
    }
    let cv = parse(
        "# A\n## One\n<!-- cac:section kind=awards -->\nId: prizes\n",
        InputFormat::Markdown,
    )
    .unwrap();
    assert_eq!(cv.sections[0].kind.as_str(), "awards");
    assert_eq!(cv.sections[0].id, "prizes");
}
