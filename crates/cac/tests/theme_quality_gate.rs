use std::fs;
use std::io::Read;
#[cfg(unix)]
use std::os::unix::fs::{PermissionsExt, symlink};
#[cfg(unix)]
use std::os::unix::net::UnixListener;
use std::path::Path;

use assert_cmd::Command;
use predicates::str::contains;
use sha2::{Digest, Sha256};
use tempfile::{TempDir, tempdir};

fn cac(root: &Path, args: &[&str]) -> Command {
    let mut command = Command::cargo_bin("cac").unwrap();
    command.current_dir(root).args(args);
    command
}

fn project() -> TempDir {
    let root = tempdir().unwrap();
    cac(
        root.path(),
        &[
            "theme",
            "init",
            "portfolio",
            "--author",
            "Nguyễn Анна",
            "--author-url",
            "https://example.com/",
        ],
    )
    .assert()
    .success();
    edit_manifest(root.path(), |manifest| {
        manifest["description"] = "A useful theme".into()
    });
    root
}

fn edit_manifest(root: &Path, edit: impl FnOnce(&mut serde_json::Value)) {
    let path = root.join(".cac/themes/portfolio/theme.json");
    let mut value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    edit(&mut value);
    fs::write(path, serde_json::to_vec_pretty(&value).unwrap()).unwrap();
}

fn artifacts(root: &Path) -> Vec<Vec<u8>> {
    [
        ".cac/themes/portfolio/theme.json",
        ".cac/themes/portfolio/README.md",
        ".cac/themes/portfolio/preview.jpg",
        "offering/portfolio.pdf",
        "portfolio.zip",
    ]
    .iter()
    .map(|path| fs::read(root.join(path)).unwrap())
    .collect()
}

#[test]
fn invalid_arguments_fail_before_any_prompt_or_files() {
    let cases = [
        vec!["--author", "bad\nauthor"],
        vec!["--author", "  "],
        vec!["--author-url", "mailto:ada@example.com"],
        vec!["--author-url", "/relative"],
        vec!["classic"],
        vec!["base"],
        vec!["main"],
        vec!["Uppercase"],
        vec!["../escape"],
        vec![""],
    ];
    for args in cases {
        let root = tempdir().unwrap();
        let output = cac(root.path(), &["theme", "init"])
            .args(args)
            .output()
            .unwrap();
        assert!(!output.status.success());
        let error = String::from_utf8(output.stderr).unwrap();
        assert!(!error.contains("Theme name:"), "{error}");
        assert!(!error.contains("input ended"), "{error}");
        assert_eq!(fs::read_dir(root.path()).unwrap().count(), 0);
    }
}

#[test]
fn initialization_never_overwrites_existing_targets() {
    for target in [
        "cv.md",
        "settings.json",
        ".cac/settings.schema.json",
        ".cac/themes/portfolio",
    ] {
        let root = tempdir().unwrap();
        let path = root.path().join(target);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, "preserve me").unwrap();
        cac(
            root.path(),
            &[
                "theme",
                "init",
                "portfolio",
                "--author",
                "Ada",
                "--author-url",
                "https://example.com",
            ],
        )
        .assert()
        .failure();
        assert_eq!(fs::read(&path).unwrap(), b"preserve me");
    }
}

#[test]
fn eof_at_every_prompt_leaves_no_project() {
    for input in ["", "portfolio\n", "portfolio\nAda\n"] {
        let root = tempdir().unwrap();
        cac(root.path(), &["theme", "init"])
            .write_stdin(input)
            .assert()
            .failure()
            .stderr(contains("input ended"));
        assert_eq!(fs::read_dir(root.path()).unwrap().count(), 0);
    }
}

#[test]
fn test_and_pack_require_consistent_project_settings() {
    for settings in [
        "{}",
        r#"{"theme":"portfolio"}"#,
        r#"{"theme":"other","themeProject":"portfolio"}"#,
        r#"{"themeProject":"portfolio"}"#,
        r#"{"theme":"../bad","themeProject":"../bad"}"#,
    ] {
        let root = tempdir().unwrap();
        fs::write(root.path().join("settings.json"), settings).unwrap();
        for command in ["test", "pack"] {
            cac(root.path(), &["theme", command]).assert().failure();
            assert!(!root.path().join("offering").exists());
        }
    }
}

