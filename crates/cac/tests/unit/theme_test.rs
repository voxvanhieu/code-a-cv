use super::super::metadata::ThemeMetadata;
use super::*;

#[test]
fn verification_rejects_modified_missing_and_undeclared_files() {
    let directory = tempfile::tempdir().unwrap();
    let mut manifest = ThemeMetadata {
        name: "example".into(),
        description: "Example theme".into(),
        author: "Ada".into(),
        author_url: None,
        license: "MIT".into(),
        preview: Some("preview.jpg".into()),
        files: Vec::new(),
    };
    for path in ["theme.typ", "README.md", "preview.jpg"] {
        fs::write(directory.path().join(path), b"original").unwrap();
        manifest.files.push(ThemeFile {
            path: path.into(),
            sha256: sha256_hex(b"original"),
        });
    }
    verify_files(directory.path(), &manifest).unwrap();
    fs::write(directory.path().join("theme.typ"), b"modified").unwrap();
    assert!(matches!(
        verify_files(directory.path(), &manifest),
        Err(Error::ThemeChecksum { .. })
    ));
    fs::write(directory.path().join("theme.typ"), b"original").unwrap();
    fs::write(directory.path().join("extra.txt"), b"undeclared").unwrap();
    assert!(verify_files(directory.path(), &manifest).is_err());
    fs::remove_file(directory.path().join("extra.txt")).unwrap();
    fs::remove_file(directory.path().join("preview.jpg")).unwrap();
    assert!(verify_files(directory.path(), &manifest).is_err());
}

#[test]
fn checksums_preserve_standard_lowercase_hex_encoding() {
    assert_eq!(
        sha256_hex(b""),
        "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
    );
    assert_eq!(
        sha256_hex(b"abc"),
        "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
    );
}
