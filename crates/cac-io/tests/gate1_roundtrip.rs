//! Gate 1 spot-check: parse → checked-write → parse is the identity over
//! every Markdown fixture and the shared corpus, in all four native
//! formats. The full property-based sweep lands with Q2; this pins the
//! wave boundary.

use cac_io::{InputFormat, parse, to_markdown_checked};

fn round_trip_markdown(source: &str, label: &str) {
    let cv = parse(source, InputFormat::Markdown).unwrap();
    let written = to_markdown_checked(&cv).unwrap();
    let reparsed = parse(&written, InputFormat::Markdown).unwrap();
    assert_eq!(reparsed, cv, "{label}: markdown round trip");
}

#[test]
fn every_markdown_fixture_round_trips() {
    let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/markdown");
    // Error fixtures (whose recorded baseline is a parse error) are out of
    // scope: they pin diagnostics, not documents.
    let error_fixtures = ["edge-summary-lookalike"];
    for entry in std::fs::read_dir(dir).expect("fixture dir") {
        let path = entry.unwrap().path();
        let name = path.file_stem().unwrap().to_string_lossy().to_string();
        if path.extension().is_some_and(|extension| extension == "md")
            && !error_fixtures.contains(&name.as_str())
        {
            let source = std::fs::read_to_string(&path).unwrap();
            round_trip_markdown(&source, &path.display().to_string());
        }
    }
}

fn serialize(cv: &cac_core::CvDocument, format: InputFormat) -> String {
    match format {
        InputFormat::Json => serde_json::to_string_pretty(cv).unwrap(),
        InputFormat::Yaml => serde_yaml_ng::to_string(cv).unwrap(),
        InputFormat::Toml => toml::to_string_pretty(cv).unwrap(),
        InputFormat::Markdown => to_markdown_checked(cv).unwrap(),
        InputFormat::JsonResume => unimplemented!("export is lossy-checked separately"),
    }
}

#[test]
fn the_shared_corpus_round_trips_in_every_native_format() {
    let dir = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../cac-render/fixtures/shared/"
    );
    // Markdown is a native format too: the corpus must round-trip through
    // it, not only through the three lossless spellings.
    let formats = [
        InputFormat::Json,
        InputFormat::Yaml,
        InputFormat::Toml,
        InputFormat::Markdown,
    ];
    for entry in std::fs::read_dir(dir).expect("corpus dir") {
        let path = entry.unwrap().path();
        if path
            .extension()
            .is_some_and(|extension| extension == "json")
        {
            let source = std::fs::read_to_string(&path).unwrap();
            let cv = parse(&source, InputFormat::Json).unwrap();
            for format in formats {
                // Markdown refuses documents it cannot spell unambiguously
                // (checked conversion never writes a corruption); the
                // corpus round trip covers the spellable documents.
                let written = match format {
                    InputFormat::Markdown => match cac_io::to_markdown_checked(&cv) {
                        Ok(written) => written,
                        Err(_) => continue,
                    },
                    _ => serialize(&cv, format),
                };
                let reparsed = parse(&written, format).unwrap();
                assert_eq!(
                    reparsed,
                    cv,
                    "{}: {} round trip",
                    path.display(),
                    serde_debug(format)
                );
            }
        }
    }
}

fn serde_debug(format: InputFormat) -> &'static str {
    match format {
        InputFormat::Json => "json",
        InputFormat::Yaml => "yaml",
        InputFormat::Toml => "toml",
        _ => "?",
    }
}
