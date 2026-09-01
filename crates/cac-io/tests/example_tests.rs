use cac_core::{CvDocument, EntryKind, SectionKind};
use cac_io::{InputFormat, parse};

#[test]
fn supported_input_examples_parse() {
    let markdown = parse(
        include_str!("../../../examples/cv.md"),
        InputFormat::Markdown,
    )
    .unwrap();

    for (source, format) in [
        (include_str!("../../../examples/cv.yaml"), InputFormat::Yaml),
        (include_str!("../../../examples/cv.json"), InputFormat::Json),
        (include_str!("../../../examples/cv.toml"), InputFormat::Toml),
    ] {
        assert_eq!(parse(source, format).unwrap(), markdown);
    }

    for kind in [
        SectionKind::Experience,
        SectionKind::Education,
        SectionKind::Projects,
        SectionKind::Publications,
        SectionKind::Skills,
        SectionKind::Custom,
    ] {
        assert!(markdown.sections.iter().any(|section| section.kind == kind));
    }
    assert_all_entry_kinds(&markdown);

    let json_resume = parse(
        include_str!("../../../examples/resume.json"),
        InputFormat::JsonResume,
    )
    .unwrap();
    assert_eq!(json_resume.profile, markdown.profile);
    assert_eq!(json_resume.sections.len(), 3);
    for kind in [
        SectionKind::Experience,
        SectionKind::Education,
        SectionKind::Skills,
    ] {
        assert!(
            json_resume
                .sections
                .iter()
                .any(|section| section.kind == kind)
        );
    }
    for actual_section in &json_resume.sections {
        let expected_section = markdown
            .sections
            .iter()
            .find(|section| section.kind == actual_section.kind)
            .unwrap();
        assert_eq!(actual_section.title, expected_section.title);
        assert_eq!(actual_section.entries.len(), expected_section.entries.len());
        for (actual_entry, expected_entry) in
            actual_section.entries.iter().zip(&expected_section.entries)
        {
            let (actual_primary, actual_secondary) = actual_entry.kind.heading();
            let (expected_primary, expected_secondary) = expected_entry.kind.heading();
            assert_eq!(actual_primary.plain(), expected_primary.plain());
            assert_eq!(
                actual_secondary.map(|value| value.plain()),
                expected_secondary.map(|value| value.plain())
            );
            assert_eq!(actual_entry.kind.period(), expected_entry.kind.period());
            assert_eq!(
                actual_entry
                    .kind
                    .highlights()
                    .iter()
                    .map(|value| value.plain())
                    .collect::<Vec<_>>(),
                expected_entry
                    .kind
                    .highlights()
                    .iter()
                    .map(|value| value.plain())
                    .collect::<Vec<_>>()
            );
        }
    }
}

fn assert_all_entry_kinds(cv: &CvDocument) {
    let mut present = [false; 7];
    for entry in cv.sections.iter().flat_map(|section| &section.entries) {
        let index = match &entry.kind {
            EntryKind::Experience(_) => 0,
            EntryKind::Education(_) => 1,
            EntryKind::Project(_) => 2,
            EntryKind::Publication(_) => 3,
            EntryKind::SkillGroup(_) => 4,
            EntryKind::Custom(_) => 5,
            EntryKind::Text(_) => 6,
        };
        present[index] = true;
    }
    assert!(present.into_iter().all(|value| value));
}
