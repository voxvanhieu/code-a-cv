//! The grammar corpus (C4): `input.md` + `input.md.expected.json` pairs
//! recording what the parser produces, so every grammar change is visible
//! as a baseline diff.
//!
//! The baselines were recorded from the 0.3.0 parser, heuristics included:
//! the whitespace-sensitivity and mode-flag behavior they capture is what
//! the AST-based parser (P1) changes deliberately. Regenerate only with a
//! reviewed grammar change:
//!
//! ```sh
//! CAC_GENERATE_FIXTURE_BASELINE=1 cargo test -p cac-io --test markdown_baseline
//! ```

use std::fs;
use std::path::Path;

const DIR: &str = "tests/fixtures/markdown";

fn fixtures() -> Vec<(String, String)> {
    let mut names: Vec<String> = fs::read_dir(DIR)
        .expect("fixture directory exists")
        .map(|entry| entry.expect("readable entry").path())
        .filter(|path| path.extension().is_some_and(|extension| extension == "md"))
        .map(|path| path.file_name().unwrap().to_string_lossy().to_string())
        .collect();
    names.sort();
    names
        .into_iter()
        .map(|name| {
            let source = fs::read_to_string(Path::new(DIR).join(&name)).expect("readable fixture");
            (name, source)
        })
        .collect()
}

fn baseline_document(source: &str) -> String {
    match cac_io::parse_markdown(source) {
        Ok(cv) => serde_json::to_string_pretty(&cv).expect("the model serializes"),
        Err(error) => {
            let message = error.to_string();
            format!(
                "{{\n  \"parse_error\": {}\n}}",
                serde_json::to_string(&message).expect("the message serializes")
            )
        }
    }
}

#[test]
fn fixtures_match_the_recorded_baseline() {
    let generate = std::env::var("CAC_GENERATE_FIXTURE_BASELINE").is_ok();
    let mut compared = 0;
    for (name, source) in fixtures() {
        let actual = baseline_document(&source);
        let expected_path = Path::new(DIR).join(format!("{name}.expected.json"));
        if generate {
            fs::write(&expected_path, format!("{actual}\n")).expect("writable baseline");
            println!("recorded {}", expected_path.display());
            compared += 1;
            continue;
        }
        let expected = fs::read_to_string(&expected_path).unwrap_or_else(|error| {
            panic!(
                "missing baseline {} ({error}); record it with CAC_GENERATE_FIXTURE_BASELINE=1",
                expected_path.display()
            )
        });
        assert_eq!(
            actual,
            expected.trim_end(),
            "baseline drifted for {name}; if the grammar changed deliberately, regenerate the \
             baseline and review the diff"
        );
        compared += 1;
    }
    assert!(
        compared >= 14,
        "the corpus must keep at least the 14 recorded fixtures, found {compared}"
    );
}