#[test]
fn invalid_metadata_and_missing_entrypoint_fail_without_output() {
    for (key, value) in [
        ("name", serde_json::json!("other")),
        ("description", serde_json::json!(" ")),
        ("author", serde_json::json!("bad\nauthor")),
        ("license", serde_json::json!("")),
        ("author_url", serde_json::json!("file:///tmp")),
    ] {
        let root = project();
        edit_manifest(root.path(), |manifest| manifest[key] = value);
        cac(root.path(), &["theme", "test"]).assert().failure();
        assert!(!root.path().join("offering").exists());
    }
    let root = project();
    fs::remove_file(root.path().join(".cac/themes/portfolio/theme.typ")).unwrap();
    cac(root.path(), &["theme", "test"])
        .assert()
        .failure()
        .stderr(contains("entrypoint"));
}

#[test]
fn failed_render_preserves_every_previous_artifact() {
    let root = project();
    cac(root.path(), &["theme", "pack"]).assert().success();
    let before = artifacts(root.path());
    fs::write(
        root.path().join(".cac/themes/portfolio/theme.typ"),
        "#let theme = (broken",
    )
    .unwrap();
    for command in ["test", "pack"] {
        cac(root.path(), &["theme", command]).assert().failure();
        assert_eq!(before, artifacts(root.path()));
    }
}

#[test]
fn archive_contains_exact_manifest_bytes_hashes_and_normalized_entries() {
    let root = project();
    let settings_path = root.path().join("settings.json");
    let mut settings: serde_json::Value =
        serde_json::from_slice(&fs::read(&settings_path).unwrap()).unwrap();
    settings["page"] = serde_json::json!({"paper": "us-letter"});
    fs::write(settings_path, serde_json::to_vec(&settings).unwrap()).unwrap();
    let theme = root.path().join(".cac/themes/portfolio");
    fs::create_dir_all(theme.join("assets/日本語")).unwrap();
    fs::write(theme.join("assets/日本語/icon.txt"), "Unicode asset").unwrap();
    fs::write(theme.join("fonts/LICENSE.txt"), "Font license").unwrap();
    edit_manifest(root.path(), |m| {
        m["description"] = "<script>alert(1)</script>\n# Heading".into();
        m["author"] = "<img src=x onerror=alert(1)> | [Анна]".into();
        m["license"] = "MIT\n| injected | row |".into();
    });
    cac(root.path(), &["theme", "pack"])
        .assert()
        .success()
        .stdout(contains("To publish:"));
    let readme = fs::read_to_string(theme.join("README.md")).unwrap();
    assert!(!readme.contains("<script>"));
    assert!(!readme.contains("<img src=x"));
    assert!(!readme.contains("\n# Heading"));
    assert!(!readme.contains("\n| injected"));
    assert!(readme.contains("width=\"612\" height=\"792\""));
    let manifest: serde_json::Value =
        serde_json::from_slice(&fs::read(theme.join("theme.json")).unwrap()).unwrap();
    let bytes = fs::read(root.path().join("portfolio.zip")).unwrap();
    let mut archive = zip::ZipArchive::new(std::io::Cursor::new(&bytes)).unwrap();
    let mut expected = vec!["portfolio/theme.json".to_string()];
    for file in manifest["files"].as_array().unwrap() {
        let path = file["path"].as_str().unwrap();
        expected.push(format!("portfolio/{path}"));
        let mut entry = archive.by_name(&format!("portfolio/{path}")).unwrap();
        assert_eq!(entry.unix_mode().unwrap() & 0o777, 0o644);
        assert_eq!(entry.last_modified().unwrap(), zip::DateTime::default());
        let mut content = Vec::new();
        entry.read_to_end(&mut content).unwrap();
        assert_eq!(format!("{:x}", Sha256::digest(&content)), file["sha256"]);
        assert_eq!(content, fs::read(theme.join(path)).unwrap());
    }
    expected.sort();
    assert_eq!(archive.file_names().collect::<Vec<_>>(), expected);
    let mut archived_manifest = Vec::new();
    archive
        .by_name("portfolio/theme.json")
        .unwrap()
        .read_to_end(&mut archived_manifest)
        .unwrap();
    assert_eq!(
        archived_manifest,
        fs::read(theme.join("theme.json")).unwrap()
    );
    cac(root.path(), &["theme", "pack"]).assert().success();
    assert_eq!(bytes, fs::read(root.path().join("portfolio.zip")).unwrap());
}

