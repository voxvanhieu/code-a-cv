use super::*;

#[test]
fn theme_project_must_match_selected_theme() {
    let settings = Settings {
        theme: Some("one".into()),
        theme_project: Some("two".into()),
        ..Settings::default()
    };
    assert!(matches!(
        Settings::validate(settings),
        Err(SettingsError::ThemeProjectMismatch)
    ));
}

#[test]
fn schema_includes_theme_project() {
    assert_eq!(
        settings_schema()["properties"]["themeProject"]["pattern"],
        "^[a-z0-9_-]+$"
    );
}
