use std::fs;

use assert_cmd::Command;
use predicates::prelude::PredicateBooleanExt;
use tempfile::tempdir;

const CLEAN_MARKDOWN: &str =
    "# Ada Lovelace\n\nada@example.com\n\n## Skills\n\n- Rust and PostgreSQL\n";
const WARNING_MARKDOWN: &str = "# Ada Lovelace\n\nada@example.com\n\n## Experience\n\n### Engineer, Example\n2020–Present\n\n- Helped my team ship software\n";

#[test]
fn help_screens_match_snapshots() {
    let snapshots = [
        (&[][..], include_str!("snapshots/root-help.txt")),
        (&["init"][..], include_str!("snapshots/init-help.txt")),
        (&["build"][..], include_str!("snapshots/build-help.txt")),
        (&["check"][..], include_str!("snapshots/check-help.txt")),
        (&["convert"][..], include_str!("snapshots/convert-help.txt")),
        (&["themes"][..], include_str!("snapshots/themes-help.txt")),
    ];

    for (arguments, snapshot) in snapshots {
        let output = Command::cargo_bin("cac")
            .unwrap()
            .args(arguments)
            .arg("--help")
            .output()
            .unwrap();

        assert!(output.status.success());
        assert_eq!(String::from_utf8(output.stdout).unwrap(), snapshot);
        assert!(output.stderr.is_empty());
    }
}

#[test]
fn version_uses_lowercase_short_option() {
    let expected = format!("cac {}\n", env!("CARGO_PKG_VERSION"));

    for option in ["-v", "--version"] {
        Command::cargo_bin("cac")
            .unwrap()
            .arg(option)
            .assert()
            .success()
            .stdout(expected.clone())
            .stderr("");
    }

    Command::cargo_bin("cac")
        .unwrap()
        .arg("-V")
        .assert()
        .code(2)
        .stderr(predicates::str::contains("unexpected argument '-V'"));
}

#[test]
fn build_rejects_json_as_an_output_format_with_syntax_status() {
    Command::cargo_bin("cac")
        .unwrap()
        .args(["build", "--format", "json"])
        .assert()
        .code(2)
        .stderr(predicates::str::contains("invalid value 'json'"));
}

#[test]
fn json_command_output_option_is_rejected_in_both_global_positions() {
    for arguments in [["--json", "themes"], ["themes", "--json"]] {
        Command::cargo_bin("cac")
            .unwrap()
            .args(arguments)
            .assert()
            .code(2)
            .stderr(predicates::str::contains("--json"));
    }
}

#[test]
fn convert_still_exports_native_json() {
    let directory = tempdir().unwrap();
    let input = directory.path().join("cv.md");
    let output = directory.path().join("cv.json");
    fs::write(&input, CLEAN_MARKDOWN).unwrap();

    Command::cargo_bin("cac")
        .unwrap()
        .arg("convert")
        .arg(&input)
        .args(["--to", "json", "--output"])
        .arg(&output)
        .assert()
        .success();

    let exported: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(output).unwrap()).unwrap();
    assert_eq!(exported["profile"]["name"], "Ada Lovelace");
}

#[test]
fn init_creates_classic_settings_and_builds_html() {
    let directory = tempdir().unwrap();
    let input = directory.path().join("cv.md");
    let output = directory.path().join("dist");

    Command::cargo_bin("cac")
        .unwrap()
        .args(["init", "--output"])
        .arg(&input)
        .assert()
        .success();
    assert_eq!(
        fs::read_to_string(directory.path().join("settings.json")).unwrap(),
        "{\n  \"theme\": \"classic\"\n}\n"
    );
    Command::cargo_bin("cac")
        .unwrap()
        .arg("build")
        .arg(&input)
        .args(["--format", "html", "--output"])
        .arg(&output)
        .assert()
        .success();

    let html = fs::read_to_string(output.join("cv.html")).unwrap();
    assert!(html.contains("Ada Lovelace"));
}

#[test]
fn build_reads_markdown_from_stdin() {
    let directory = tempdir().unwrap();

    Command::cargo_bin("cac")
        .unwrap()
        .args(["build", "-", "--format", "html", "--output"])
        .arg(directory.path())
        .write_stdin(CLEAN_MARKDOWN)
        .assert()
        .success()
        .stdout(predicates::str::contains("BUILT"))
        .stderr("");

    assert!(directory.path().join("cv.html").is_file());
}

