use super::*;

#[test]
fn unix_file_urls_keep_the_absolute_root() {
    let path = file_resource_path("file:///tmp/registry/index.json").unwrap();
    assert_eq!(path, PathBuf::from("/tmp/registry/index.json"));
}

#[test]
fn windows_drive_urls_drop_the_authority_slash() {
    let path = file_resource_path("file:///C:/Users/ada/.cac/index.json").unwrap();
    assert_eq!(path, PathBuf::from("C:/Users/ada/.cac/index.json"));
    assert_eq!(
        file_resource_path("file://C:/Users/ada/.cac/index.json").unwrap(),
        PathBuf::from("C:/Users/ada/.cac/index.json"),
        "two-slash drive URLs are tolerated"
    );
}

#[test]
fn unix_paths_are_never_treated_as_drives() {
    // `/C:/...` is only special behind a Windows drive letter; on any
    // platform a one-letter directory is still a directory.
    let path = file_resource_path("file:///C Fans/index.json").unwrap();
    assert_eq!(path, PathBuf::from("/C Fans/index.json"));
}

#[test]
fn percent_escapes_decode_into_paths() {
    let path = file_resource_path("file:///tmp/My%20Themes/index.json").unwrap();
    assert_eq!(path, PathBuf::from("/tmp/My Themes/index.json"));
    assert_eq!(
        file_resource_path("file:///tmp/100%25/index.json").unwrap(),
        PathBuf::from("/tmp/100%/index.json")
    );
    // A lone percent sign is not an escape.
    assert_eq!(
        file_resource_path("file:///tmp/100%.json").unwrap(),
        PathBuf::from("/tmp/100%.json")
    );
}

#[test]
fn non_file_schemes_are_not_paths() {
    assert_eq!(file_resource_path("https://example.com/index.json"), None);
}
