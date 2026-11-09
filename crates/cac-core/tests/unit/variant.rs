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
fn a_tagged_section_keeps_shared_and_matching_entries() {
    let cv = document(vec![
        section(
            "backend",
            &["backend"],
            vec![entry(&[]), entry(&["frontend"]), entry(&["backend"])],
        ),
        section("frontend", &["frontend"], vec![entry(&[])]),
    ]);
    let variant = cv.variant(&set(&["backend"])).unwrap();
    assert_eq!(variant.sections.len(), 1);
    // The frontend-tagged entry drops: an entry tagged for another job
    // never leaks into this variant, even from a matching section.
    let roles: Vec<_> = variant.sections[0]
        .entries
        .iter()
        .map(|entry| entry.tags.clone())
        .collect();
    assert_eq!(roles.len(), 2);
    assert!(
        roles
            .iter()
            .all(|tags| tags.is_empty() || tags.contains("backend"))
    );
    assert_eq!(variant.profile.name, "Ada Lovelace");
}

#[test]
fn an_untagged_section_is_shared_and_survives_every_variant() {
    // Education carries no tags: it belongs to every variant, not just to
    // the ones its entries name.
    let education = Section {
        id: "education".into(),
        title: "Education".into(),
        kind: SectionKind::Education,
        entries: vec![entry(&[])],
        tags: TagSet::new(),
    };
    let cv = document(vec![
        section(
            "experience",
            &[],
            vec![entry(&["leadership"]), entry(&[]), entry(&["backend"])],
        ),
        education,
    ]);
    let variant = cv.variant(&set(&["leadership"])).unwrap();
    let ids: Vec<_> = variant
        .sections
        .iter()
        .map(|section| section.id.as_str())
        .collect();
    assert_eq!(ids, ["experience", "education"]);
    // Inside the shared section: the leadership entry stays, the
    // backend-only entry drops, the untagged entry stays.
    assert_eq!(variant.sections[0].entries.len(), 2);
    assert!(variant.sections[0].entries[0].tags.contains("leadership"));
    // And the same document builds the other variant with education too.
    let variant = cv.variant(&set(&["backend"])).unwrap();
    let ids: Vec<_> = variant
        .sections
        .iter()
        .map(|section| section.id.as_str())
        .collect();
    assert_eq!(ids, ["experience", "education"]);
}

#[test]
fn a_section_tagged_for_another_job_drops_whole() {
    let cv = document(vec![
        section("platform", &["backend"], vec![entry(&[])]),
        section("web", &["frontend"], vec![entry(&[])]),
    ]);
    let variant = cv.variant(&set(&["frontend"])).unwrap();
    let ids: Vec<_> = variant
        .sections
        .iter()
        .map(|section| section.id.as_str())
        .collect();
    assert_eq!(ids, ["web"]);
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
        ["backend", "leadership", "general"]
    );
}

#[test]
fn only_tagged_variant_keeps_the_strict_spelling() {
    // `--only-tagged` is the 0.4 strict projection: untagged sections and
    // entries drop; only explicitly tagged content survives.
    let cv = document(vec![
        section(
            "platform",
            &["backend"],
            vec![entry(&[]), entry(&["backend"])],
        ),
        section("general", &[], vec![entry(&[])]),
    ]);
    let variant = cv.only_tagged_variant(&set(&["backend"])).unwrap();
    assert_eq!(variant.sections.len(), 1);
    assert_eq!(variant.sections[0].entries.len(), 2);
    // The strict projection has no shared content at all.
    let strict = document(vec![section("general", &[], vec![entry(&[])])]);
    assert!(strict.only_tagged_variant(&set(&["backend"])).is_none());
    // While the shared projection keeps the untagged section.
    assert!(strict.variant(&set(&["backend"])).is_some());
}

#[test]
fn nothing_matching_is_none() {
    let cv = document(vec![section("skills", &[], vec![entry(&["rust"])])]);
    assert!(cv.variant(&set(&["backend"])).is_none());
    assert!(
        cv.variant(&set(&[])).is_none(),
        "an empty tag set matches nothing"
    );
    assert!(
        cv.only_tagged_variant(&set(&[])).is_none(),
        "an empty tag set matches nothing"
    );
}

#[test]
fn an_empty_document_is_none() {
    assert!(document(Vec::new()).variant(&set(&["backend"])).is_none());
}