#[test]
fn check_reads_markdown_from_stdin() {
    Command::cargo_bin("cac")
        .unwrap()
        .args(["check", "-"])
        .write_stdin(CLEAN_MARKDOWN)
        .assert()
        .success()
        .stdout("PASS\n")
        .stderr("");
}

#[test]
fn convert_reads_structured_data_from_stdin() {
    Command::cargo_bin("cac")
        .unwrap()
        .args(["convert", "-", "--input-format", "json", "--to", "markdown"])
        .write_stdin(include_str!("../../../examples/cv.json"))
        .assert()
        .success()
        .stdout(predicates::str::starts_with("# Ada Lovelace\n"))
        .stderr("");
}

#[test]
fn init_writes_only_document_content_to_stdout() {
    let directory = tempdir().unwrap();

    Command::cargo_bin("cac")
        .unwrap()
        .current_dir(directory.path())
        .args(["init", "--output", "-"])
        .assert()
        .success()
        .stdout(predicates::str::starts_with("# Ada Lovelace\n"))
        .stdout(predicates::str::contains("CREATED").not())
        .stderr("");
    assert!(!directory.path().join("settings.json").exists());
}

#[test]
fn init_imports_json_resume_from_stdin() {
    Command::cargo_bin("cac")
        .unwrap()
        .args(["init", "--from", "-", "--output", "-"])
        .write_stdin(include_str!("../../../examples/resume.json"))
        .assert()
        .success()
        .stdout(predicates::str::starts_with("# Ada Lovelace\n"))
        .stdout(predicates::str::contains("CREATED").not())
        .stderr("");
}

#[test]
fn convert_writes_only_document_content_to_stdout() {
    let directory = tempdir().unwrap();
    let input = directory.path().join("cv.md");
    fs::write(&input, CLEAN_MARKDOWN).unwrap();

    for output_arguments in [&[][..], &["--output", "-"][..]] {
        Command::cargo_bin("cac")
            .unwrap()
            .arg("convert")
            .arg(&input)
            .args(["--to", "json"])
            .args(output_arguments)
            .assert()
            .success()
            .stdout(predicates::str::starts_with("{\n"))
            .stdout(predicates::str::contains("CREATED").not())
            .stderr("");
    }
}

#[test]
fn check_prints_a_final_result_and_uses_runtime_failure_status() {
    Command::cargo_bin("cac")
        .unwrap()
        .args(["check", "-"])
        .write_stdin(WARNING_MARKDOWN)
        .assert()
        .success()
        .stdout(predicates::str::contains("CAC301"))
        .stdout(predicates::str::ends_with("PASS\n"))
        .stderr("");

    Command::cargo_bin("cac")
        .unwrap()
        .args(["check", "-", "--strict"])
        .write_stdin(WARNING_MARKDOWN)
        .assert()
        .code(1)
        .stdout(predicates::str::ends_with("FAIL\n"))
        .stderr("error: CV checks failed\n");
}

#[test]
fn init_requires_force_to_replace_an_existing_file() {
    let directory = tempdir().unwrap();
    let output = directory.path().join("cv.md");
    fs::write(&output, "keep me").unwrap();

    Command::cargo_bin("cac")
        .unwrap()
        .args(["init", "--output"])
        .arg(&output)
        .assert()
        .code(1)
        .stdout("")
        .stderr(predicates::str::contains("use --force"));
    assert_eq!(fs::read_to_string(&output).unwrap(), "keep me");

    Command::cargo_bin("cac")
        .unwrap()
        .args(["init", "--output"])
        .arg(&output)
        .arg("--force")
        .assert()
        .success();
    assert_ne!(fs::read_to_string(output).unwrap(), "keep me");
    assert_eq!(
        fs::read_to_string(directory.path().join("settings.json")).unwrap(),
        "{\n  \"theme\": \"classic\"\n}\n"
    );
}

#[test]
fn init_requires_force_to_replace_existing_settings() {
    let directory = tempdir().unwrap();
    let output = directory.path().join("cv.md");
    let settings = directory.path().join("settings.json");
    fs::write(&settings, "keep me").unwrap();

    Command::cargo_bin("cac")
        .unwrap()
        .args(["init", "--output"])
        .arg(&output)
        .assert()
        .code(1)
        .stdout("")
        .stderr(predicates::str::contains("settings.json already exists"));
    assert!(!output.exists());
    assert_eq!(fs::read_to_string(&settings).unwrap(), "keep me");

    Command::cargo_bin("cac")
        .unwrap()
        .args(["init", "--output"])
        .arg(&output)
        .arg("--force")
        .assert()
        .success();
    assert!(output.is_file());
    assert_eq!(
        fs::read_to_string(settings).unwrap(),
        "{\n  \"theme\": \"classic\"\n}\n"
    );
}