#[test]
fn failed_archive_replacement_cleans_up_temporary_files() {
    let root = project();
    fs::create_dir(root.path().join("portfolio.zip")).unwrap();
    let before = fs::read_dir(root.path()).unwrap().count();
    cac(root.path(), &["theme", "pack"]).assert().failure();
    // The validation phase creates only offering/ at the project root.
    assert_eq!(fs::read_dir(root.path()).unwrap().count(), before + 1);
    assert!(root.path().join("portfolio.zip").is_dir());
}

#[cfg(unix)]
#[test]
fn unsafe_files_and_symlinks_are_rejected_before_output_changes() {
    let root = project();
    cac(root.path(), &["theme", "pack"]).assert().success();
    let before = artifacts(root.path());
    let theme = root.path().join(".cac/themes/portfolio");
    for name in ["assets\\alias.txt", "C:drive.txt", "control\nname"] {
        fs::write(theme.join(name), "unsafe").unwrap();
        cac(root.path(), &["theme", "pack"])
            .assert()
            .failure()
            .stderr(contains("unsafe theme path"));
        assert_eq!(before, artifacts(root.path()));
        fs::remove_file(theme.join(name)).unwrap();
    }
    symlink(root.path().join("cv.md"), theme.join("assets/link")).unwrap();
    cac(root.path(), &["theme", "pack"])
        .assert()
        .failure()
        .stderr(contains("symlinks"));
    assert_eq!(before, artifacts(root.path()));
    fs::remove_file(theme.join("assets/link")).unwrap();
    fs::rename(&theme, root.path().join("external-theme")).unwrap();
    symlink(root.path().join("external-theme"), &theme).unwrap();
    cac(root.path(), &["theme", "test"])
        .assert()
        .failure()
        .stderr(contains("symlinks"));
    assert_eq!(before, artifacts(root.path()));
}

#[cfg(unix)]
#[test]
fn init_rejects_dangling_targets_and_symlinked_parents() {
    for target in ["cv.md", ".cac"] {
        let root = tempdir().unwrap();
        let external = tempdir().unwrap();
        let destination = if target == "cv.md" {
            external.path().join("missing")
        } else {
            external.path().to_owned()
        };
        symlink(&destination, root.path().join(target)).unwrap();
        cac(
            root.path(),
            &[
                "theme",
                "init",
                "portfolio",
                "--author",
                "Ada",
                "--author-url",
                "https://example.com",
            ],
        )
        .assert()
        .failure();
        assert_eq!(fs::read_dir(external.path()).unwrap().count(), 0);
        assert_eq!(fs::read_dir(root.path()).unwrap().count(), 1);
    }
}

#[test]
fn development_project_allows_build_discovery_and_unrelated_removal() {
    let root = project();
    cac(root.path(), &["build"])
        .assert()
        .success()
        .stdout(contains("portfolio"));
    cac(root.path(), &["theme", "list"])
        .assert()
        .success()
        .stdout(contains("portfolio"));
    let unrelated = root.path().join(".cac/themes/unrelated");
    fs::create_dir_all(&unrelated).unwrap();
    fs::write(unrelated.join("theme.typ"), "").unwrap();
    cac(root.path(), &["theme", "remove", "unrelated", "--local"])
        .assert()
        .success();
    assert!(!unrelated.exists());
    assert!(
        root.path()
            .join(".cac/themes/portfolio/theme.typ")
            .is_file()
    );
}

#[test]
fn invalid_generated_destination_preserves_other_successful_artifacts() {
    let root = project();
    cac(root.path(), &["theme", "pack"]).assert().success();
    let before = artifacts(root.path());
    let pdf = root.path().join("offering/portfolio.pdf");
    fs::remove_file(&pdf).unwrap();
    fs::create_dir(&pdf).unwrap();
    fs::write(
        root.path().join("cv.md"),
        "# A different representative CV\n\n## Skills\n\n- Rust\n",
    )
    .unwrap();
    cac(root.path(), &["theme", "test"])
        .assert()
        .failure()
        .stderr(contains("not a regular file"));
    for (index, path) in [
        ".cac/themes/portfolio/theme.json",
        ".cac/themes/portfolio/README.md",
        ".cac/themes/portfolio/preview.jpg",
    ]
    .iter()
    .enumerate()
    {
        assert_eq!(fs::read(root.path().join(path)).unwrap(), before[index]);
    }
    assert_eq!(
        fs::read(root.path().join("portfolio.zip")).unwrap(),
        before[4]
    );
}

