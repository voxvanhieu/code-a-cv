use super::*;

fn set(values: &[&str]) -> TagSet {
    values.iter().map(|value| (*value).to_string()).collect()
}

fn entry(tags: &[&str]) -> Entry {
    Entry {
        kind: EntryKind::Experience(ExperienceEntry {
            role: RichText::literal("Engineer"),
            ..Default::default()
        }),
        content: None,
        fields: FieldMap::new(),
        tags: set(tags),
        origin: Origin::default(),
    }
}

fn section(id: &str, section_tags: &[&str], entries: Vec<Entry>) -> Section {
    Section {
        id: id.into(),
        title: id.into(),
        kind: SectionKind::Experience,
        entries,
        tags: set(section_tags),
    }
}

fn document(sections: Vec<Section>) -> CvDocument {
    CvDocument {
        profile: Profile {
            name: "Ada Lovelace".into(),
            ..Default::default()
        },
        sections,
    }
}

#[test]
fn a_tagged_section_survives_whole() {
    let cv = document(vec![
        section(
            "backend",
            &["backend"],
            vec![entry(&[]), entry(&["frontend"])],
        ),
        section("frontend", &["frontend"], vec![entry(&[])]),
    ]);
    let variant = cv.variant(&set(&["backend"])).unwrap();
    assert_eq!(variant.sections.len(), 1);
    // The whole section, untagged entries included.
    assert_eq!(variant.sections[0].entries.len(), 2);
    assert_eq!(variant.profile.name, "Ada Lovelace");
}

#[test]
fn an_untagged_section_survives_through_tagged_entries() {
    let cv = document(vec![section(
        "experience",
        &[],
        vec![entry(&["leadership"]), entry(&[]), entry(&["backend"])],
    )]);
    let variant = cv.variant(&set(&["leadership"])).unwrap();
    assert_eq!(variant.sections.len(), 1);
    assert_eq!(variant.sections[0].entries.len(), 1);
    assert!(variant.sections[0].entries[0].tags.contains("leadership"));
}

#[test]
fn multiple_tags_union_across_sections() {
    let cv = document(vec![
        section("backend", &["backend"], vec![entry(&[])]),
        section("leadership", &["leadership"], vec![entry(&[])]),
        section("general", &[], vec![entry(&[])]),
    ]);
    let variant = cv.variant(&set(&["backend", "leadership"])).unwrap();
    assert_eq!(
        variant
            .sections
            .iter()
            .map(|section| section.id.as_str())
            .collect::<Vec<_>>(),
        ["backend", "leadership"]
    );
}

#[test]
fn nothing_matching_is_none() {
    let cv = document(vec![section("skills", &[], vec![entry(&["rust"])])]);
    assert!(cv.variant(&set(&["backend"])).is_none());
    assert!(
        cv.variant(&set(&[])).is_none(),
        "empty tag set matches nothing"
    );
}

#[test]
fn an_empty_document_is_none() {
    assert!(document(Vec::new()).variant(&set(&["backend"])).is_none());
}
