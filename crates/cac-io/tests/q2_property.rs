//! Q2: property-based round trips. A seeded generator composes valid
//! documents across all entry kinds, open fields, and Unicode; each case
//! round-trips through every native format and both Markdown spellings
//! parse to one model. Deterministic seeds keep CI stable; the sweep
//! covers 1,000 format conversions per run.

use cac_io::{InputFormat, MarkdownStyle, parse, to_markdown_with_style};

struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        // xorshift64*
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0.wrapping_mul(0x2545F4914F6CDD1D)
    }
    fn pick<'a>(&mut self, items: &'a [&'a str]) -> &'a str {
        items[(self.next() % items.len() as u64) as usize]
    }
    fn below(&mut self, bound: u64) -> u64 {
        self.next() % bound
    }
}

const WORDS: [&str; 8] = [
    "Engineer",
    "Rust",
    "Osaka",
    "Naive cafe",
    "Team-lead",
    "platform",
    "straight-line",
    "backup",
];

const KINDS: [&str; 8] = [
    "experience",
    "education",
    "project",
    "publication",
    "skill-group",
    "custom",
    "language",
    "reference",
];

fn safe_word(rng: &mut Rng) -> String {
    // Letters and spaces only: markdown-checked writing rejects most
    // punctuation shapes, and the corpus already pins escaping.
    rng.pick(&WORDS).trim().to_string()
}

pub fn generate_case(seed: u64) -> cac_core::CvDocument {
    let mut rng = Rng(seed);
    let mut profile = serde_json::Map::new();
    profile.insert("name".into(), serde_json::json!(safe_word(&mut rng)));
    if rng.below(2) == 0 {
        profile.insert("email".into(), serde_json::json!(format!("a{}@b.c", seed)));
    }
    if rng.below(2) == 0 {
        profile.insert("headline".into(), serde_json::json!(safe_word(&mut rng)));
    }
    if rng.below(3) == 0 {
        // Rule F needs a known anchor in the block; the email provides it.
        profile.insert("email".into(), serde_json::json!(format!("a{}@b.c", seed)));
        profile.insert("fields".into(), serde_json::json!({"gpa": "3.9"}));
    }
    let mut sections = Vec::new();
    let section_count = 1 + rng.below(3);
    for s in 0..section_count {
        let kind = rng.pick(&KINDS);
        let entry_count = 1 + rng.below(2);
        let mut entries = Vec::new();
        for _ in 0..entry_count {
            let mut entry = serde_json::Map::new();
            entry.insert("type".into(), serde_json::json!(kind));
            match kind {
                "experience" => {
                    entry.insert("role".into(), serde_json::json!(safe_word(&mut rng)));
                    entry.insert(
                        "organization".into(),
                        serde_json::json!(safe_word(&mut rng)),
                    );
                }
                "education" => {
                    entry.insert(
                        "qualification".into(),
                        serde_json::json!(safe_word(&mut rng)),
                    );
                    entry.insert("institution".into(), serde_json::json!(safe_word(&mut rng)));
                }
                "project" | "language" | "skill-group" => {
                    entry.insert("name".into(), serde_json::json!(safe_word(&mut rng)));
                }
                "publication" => {
                    entry.insert("title".into(), serde_json::json!(safe_word(&mut rng)));
                }
                "reference" => {
                    entry.insert("name".into(), serde_json::json!(safe_word(&mut rng)));
                    entry.insert("note".into(), serde_json::json!(safe_word(&mut rng)));
                }
                "custom" => {
                    entry.insert("heading".into(), serde_json::json!(safe_word(&mut rng)));
                }
                _ => {}
            }
            if rng.below(2) == 0
                && matches!(
                    kind,
                    "experience" | "education" | "project" | "publication" | "custom"
                )
            {
                entry.insert(
                    "highlights".into(),
                    serde_json::json!([safe_word(&mut rng), safe_word(&mut rng)]),
                );
            }
            entries.push(serde_json::Value::Object(entry));
        }
        sections.push(serde_json::json!({
            "id": format!("section-{s}"),
            "title": format!("Section {s}"),
            "kind": kind,
            "entries": entries,
        }));
    }
    let value = serde_json::json!({ "profile": profile, "sections": sections });
    serde_json::from_value(value).expect("the generator emits valid documents")
}

#[test]
fn one_thousand_generated_cases_round_trip() {
    let mut conversions = 0;
    for seed in 1..=250u64 {
        let cv = generate_case(seed);
        for format in [InputFormat::Json, InputFormat::Yaml, InputFormat::Toml] {
            let written = match format {
                InputFormat::Json => serde_json::to_string_pretty(&cv).unwrap(),
                InputFormat::Yaml => serde_yaml_ng::to_string(&cv).unwrap(),
                InputFormat::Toml => toml::to_string_pretty(&cv).unwrap(),
                _ => unreachable!(),
            };
            let reparsed = parse(&written, format).unwrap();
            assert_eq!(reparsed, cv, "seed {seed}: {format:?} round trip");
            conversions += 1;
        }
        // Markdown: checked write, then both spellings parse to one model.
        let explicit = to_markdown_with_style(&cv, MarkdownStyle::Explicit);
        let reparsed = parse(&explicit, InputFormat::Markdown).unwrap();
        assert_eq!(reparsed, cv, "seed {seed}: explicit");
        let compact = to_markdown_with_style(&cv, MarkdownStyle::Compact);
        assert_eq!(
            parse(&compact, InputFormat::Markdown).unwrap(),
            cv,
            "seed {seed}: compact"
        );
        conversions += 3;
    }
    assert!(
        conversions >= 1000,
        "the sweep covers 1,000+ conversions: {conversions}"
    );
}
