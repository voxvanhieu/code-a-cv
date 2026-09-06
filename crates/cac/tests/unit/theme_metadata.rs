use super::*;

fn theme(author_url: Option<url::Url>) -> ThemeMetadata {
    ThemeMetadata {
        name: "my-theme".into(),
        description: "Clear & compact".into(),
        author: "Ada | Team".into(),
        author_url,
        license: "MIT".into(),
        preview: Some("preview.jpg".into()),
        files: Vec::new(),
    }
}

#[test]
fn readme_links_author_and_escapes_content() {
    let output = readme(
        &theme(Some(url::Url::parse("https://example.com/ada").unwrap())),
        612,
        792,
    );
    assert!(output.contains("[Ada \\| Team](https://example.com/ada)"));
    assert!(output.contains("width=\"612\" height=\"792\""));
}

#[test]
fn readme_keeps_an_author_without_a_url_unlinked() {
    let output = readme(&theme(None), 612, 792);
    assert!(output.contains("| Author | Ada \\| Team |"));
    assert!(!output.contains("[Ada"));
}

#[test]
fn safe_paths_reject_traversal_and_archive_separators() {
    assert!(safe_path("assets/icon.svg"));
    assert!(!safe_path("../theme.typ"));
    assert!(!safe_path("assets\\icon.svg"));
}

#[test]
fn rejects_noncanonical_and_nonportable_paths() {
    for path in [
        "",
        "/absolute",
        "a//b",
        "a/./b",
        "a/../b",
        "a/",
        "C:foo",
        "C:/foo",
        "bad\nname",
        "bad\0name",
    ] {
        assert!(!safe_path(path), "accepted {path:?}");
    }
    assert!(safe_path("assets/日本語.svg"));
}

#[test]
fn packaged_manifest_rejects_self_hash_duplicates_and_bad_hashes() {
    let mut manifest = theme(None);
    manifest.files = ["theme.typ", "README.md", "preview.jpg"]
        .into_iter()
        .map(|path| ThemeFile {
            path: path.into(),
            sha256: "a".repeat(64),
        })
        .collect();
    assert!(validate_packaged(&manifest).is_ok());
    for path in ["theme.json", "theme.typ", "assets/../bad", ""] {
        let mut invalid = manifest.clone();
        invalid.files.push(ThemeFile {
            path: path.into(),
            sha256: "a".repeat(64),
        });
        assert!(validate_packaged(&invalid).is_err(), "accepted {path:?}");
    }
    for hash in ["A".repeat(64), "g".repeat(64), "a".repeat(63)] {
        let mut invalid = manifest.clone();
        invalid.files[0].sha256 = hash;
        assert!(validate_packaged(&invalid).is_err());
    }
    for index in 0..3 {
        let mut invalid = manifest.clone();
        invalid.files.remove(index);
        assert!(validate_packaged(&invalid).is_err());
    }
}

#[test]
fn readme_treats_metadata_as_text_in_every_context() {
    let mut manifest = theme(Some(url::Url::parse("https://example.com/a(b)|c").unwrap()));
    manifest.author = "<script> & [Ada]".into();
    manifest.description = "# Heading\n![image](evil)\n<div>".into();
    manifest.license = "MIT\n| forged | row |".into();
    let output = readme(&manifest, 123, 456);
    assert!(output.contains("&lt;script&gt; &amp;"));
    assert!(!output.contains("<div>"));
    assert!(!output.contains("\n| forged"));
    assert!(!output.contains("\n# Heading"));
    assert!(output.contains("https://example.com/a%28b%29%7Cc"));
}
