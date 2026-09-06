use cac_check::{Severity, check_content};
use cac_io::{InputFormat, parse};

#[test]
fn content_rules_report_stable_codes() {
    let source = "# Ada\n\nLondon\n\n## Experience\n\n### Engineer, Example\n2020–Present\n\n- Helped my team ship software\n";
    let cv = parse(source, InputFormat::Markdown).unwrap();
    let diagnostics = check_content(&cv);
    let codes: Vec<_> = diagnostics.iter().map(|value| value.code).collect();

    assert!(codes.contains(&"CAC101"));
    assert!(codes.contains(&"CAC201"));
    assert!(codes.contains(&"CAC202"));
    assert!(codes.contains(&"CAC301"));
    assert!(
        diagnostics
            .iter()
            .all(|value| value.severity == Severity::Warning)
    );
}

#[test]
fn skill_names_do_not_trigger_result_measurement_rules() {
    let source = "# Ada\n\nada@example.com\n\n## Skills\n\n- Rust and PostgreSQL\n";
    let cv = parse(source, InputFormat::Markdown).unwrap();

    assert!(check_content(&cv).is_empty());
}

#[test]
fn contact_omission_is_advisory_and_labeled_links_count_as_contact_methods() {
    let minimal = parse("# Anonymous", InputFormat::Markdown).unwrap();
    let diagnostics = check_content(&minimal);
    assert_eq!(diagnostics.len(), 1);
    assert_eq!(diagnostics[0].severity, Severity::Warning);
    for source in [
        "# A\nWebsite: https://example.com",
        "# A\nContact: [Work](mailto:a@example.com)",
    ] {
        assert!(check_content(&parse(source, InputFormat::Markdown).unwrap()).is_empty());
    }
}
