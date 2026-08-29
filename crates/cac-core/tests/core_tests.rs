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
