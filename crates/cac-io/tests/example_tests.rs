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

    let json_resume = parse(
        include_str!("../../../examples/resume.json"),
        InputFormat::JsonResume,
    )
    .unwrap();
    assert_eq!(json_resume.profile, markdown.profile);
    assert_eq!(json_resume.sections.len(), markdown.sections.len());
    for (actual_section, expected_section) in json_resume.sections.iter().zip(&markdown.sections) {
        assert_eq!(actual_section.title, expected_section.title);
        assert_eq!(actual_section.kind, expected_section.kind);
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
