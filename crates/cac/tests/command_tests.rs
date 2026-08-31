use std::fs;

use assert_cmd::Command;
use tempfile::tempdir;

#[test]
fn help_uses_cac_as_the_only_program_name() {
    let output = Command::cargo_bin("cac")
        .unwrap()
        .arg("--help")
        .output()
        .unwrap();

    assert!(output.status.success());
    let help = String::from_utf8(output.stdout).unwrap();
    assert!(help.contains("Usage: cac"));
    assert!(!help.to_ascii_lowercase().contains("cli"));
    assert!(!help.contains("--json"));
}

#[test]
fn build_does_not_accept_json_as_an_output_format() {
    Command::cargo_bin("cac")
        .unwrap()
        .args(["build", "--format", "json"])
        .assert()
        .failure();
}

#[test]
fn global_json_output_is_not_accepted() {
    Command::cargo_bin("cac")
        .unwrap()
        .args(["--json", "themes"])
        .assert()
        .failure();
}

#[test]
fn convert_still_exports_native_json() {
    let directory = tempdir().unwrap();
    let input = directory.path().join("cv.md");
    let output = directory.path().join("cv.json");

    Command::cargo_bin("cac")
        .unwrap()
        .args(["init", "--output"])
        .arg(&input)
        .assert()
        .success();
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
fn init_and_build_html_work_without_configuration() {
    let directory = tempdir().unwrap();
    let input = directory.path().join("cv.md");
    let output = directory.path().join("dist");

    Command::cargo_bin("cac")
        .unwrap()
        .args(["init", "--output"])
        .arg(&input)
        .assert()
        .success();
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
fn init_does_not_replace_an_existing_file() {
    let directory = tempdir().unwrap();
    let input = directory.path().join("cv.md");
    fs::write(&input, "keep me").unwrap();

    Command::cargo_bin("cac")
        .unwrap()
        .args(["init", "--output"])
        .arg(&input)
        .assert()
        .failure();
    assert_eq!(fs::read_to_string(input).unwrap(), "keep me");
}