#[cfg(unix)]
#[test]
fn packaging_does_not_follow_a_predictable_temporary_zip_symlink() {
    let root = project();
    let external = tempdir().unwrap();
    let victim = external.path().join("keep.txt");
    fs::write(&victim, "keep").unwrap();
    symlink(&victim, root.path().join(".portfolio.zip.tmp")).unwrap();
    cac(root.path(), &["theme", "pack"]).assert().success();
    assert_eq!(fs::read(victim).unwrap(), b"keep");
}

#[cfg(unix)]
#[test]
fn testing_preserves_read_only_sources_and_rejects_unsupported_entries() {
    let root = project();
    let theme = root.path().join(".cac/themes/portfolio");
    let source = theme.join("theme.typ");
    let before = fs::read(&source).unwrap();
    fs::set_permissions(&source, fs::Permissions::from_mode(0o444)).unwrap();
    cac(root.path(), &["theme", "pack"]).assert().success();
    assert_eq!(fs::read(&source).unwrap(), before);
    assert_eq!(
        fs::metadata(&source).unwrap().permissions().mode() & 0o777,
        0o444
    );
    let before = artifacts(root.path());
    // Keep the socket path short enough for macOS sockaddr_un.
    let socket_dir = tempfile::Builder::new()
        .prefix("cac")
        .tempdir_in("/tmp")
        .unwrap();
    let socket = socket_dir.path().join("socket");
    let listener = UnixListener::bind(&socket).unwrap();
    fs::rename(&socket, theme.join("socket")).unwrap();
    cac(root.path(), &["theme", "test"])
        .assert()
        .failure()
        .stderr(contains("unsupported theme entry"));
    assert_eq!(artifacts(root.path()), before);
    drop(listener);
}

#[test]
fn shared_test_keeps_fixture_pdfs_out_of_the_package_inventory() {
    let root = project();
    cac(root.path(), &["theme", "test", "--shared"])
        .assert()
        .success();
    assert!(
        root.path()
            .join("offering/portfolio-shared-complete.pdf")
            .is_file()
    );
    assert!(
        root.path()
            .join("offering/portfolio-shared-long.pdf")
            .is_file()
    );
    let manifest: serde_json::Value = serde_json::from_slice(
        &fs::read(root.path().join(".cac/themes/portfolio/theme.json")).unwrap(),
    )
    .unwrap();
    assert!(
        manifest["files"]
            .as_array()
            .unwrap()
            .iter()
            .all(|file| !file["path"].as_str().unwrap().contains("shared"))
    );
    let preview = fs::read(root.path().join(".cac/themes/portfolio/preview.jpg")).unwrap();
    cac(root.path(), &["theme", "test"]).assert().success();
    assert_eq!(
        preview,
        fs::read(root.path().join(".cac/themes/portfolio/preview.jpg")).unwrap()
    );
}

#[test]
fn shared_failure_names_the_fixture_and_preserves_previous_artifacts() {
    let root = project();
    cac(root.path(), &["theme", "pack"]).assert().success();
    let before = artifacts(root.path());
    fs::write(root.path().join(".cac/themes/portfolio/theme.typ"), "#import \"/.cac/base.typ\" as base\n#let theme = base.extend(components: (header: ctx => [Missing profile]))\n").unwrap();
    cac(root.path(), &["theme", "test", "--shared"])
        .assert()
        .failure()
        .stderr(contains("shared theme fixture `minimal` failed"));
    assert_eq!(before, artifacts(root.path()));
}

#[test]
fn theme_projects_package_without_version_metadata() {
    let root = project();
    let manifest: serde_json::Value = serde_json::from_slice(
        &fs::read(root.path().join(".cac/themes/portfolio/theme.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(
        manifest
            .as_object()
            .unwrap()
            .keys()
            .filter(|key| key.contains("version") || key.contains("api"))
            .count(),
        0
    );
    cac(root.path(), &["theme", "pack"]).assert().success();
}
