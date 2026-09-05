use std::fs;
use std::path::PathBuf;

use assert_cmd::Command;
use predicates::prelude::PredicateBooleanExt;
use tempfile::tempdir;

const CLEAN_MARKDOWN: &str =
    "# Ada Lovelace\n\nada@example.com\n\n## Skills\n\n- Rust and PostgreSQL\n";
const CLASSIC_SETTINGS: &str = "{\n  \"$schema\": \".cac/settings.schema.json\",\n  \"root\": \"cv.md\",\n  \"theme\": \"classic\"\n}\n";
const WARNING_MARKDOWN: &str = "# Ada Lovelace\n\nada@example.com\n\n## Experience\n\n### Engineer, Example\n2020–Present\n\n- Helped my team ship software\n";

#[test]
fn help_screens_match_snapshots() {
    let snapshots = [
        (&[][..], include_str!("snapshots/root-help.txt")),
        (&["init"][..], include_str!("snapshots/init-help.txt")),
        (&["build"][..], include_str!("snapshots/build-help.txt")),
        (&["check"][..], include_str!("snapshots/check-help.txt")),
        (&["convert"][..], include_str!("snapshots/convert-help.txt")),
        (&["schema"][..], include_str!("snapshots/schema-help.txt")),
        (&["theme"][..], include_str!("snapshots/theme-help.txt")),
        (
            &["theme", "init"][..],
            include_str!("snapshots/theme-init-help.txt"),
        ),
        (
            &["theme", "test"][..],
            include_str!("snapshots/theme-test-help.txt"),
        ),
        (
            &["theme", "pack"][..],
            include_str!("snapshots/theme-pack-help.txt"),
        ),
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
fn theme_init_creates_a_development_project_from_arguments() {
    let directory = tempdir().unwrap();
    Command::cargo_bin("cac")
        .unwrap()
        .current_dir(directory.path())
        .args([
            "theme",
            "init",
            "portfolio",
            "--author",
            "Ada Lovelace",
            "--author-url",
            "https://example.com/ada",
        ])
        .assert()
        .success();

    let settings: serde_json::Value =
        serde_json::from_slice(&fs::read(directory.path().join("settings.json")).unwrap()).unwrap();
    assert_eq!(settings["theme"], "portfolio");
    assert_eq!(settings["themeProject"], "portfolio");
    assert!(
        directory
            .path()
            .join(".cac/themes/portfolio/theme.typ")
            .is_file()
    );
    let schema: serde_json::Value = serde_json::from_slice(
        &fs::read(directory.path().join(".cac/settings.schema.json")).unwrap(),
    )
    .unwrap();
    assert!(schema["properties"]["themeProject"].is_object());
}

#[test]
fn theme_init_prompts_and_retries_without_leaving_partial_files() {
    let directory = tempdir().unwrap();
    Command::cargo_bin("cac")
        .unwrap()
        .current_dir(directory.path())
        .args(["theme", "init"])
        .write_stdin("classic\nmy-theme\n\nGrace Hopper\nmailto:grace@example.com\n\n")
        .assert()
        .success()
        .stderr(
            predicates::str::contains("Invalid Theme name")
                .and(predicates::str::contains("Invalid Author")),
        );
    let manifest: serde_json::Value = serde_json::from_slice(
        &fs::read(directory.path().join(".cac/themes/my-theme/theme.json")).unwrap(),
    )
    .unwrap();
    assert!(manifest.get("author_url").is_none());

    let interrupted = tempdir().unwrap();
    Command::cargo_bin("cac")
        .unwrap()
        .current_dir(interrupted.path())
        .args(["theme", "init"])
        .write_stdin("unfinished\n")
        .assert()
        .failure();
    assert!(!interrupted.path().join("cv.md").exists());
}

#[test]
fn theme_test_and_pack_generate_verified_reproducible_artifacts() {
    let directory = tempdir().unwrap();
    Command::cargo_bin("cac")
        .unwrap()
        .current_dir(directory.path())
        .args([
            "theme",
            "init",
            "portfolio",
            "--author",
            "Ada Lovelace",
            "--author-url",
            "https://example.com/ada",
        ])
        .assert()
        .success();
    let manifest_path = directory.path().join(".cac/themes/portfolio/theme.json");
    let mut manifest: serde_json::Value =
        serde_json::from_slice(&fs::read(&manifest_path).unwrap()).unwrap();
    manifest["description"] = "A representative theme".into();
    fs::write(
        &manifest_path,
        serde_json::to_vec_pretty(&manifest).unwrap(),
    )
    .unwrap();
    fs::create_dir_all(directory.path().join(".cac/themes/portfolio/assets/nested")).unwrap();
    fs::write(
        directory
            .path()
            .join(".cac/themes/portfolio/assets/nested/icon.txt"),
        "asset",
    )
    .unwrap();

    Command::cargo_bin("cac")
        .unwrap()
        .current_dir(directory.path())
        .args(["theme", "test"])
        .assert()
        .success();
    assert!(directory.path().join("offering/portfolio.pdf").is_file());
    assert!(
        directory
            .path()
            .join(".cac/themes/portfolio/preview.jpg")
            .is_file()
    );
    let manifest: serde_json::Value =
        serde_json::from_slice(&fs::read(&manifest_path).unwrap()).unwrap();
    assert!(
        manifest["files"]
            .as_array()
            .unwrap()
            .iter()
            .any(|file| file["path"] == "assets/nested/icon.txt")
    );

    Command::cargo_bin("cac")
        .unwrap()
        .current_dir(directory.path())
        .args(["theme", "pack"])
        .assert()
        .success();
    let first = fs::read(directory.path().join("portfolio.zip")).unwrap();
    Command::cargo_bin("cac")
        .unwrap()
        .current_dir(directory.path())
        .args(["theme", "pack"])
        .assert()
        .success();
    assert_eq!(
        first,
        fs::read(directory.path().join("portfolio.zip")).unwrap()
    );
    let reader = std::io::Cursor::new(first);
    let archive = zip::ZipArchive::new(reader).unwrap();
    let names = archive.file_names().map(str::to_owned).collect::<Vec<_>>();
    assert!(names.contains(&"portfolio/theme.json".into()));
    assert!(names.contains(&"portfolio/preview.jpg".into()));
    assert!(!names.iter().any(|name| name.ends_with(".pdf")));
}

#[test]
fn theme_project_restricts_install_and_selected_removal() {
    let directory = tempdir().unwrap();
    fs::write(
        directory.path().join("settings.json"),
        r#"{"theme":"developing","themeProject":"developing"}"#,
    )
    .unwrap();
    fs::create_dir_all(directory.path().join(".cac/themes/developing")).unwrap();
    fs::write(
        directory.path().join(".cac/themes/developing/theme.typ"),
        "",
    )
    .unwrap();
    Command::cargo_bin("cac")
        .unwrap()
        .current_dir(directory.path())
        .args(["theme", "install", "anything", "--local"])
        .env("CAC_THEME_REGISTRY", "file:///definitely-unavailable")
        .assert()
        .failure()
        .stderr(predicates::str::contains(
            "cannot be installed inside a theme project",
        ));
    Command::cargo_bin("cac")
        .unwrap()
        .current_dir(directory.path())
        .args(["theme", "remove", "developing", "--local"])
        .assert()
        .failure()
        .stderr(predicates::str::contains("cannot be removed"));
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
fn schema_creates_updates_and_validates_project_settings() {
    let directory = tempdir().unwrap();
    let schema = directory.path().join(".cac/settings.schema.json");
    fs::write(
        directory.path().join("settings.json"),
        r#"{"$schema":".cac/settings.schema.json","root":"cv.md","theme":"classic"}"#,
    )
    .unwrap();

    Command::cargo_bin("cac")
        .unwrap()
        .current_dir(directory.path())
        .arg("schema")
        .assert()
        .success()
        .stdout("CREATED .cac/settings.schema.json\nVALID settings.json\n");
    let generated: serde_json::Value = serde_json::from_slice(&fs::read(&schema).unwrap()).unwrap();
    assert_eq!(generated["type"], "object");
    assert_eq!(generated["properties"]["page"]["$ref"], "#/$defs/page");
    assert_eq!(
        generated["properties"]["typography"]["$ref"],
        "#/$defs/typography"
    );

    fs::write(&schema, "stale").unwrap();
    Command::cargo_bin("cac")
        .unwrap()
        .current_dir(directory.path())
        .arg("schema")
        .assert()
        .success()
        .stdout("UPDATED .cac/settings.schema.json\nVALID settings.json\n");

    Command::cargo_bin("cac")
        .unwrap()
        .current_dir(directory.path())
        .arg("schema")
        .assert()
        .success()
        .stdout("VALID settings.json\n");

    fs::write(
        directory.path().join("settings.json"),
        r#"{"page":{"paper":"a3"}}"#,
    )
    .unwrap();
    Command::cargo_bin("cac")
        .unwrap()
        .current_dir(directory.path())
        .arg("schema")
        .assert()
        .code(1)
        .stdout("")
        .stderr(predicates::str::contains("expected `a4` or `us-letter`"));
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
fn init_creates_root_and_classic_settings_and_builds_html() {
    let directory = tempdir().unwrap();
    let input = directory.path().join("profile.md");
    let output = directory.path().join("offering");

    Command::cargo_bin("cac")
        .unwrap()
        .args(["init", "--output"])
        .arg(&input)
        .assert()
        .success();
    assert_eq!(
        fs::read_to_string(directory.path().join("settings.json")).unwrap(),
        "{\n  \"$schema\": \".cac/settings.schema.json\",\n  \"root\": \"profile.md\",\n  \"theme\": \"classic\"\n}\n"
    );
    assert!(directory.path().join(".cac/settings.schema.json").is_file());
    Command::cargo_bin("cac")
        .unwrap()
        .arg("build")
        .arg(&input)
        .args(["--format", "html", "--output"])
        .arg(&output)
        .assert()
        .success();

    let html = fs::read_to_string(output.join("profile.html")).unwrap();
    assert!(html.contains("Ada Lovelace"));
}

#[test]
fn init_uses_the_format_default_output() {
    let directory = tempdir().unwrap();

    Command::cargo_bin("cac")
        .unwrap()
        .current_dir(directory.path())
        .args(["init", "--format", "json"])
        .assert()
        .success()
        .stdout(predicates::str::contains("CREATED cv.json"));

    let cv: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(directory.path().join("cv.json")).unwrap())
            .unwrap();
    assert_eq!(cv["profile"]["name"], "Ada Lovelace");
    assert_eq!(
        fs::read_to_string(directory.path().join("settings.json")).unwrap(),
        "{\n  \"$schema\": \".cac/settings.schema.json\",\n  \"root\": \"cv.json\",\n  \"theme\": \"classic\"\n}\n"
    );
}

#[test]
fn init_uses_the_custom_output_as_the_settings_root() {
    let directory = tempdir().unwrap();

    Command::cargo_bin("cac")
        .unwrap()
        .current_dir(directory.path())
        .args(["init", "--format", "yaml", "--output", "new-cv.yaml"])
        .assert()
        .success();

    let cv: serde_yaml_ng::Value =
        serde_yaml_ng::from_str(&fs::read_to_string(directory.path().join("new-cv.yaml")).unwrap())
            .unwrap();
    assert_eq!(cv["profile"]["name"], "Ada Lovelace");
    assert_eq!(
        fs::read_to_string(directory.path().join("settings.json")).unwrap(),
        "{\n  \"$schema\": \".cac/settings.schema.json\",\n  \"root\": \"new-cv.yaml\",\n  \"theme\": \"classic\"\n}\n"
    );
}

#[test]
fn init_rejects_an_output_extension_that_does_not_match_the_format() {
    let directory = tempdir().unwrap();

    Command::cargo_bin("cac")
        .unwrap()
        .current_dir(directory.path())
        .args(["init", "--format", "json", "--output", "cv.md"])
        .assert()
        .code(1)
        .stdout("")
        .stderr(predicates::str::contains(
            "does not have an extension compatible with the `json` format",
        ));
    assert!(!directory.path().join("cv.md").exists());
    assert!(!directory.path().join("settings.json").exists());
}

#[test]
fn build_uses_the_settings_root_when_input_is_omitted() {
    let directory = tempdir().unwrap();
    fs::write(
        directory.path().join("primary.json"),
        include_str!("../../../docs/examples/structured-formats/cv.json"),
    )
    .unwrap();
    fs::write(
        directory.path().join("settings.json"),
        r#"{"root":"primary.json","theme":"classic"}"#,
    )
    .unwrap();

    Command::cargo_bin("cac")
        .unwrap()
        .current_dir(directory.path())
        .args(["build", "--format", "html"])
        .assert()
        .success()
        .stdout(predicates::str::contains("BUILT offering/primary.html"));
    assert!(directory.path().join("offering/primary.html").is_file());
}

#[test]
fn build_uses_the_configured_artifact_name() {
    let directory = tempdir().unwrap();
    fs::write(directory.path().join("resume.md"), CLEAN_MARKDOWN).unwrap();
    fs::write(
        directory.path().join("settings.json"),
        r#"{"root":"resume.md","naming":"Ada_Lovelace_Engineering","theme":"classic"}"#,
    )
    .unwrap();

    Command::cargo_bin("cac")
        .unwrap()
        .current_dir(directory.path())
        .args(["build", "--format", "html"])
        .assert()
        .success()
        .stdout(predicates::str::contains(
            "BUILT offering/Ada_Lovelace_Engineering.html",
        ));
    assert!(
        directory
            .path()
            .join("offering/Ada_Lovelace_Engineering.html")
            .is_file()
    );
}

#[test]
fn build_resolves_root_relative_to_an_explicit_settings_file() {
    let directory = tempdir().unwrap();
    let project = directory.path().join("project");
    fs::create_dir(&project).unwrap();
    fs::write(
        project.join("primary.json"),
        include_str!("../../../docs/examples/structured-formats/cv.json"),
    )
    .unwrap();
    fs::write(
        project.join("settings.json"),
        r#"{"root":"primary.json","theme":"classic"}"#,
    )
    .unwrap();

    Command::cargo_bin("cac")
        .unwrap()
        .current_dir(directory.path())
        .args([
            "build",
            "--settings",
            "project/settings.json",
            "--format",
            "html",
        ])
        .assert()
        .success()
        .stdout(predicates::str::contains("BUILT offering/primary.html"));
}

#[test]
fn build_without_settings_falls_back_to_cv_markdown() {
    let directory = tempdir().unwrap();
    fs::write(directory.path().join("cv.md"), CLEAN_MARKDOWN).unwrap();

    Command::cargo_bin("cac")
        .unwrap()
        .current_dir(directory.path())
        .args(["build", "--format", "html"])
        .assert()
        .success()
        .stdout(predicates::str::contains("BUILT offering/cv.html"));
    let generated: serde_json::Value = serde_json::from_slice(
        &fs::read(directory.path().join(".cac/settings.schema.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(generated["type"], "object");
    assert_eq!(generated["additionalProperties"], false);
}

#[test]
fn explicit_build_input_takes_precedence_over_the_settings_root() {
    let directory = tempdir().unwrap();
    fs::write(directory.path().join("selected.md"), CLEAN_MARKDOWN).unwrap();
    fs::write(
        directory.path().join("settings.json"),
        r#"{"root":"missing.json","theme":"classic"}"#,
    )
    .unwrap();

    Command::cargo_bin("cac")
        .unwrap()
        .current_dir(directory.path())
        .args(["build", "selected.md", "--format", "html"])
        .assert()
        .success()
        .stdout(predicates::str::contains("BUILT offering/selected.html"));
}

#[test]
fn build_reads_markdown_from_stdin() {
    let directory = tempdir().unwrap();

    Command::cargo_bin("cac")
        .unwrap()
        .current_dir(directory.path())
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
        .write_stdin(include_str!(
            "../../../docs/examples/structured-formats/cv.json"
        ))
        .assert()
        .success()
        .stdout(predicates::str::starts_with("# Ada Lovelace\n"))
        .stderr("");
}

#[test]
fn init_rejects_standard_output() {
    let directory = tempdir().unwrap();

    Command::cargo_bin("cac")
        .unwrap()
        .current_dir(directory.path())
        .args(["init", "--output", "-"])
        .assert()
        .failure()
        .stdout("")
        .stderr(predicates::str::contains(
            "cac init requires a file output; standard output is not supported",
        ));
    assert!(!directory.path().join("settings.json").exists());
}

#[test]
fn init_imports_json_resume_from_stdin() {
    let directory = tempdir().unwrap();
    let output = directory.path().join("cv.md");

    Command::cargo_bin("cac")
        .unwrap()
        .args(["init", "--from", "-", "--output"])
        .arg(&output)
        .write_stdin(include_str!(
            "../../../docs/examples/json-resume-import/resume.json"
        ))
        .assert()
        .success();
    assert!(
        fs::read_to_string(output)
            .unwrap()
            .starts_with("# Ada Lovelace\n")
    );
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
        CLASSIC_SETTINGS
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
    assert_eq!(fs::read_to_string(settings).unwrap(), CLASSIC_SETTINGS);
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
    let output = directory.path().join("offering");
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
    let output = directory.path().join("offering");
    let theme = directory.path().join(".cac/themes/compact");
    fs::create_dir_all(&theme).unwrap();
    fs::write(&input, CLEAN_MARKDOWN).unwrap();
    fs::write(
        directory.path().join("settings.json"),
        r#"{"theme":"compact","page":{"margin":"12mm"}}"#,
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
fn themes_install_rejects_system_theme_names() {
    let directory = tempdir().unwrap();

    for theme in ["classic", "base", "main"] {
        Command::cargo_bin("cac")
            .unwrap()
            .current_dir(directory.path())
            .args(["theme", "install", theme, "--local"])
            .assert()
            .code(1)
            .stdout("")
            .stderr(format!(
                "error: theme name `{theme}` is reserved by cac and cannot be installed\n"
            ));
        assert!(!directory.path().join(".cac/themes").join(theme).exists());
    }

    Command::cargo_bin("cac")
        .unwrap()
        .current_dir(directory.path())
        .args(["theme", "list"])
        .assert()
        .success()
        .stdout(predicates::str::contains("classic (embedded)"))
        .stdout(predicates::str::contains("classic-left").not());
}

#[test]
fn themes_list_and_remove_a_local_theme() {
    let directory = tempdir().unwrap();
    let theme = directory.path().join(".cac/themes/custom");
    fs::create_dir_all(&theme).unwrap();
    fs::write(theme.join("theme.typ"), "#let theme = (:)").unwrap();

    Command::cargo_bin("cac")
        .unwrap()
        .current_dir(directory.path())
        .args(["theme", "list"])
        .assert()
        .success()
        .stdout(predicates::str::contains("custom (project)"));
    Command::cargo_bin("cac")
        .unwrap()
        .current_dir(directory.path())
        .args(["theme", "remove", "custom", "--local"])
        .assert()
        .success()
        .stdout("REMOVED custom (project)\n");

    assert!(!theme.exists());
}

#[test]
fn themes_search_info_install_and_build_from_a_registry() {
    let directory = tempdir().unwrap();
    let registry = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("themes")
        .canonicalize()
        .unwrap();
    let registry = format!("file://{}", registry.display());

    Command::cargo_bin("cac")
        .unwrap()
        .env("CAC_THEME_REGISTRY", &registry)
        .args(["theme", "search", "blue"])
        .assert()
        .success()
        .stdout("classic-blue: The classic centered CV with blue headings\n");

    Command::cargo_bin("cac")
        .unwrap()
        .env("CAC_THEME_REGISTRY", &registry)
        .args(["theme", "info", "classic-blue"])
        .assert()
        .success()
        .stdout(predicates::str::contains("NAME classic-blue\n"))
        .stdout(predicates::str::contains(
            "AUTHOR URL https://github.com/voxvanhieu/code-a-cv/graphs/contributors\n",
        ))
        .stdout(predicates::str::contains("THEME API 1\n"));

    Command::cargo_bin("cac")
        .unwrap()
        .current_dir(directory.path())
        .env("CAC_THEME_REGISTRY", &registry)
        .args(["theme", "install", "classic-blue", "--local"])
        .assert()
        .success()
        .stdout("INSTALLED classic-blue (project)\n");
    assert!(
        directory
            .path()
            .join(".cac/themes/classic-blue/theme.typ")
            .is_file()
    );

    fs::write(directory.path().join("cv.md"), CLEAN_MARKDOWN).unwrap();
    fs::write(
        directory.path().join("settings.json"),
        r#"{"root":"cv.md","theme":"classic-blue"}"#,
    )
    .unwrap();
    Command::cargo_bin("cac")
        .unwrap()
        .current_dir(directory.path())
        .args(["build"])
        .assert()
        .success()
        .stdout(predicates::str::contains("THEME classic-blue (project)"));
    assert!(directory.path().join("offering/cv.pdf").is_file());

    Command::cargo_bin("cac")
        .unwrap()
        .current_dir(directory.path())
        .env("CAC_THEME_REGISTRY", &registry)
        .args(["theme", "install", "classic-left", "--local"])
        .assert()
        .success()
        .stdout("INSTALLED classic-left (project)\n");
    fs::write(
        directory.path().join("settings.json"),
        r#"{"root":"cv.md","theme":"classic-left"}"#,
    )
    .unwrap();
    Command::cargo_bin("cac")
        .unwrap()
        .current_dir(directory.path())
        .args(["build"])
        .assert()
        .success()
        .stdout(predicates::str::contains("THEME classic-left (project)"));
}

#[test]
fn themes_reject_non_web_author_urls() {
    let registry = tempdir().unwrap();
    let theme = registry.path().join("classic-blue");
    fs::create_dir(&theme).unwrap();
    fs::write(
        registry.path().join("index.json"),
        include_str!("../../../themes/index.json"),
    )
    .unwrap();
    let manifest = include_str!("../../../themes/classic-blue/theme.json").replace(
        "https://github.com/voxvanhieu/code-a-cv/graphs/contributors",
        "mailto:themes@example.com",
    );
    fs::write(theme.join("theme.json"), manifest).unwrap();

    Command::cargo_bin("cac")
        .unwrap()
        .env(
            "CAC_THEME_REGISTRY",
            format!("file://{}", registry.path().display()),
        )
        .args(["theme", "info", "classic-blue"])
        .assert()
        .code(1)
        .stdout("")
        .stderr(predicates::str::contains("invalid author URL"));
}

#[test]
fn themes_install_rejects_a_download_with_the_wrong_checksum() {
    let directory = tempdir().unwrap();
    let registry = tempdir().unwrap();
    let theme = registry.path().join("classic-blue");
    fs::create_dir(&theme).unwrap();
    fs::write(
        registry.path().join("index.json"),
        include_str!("../../../themes/index.json"),
    )
    .unwrap();
    let manifest = include_str!("../../../themes/classic-blue/theme.json").replace(
        "32a12bccc99e93c7756995325358fd5e3cf09c552380fa13b915a416777681b9",
        "0000000000000000000000000000000000000000000000000000000000000000",
    );
    fs::write(theme.join("theme.json"), manifest).unwrap();
    fs::write(
        theme.join("theme.typ"),
        include_str!("../../../themes/classic-blue/theme.typ"),
    )
    .unwrap();

    Command::cargo_bin("cac")
        .unwrap()
        .current_dir(directory.path())
        .env(
            "CAC_THEME_REGISTRY",
            format!("file://{}", registry.path().display()),
        )
        .args(["theme", "install", "classic-blue", "--local"])
        .assert()
        .code(1)
        .stdout("")
        .stderr(predicates::str::contains("failed checksum verification"));
    assert!(!directory.path().join(".cac/themes/classic-blue").exists());
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
