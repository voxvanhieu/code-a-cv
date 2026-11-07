//! The V1 acceptance: the view emits the C3 named slots, open fields, and
//! structured dates; the corpus matches the slot names; date snapshots in
//! five locales.

use cac_io::{parse, InputFormat};

fn corpus(name: &str) -> cac_core::CvDocument {
    let source = std::fs::read_to_string(format!(
        "../../crates/cac-io/tests/fixtures/markdown/{name}.md"
    ))
    .unwrap_or_else(|_| {
        std::fs::read_to_string(format!("../../crates/cac-io/tests/fixtures/{name}.json")).unwrap()
    });
    if name.ends_with(".json") || source.trim_start().starts_with('{') {
        parse(&source, InputFormat::Json).unwrap()
    } else {
        parse(&source, InputFormat::Markdown).unwrap()
    }
}

#[test]
fn every_corpus_document_exposes_the_named_slots() {
    for fixture in [
        "labeled-style",
        "plain-style",
        "mixed-style",
        "minimal",
        "front-matter",
        "legacy-directives",
    ] {
        let cv = corpus(fixture);
        let view = serde_json::to_value(cac_render::test_render_view(&cv)).unwrap();
        let profile = view["profile"].as_object().unwrap();
        for slot in ["name", "contacts", "summary", "photo", "headline", "address", "fields"] {
            assert!(profile.contains_key(slot), "{fixture}: profile has `{slot}`");
        }
        for section in view["sections"].as_array().unwrap() {
            for slot in ["id", "kind", "title", "entries", "tags"] {
                assert!(
                    section.as_object().unwrap().contains_key(slot),
                    "{fixture}: section has `{slot}`"
                );
            }
            for entry in section["entries"].as_array().unwrap() {
                for slot in [
                    "kind", "primary", "secondary", "period", "structured_period", "date",
                    "location", "url", "fields", "tags", "highlights",
                ] {
                    assert!(
                        entry.as_object().unwrap().contains_key(slot),
                        "{fixture}: entry has `{slot}`"
                    );
                }
            }
        }
    }
}

#[test]
fn structured_dates_use_canonical_strings() {
    let cv = corpus("labeled-style");
    let view = serde_json::to_value(cac_render::test_render_view(&cv)).unwrap();
    let entry = &view["sections"][0]["entries"][0];
    let period = entry["structured_period"].as_object().expect("structured period");
    assert_eq!(period["start"], "2023-01");
    assert_eq!(period["end"], "Present");
    assert_eq!(entry["period"], "Jan 2023 – Present", "display form beside structure");
}

#[test]
fn open_fields_reach_the_view() {
    let cv = corpus("minimal");
    let _ = &cv;
    let source = r#"{
        "profile": { "name": "Ada", "fields": { "gpa": "3.9" } },
        "sections": [{ "id": "s", "title": "S", "entries": [{
            "type": "custom", "heading": "Facts",
            "fields": { "furigana": "エイダ" }
        }]}]
    }"#;
    let document: cac_core::CvDocument = serde_json::from_str(source).unwrap();
    let view = serde_json::to_value(cac_render::test_render_view(&document)).unwrap();
    assert_eq!(view["profile"]["fields"]["gpa"], "3.9");
    assert_eq!(view["sections"][0]["entries"][0]["fields"]["furigana"], "エイダ");
}

#[test]
fn date_snapshots_in_five_locales() {
    use cac_render::{format_date_locale, Length};
    let _ = Length::new(10.0, cac_render::LengthUnit::Pt);
    let jan2023 = cac_core::DatePoint::YearMonth(2023, 1);
    assert_eq!(format_date_locale("en", &jan2023), "Jan 2023");
    assert_eq!(format_date_locale("de", &jan2023), "Januar 2023");
    assert_eq!(format_date_locale("fr", &jan2023), "janvier 2023");
    assert_eq!(format_date_locale("vi", &jan2023), "Tháng Một 2023");
    assert_eq!(format_date_locale("ja", &jan2023), "2023年1月");
}
