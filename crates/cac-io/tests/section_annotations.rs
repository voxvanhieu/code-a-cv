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
        "<!-- cac:section kind=invalid -->",
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
