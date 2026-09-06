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
