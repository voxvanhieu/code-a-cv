use std::fs;

use assert_cmd::Command;
use tempfile::tempdir;

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
