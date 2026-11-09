use cac_core::{DatePoint, Inline, Period, RichText};

#[test]
fn rich_text_preserves_inline_structure() {
    let value =
        RichText::parse("Built **C#** with *care*, `~/bin`, and [docs](https://example.com)");

    assert_eq!(value.plain(), "Built C# with care, ~/bin, and docs");
    assert!(value.0.iter().any(|item| matches!(item, Inline::Strong(_))));
    assert!(value.0.iter().any(|item| matches!(item, Inline::Code(_))));
    assert!(
        value
            .0
            .iter()
            .any(|item| matches!(item, Inline::Link { .. }))
    );
    assert_eq!(RichText::parse(&value.to_markdown()), value);
}

#[test]
fn date_points_have_stable_structured_representations() {
    assert_eq!(
        serde_json::to_string(&DatePoint::Year(2024)).unwrap(),
        "\"2024\""
    );
    assert_eq!(
        serde_json::to_string(&DatePoint::year_month(2024, 3).unwrap()).unwrap(),
        "\"2024-03\""
    );
    assert_eq!(
        serde_json::from_str::<DatePoint>("\"present\"").unwrap(),
        DatePoint::Present
    );
}

#[test]
fn period_rejects_reverse_dates() {
    assert!(Period::new(DatePoint::Year(2024), DatePoint::Year(2023)).is_err());
    assert!(Period::new(DatePoint::Year(2024), DatePoint::Present).is_ok());
}

#[test]
fn partial_periods_keep_unknown_endpoints_and_compare_date_precision() {
    let period = Period::partial(Some(DatePoint::Year(2022)), None).unwrap();
    assert_eq!(period.to_string(), "2022–");
    assert_eq!(
        serde_json::to_string(&period).unwrap(),
        r#"{"start":"2022"}"#
    );
    assert!(Period::partial(None, Some(DatePoint::Year(2024))).is_ok());
    assert!(Period::partial(None, None).is_err());
    assert!(Period::new(DatePoint::Present, DatePoint::Present).is_err());
    assert!(Period::new(DatePoint::YearMonth(2024, 12), DatePoint::Year(2024)).is_ok());
    assert!(
        Period::new(
            DatePoint::YearMonth(2024, 12),
            DatePoint::YearMonth(2024, 11)
        )
        .is_err()
    );
}

#[test]
fn rich_text_round_trips_literal_punctuation_and_nested_blocks() {
    for source in [
        r"literal \*stars\*, \[brackets\], \<tag\>, \\ and \#heading",
        "A paragraph.\n\nAnother **paragraph**.",
        "- Parent\n  - Child one\n  - Child two\n- Next",
        "3. First\n4. Second\n\nAfter the list.",
        "` a ` and ``a`b`` and `` `x` ``",
        "[A link](https://example.com/a_(b))",
        "Line one.\\\nLine two.",
        "*a*_b_ and __a__**b**",
        "`a`<!-- note -->``b``",
        r"\![a](https://example.com)",
        r"1\) literal numbering",
    ] {
        let rich = RichText::try_parse(source).unwrap();
        let serialized = rich.to_markdown();
        assert_eq!(
            RichText::try_parse(&serialized).unwrap(),
            rich,
            "{source}\n{serialized}"
        );
    }
}

#[test]
fn unsupported_rich_text_is_rejected_in_structured_documents() {
    for source in [
        "![photo](https://example.com/a.png)",
        "[unsafe](javascript:alert)",
        "<b>HTML</b>",
    ] {
        assert!(RichText::try_parse(source).is_err());
        assert!(serde_json::from_str::<RichText>(&serde_json::to_string(source).unwrap()).is_err());
        assert_eq!(RichText::parse(source).plain(), source);
    }
}

#[test]
fn plain_imports_keep_line_breaks_and_literal_markdown() {
    for source in [
        "First line.\nSecond line.",
        "First paragraph.\n\nSecond paragraph.",
        "---",
        "1) item",
        "*not emphasis*\n[not a link](https://example.com)",
    ] {
        let value = RichText::literal(source);
        assert_eq!(value.plain(), source);
        assert_eq!(
            RichText::try_parse(&value.to_markdown()).unwrap(),
            value,
            "{source}"
        );
    }
}

#[test]
fn entries_report_the_core_fields_their_typed_slots_carry() {
    let experience: cac_core::Entry = serde_json::from_str(
        r#"{"type":"experience","role":"Engineer","organization":"Acme","period":{"start":"2020","end":"2021"}}"#,
    )
    .unwrap();
    assert!(experience.carries_core_field("organization"));
    assert!(experience.carries_core_field("Organization"));
    assert!(experience.carries_core_field("role"));
    assert!(experience.carries_core_field("period"));
    assert!(
        !experience.carries_core_field("institution"),
        "fields outside the kind's scopes are not carried"
    );
    let bare: cac_core::Entry =
        serde_json::from_str(r#"{"type":"experience","role":"Engineer"}"#).unwrap();
    assert!(!bare.carries_core_field("organization"));
    assert!(!bare.carries_core_field("nonexistent-field"));

    let skill: cac_core::Entry =
        serde_json::from_str(r#"{"type":"skill-group","name":"Spoken","skills":["Japanese"]}"#)
            .unwrap();
    assert!(skill.carries_core_field("name"));
    assert!(skill.carries_core_field("skills"));
    assert!(
        !skill.carries_core_field("organization"),
        "a required organization must not fire on a skill group"
    );
    assert_eq!(skill.kind.kind_name(), cac_core::EntryKindName::SkillGroup);
}

#[test]
fn profiles_report_core_fields_through_their_typed_slots() {
    let profile: cac_core::Profile =
        serde_json::from_str(r#"{"name":"Ada","email":"ada@x.io","photo":"p.jpg"}"#).unwrap();
    assert!(profile.carries_core_field("email"));
    assert!(profile.carries_core_field("PHOTO"));
    assert!(!profile.carries_core_field("phone"));
    assert!(!profile.carries_core_field("address"));
    assert!(
        !profile.carries_core_field("empty"),
        "non-core keys fall back to the open fields"
    );
    let blank: cac_core::Profile =
        serde_json::from_str(r#"{"name":"Ada","email":"  ","fields":{"extra":"x"}}"#).unwrap();
    assert!(!blank.carries_core_field("email"), "blank text is missing");
    assert!(blank.carries_core_field("extra"));
}