#[test]
fn init_rejects_settings_as_the_cv_output() {
    let directory = tempdir().unwrap();

    Command::cargo_bin("cac")
        .unwrap()
        .current_dir(directory.path())
        .args(["init", "--output", "./settings.json"])
        .assert()
        .code(1)
        .stdout("")
        .stderr(predicates::str::contains("reserved for theme settings"));
    assert!(!directory.path().join("settings.json").exists());
}

#[test]
fn build_replaces_an_existing_artifact() {
    let directory = tempdir().unwrap();
    let input = directory.path().join("cv.md");
    let output = directory.path().join("dist");
    fs::create_dir(&output).unwrap();
    fs::write(&input, CLEAN_MARKDOWN).unwrap();
    fs::write(output.join("cv.html"), "old").unwrap();

    Command::cargo_bin("cac")
        .unwrap()
        .arg("build")
        .arg(&input)
        .args(["--format", "html", "--output"])
        .arg(&output)
        .assert()
        .success();

    assert_ne!(fs::read_to_string(output.join("cv.html")).unwrap(), "old");
}

#[test]
fn build_reads_settings_beside_the_cv_and_reports_the_theme() {
    let directory = tempdir().unwrap();
    let input = directory.path().join("cv.md");
    let output = directory.path().join("dist");
    let theme = directory.path().join(".cac/themes/compact");
    fs::create_dir_all(&theme).unwrap();
    fs::write(&input, CLEAN_MARKDOWN).unwrap();
    fs::write(
        directory.path().join("settings.json"),
        r#"{"theme":"compact","page_margin":"12mm"}"#,
    )
    .unwrap();
    fs::write(
        theme.join("theme.typ"),
        r#"
#import "/.cac/base.typ" as base
#let theme = base.extend(page: (margin: 20mm))
"#,
    )
    .unwrap();

    Command::cargo_bin("cac")
        .unwrap()
        .arg("build")
        .arg(&input)
        .args(["--output"])
        .arg(&output)
        .assert()
        .success()
        .stdout(predicates::str::contains("THEME compact (project)"))
        .stdout(predicates::str::contains("BUILT"));

    assert!(output.join("cv.pdf").is_file());
}

#[test]
fn themes_install_list_and_remove_locally() {
    let directory = tempdir().unwrap();

    Command::cargo_bin("cac")
        .unwrap()
        .current_dir(directory.path())
        .args(["themes", "install", "classic", "--local"])
        .assert()
        .success()
        .stdout("INSTALLED classic (project)\n");
    Command::cargo_bin("cac")
        .unwrap()
        .current_dir(directory.path())
        .args(["themes", "list"])
        .assert()
        .success()
        .stdout(predicates::str::contains("classic (embedded, project)"))
        .stdout(predicates::str::contains("classic-left (embedded)"));
    Command::cargo_bin("cac")
        .unwrap()
        .current_dir(directory.path())
        .args(["themes", "remove", "classic", "--local"])
        .assert()
        .success()
        .stdout("REMOVED classic (project)\n");

    assert!(!directory.path().join(".cac/themes/classic").exists());

    Command::cargo_bin("cac")
        .unwrap()
        .current_dir(directory.path())
        .args(["themes", "install", "classic-left", "--local"])
        .assert()
        .success()
        .stdout("INSTALLED classic-left (project)\n");
    assert!(
        directory
            .path()
            .join(".cac/themes/classic-left/theme.typ")
            .is_file()
    );
}

#[test]
fn convert_replaces_an_existing_output_file() {
    let directory = tempdir().unwrap();
    let input = directory.path().join("cv.md");
    let output = directory.path().join("cv.json");
    fs::write(&input, CLEAN_MARKDOWN).unwrap();
    fs::write(&output, "old").unwrap();

    Command::cargo_bin("cac")
        .unwrap()
        .arg("convert")
        .arg(&input)
        .args(["--to", "json", "--output"])
        .arg(&output)
        .assert()
        .success();

    assert_ne!(fs::read_to_string(output).unwrap(), "old");
}
