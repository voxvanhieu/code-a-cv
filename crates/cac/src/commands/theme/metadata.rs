use std::collections::BTreeSet;
use std::path::{Component, Path};

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct ThemeMetadata {
    pub name: String,
    pub description: String,
    pub author: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub author_url: Option<url::Url>,
    pub license: String,
    pub theme_api: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub preview: Option<String>,
    #[serde(default)]
    pub files: Vec<ThemeFile>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct ThemeFile {
    pub path: String,
    pub sha256: String,
}

pub(super) fn validate_editable(theme: &ThemeMetadata) -> Result<(), String> {
    cac_render::validate_theme_name(&theme.name).map_err(|error| error.to_string())?;
    if theme.theme_api != cac_render::THEME_API_VERSION {
        return Err(format!(
            "theme `{}` uses unsupported theme API {}; expected {}",
            theme.name,
            theme.theme_api,
            cac_render::THEME_API_VERSION
        ));
    }
    if theme.description.trim().is_empty()
        || theme.author.trim().is_empty()
        || theme.author.contains(['\n', '\r'])
        || theme.license.trim().is_empty()
    {
        return Err(format!("theme `{}` has incomplete metadata", theme.name));
    }
    if theme
        .author_url
        .as_ref()
        .is_some_and(|url| !matches!(url.scheme(), "http" | "https"))
    {
        return Err(format!("theme `{}` has an invalid author URL", theme.name));
    }
    Ok(())
}

pub(super) fn validate_packaged(theme: &ThemeMetadata) -> Result<(), String> {
    validate_editable(theme)?;
    let mut paths = BTreeSet::new();
    for file in &theme.files {
        if !safe_path(&file.path)
            || !paths.insert(file.path.as_str())
            || file.sha256.len() != 64
            || !file
                .sha256
                .bytes()
                .all(|byte| byte.is_ascii_digit() || matches!(byte, b'a'..=b'f'))
        {
            return Err(format!("theme `{}` has an invalid file entry", theme.name));
        }
    }
    for required in ["theme.typ", "README.md", "preview.jpg"] {
        if !paths.contains(required) {
            return Err(format!("theme `{}` is missing `{required}`", theme.name));
        }
    }
    if theme.preview.as_deref() != Some("preview.jpg") {
        return Err(format!("theme `{}` has an invalid preview", theme.name));
    }
    Ok(())
}

pub(super) fn safe_path(value: &str) -> bool {
    let path = Path::new(value);
    !value.contains('\\')
        && !path.is_absolute()
        && path
            .components()
            .all(|component| matches!(component, Component::Normal(_)))
}

pub(super) fn read(path: &Path) -> Result<ThemeMetadata, String> {
    let bytes = std::fs::read(path)
        .map_err(|error| format!("could not read `{}`: {error}", path.display()))?;
    serde_json::from_slice(&bytes).map_err(|error| format!("invalid `{}`: {error}", path.display()))
}

pub(super) fn bytes(theme: &ThemeMetadata) -> Result<Vec<u8>, serde_json::Error> {
    let mut bytes = serde_json::to_vec_pretty(theme)?;
    bytes.push(b'\n');
    Ok(bytes)
}

pub(super) fn readme(theme: &ThemeMetadata, width: u32, height: u32) -> String {
    let title = theme
        .name
        .split(['-', '_'])
        .filter(|part| !part.is_empty())
        .map(capitalize)
        .collect::<Vec<_>>()
        .join(" ");
    let author = escape_markdown(&theme.author);
    let author = theme.author_url.as_ref().map_or(author.clone(), |url| {
        format!("[{author}]({})", url.as_str().replace(')', "%29"))
    });
    format!(
        "# {title}\n\n{}\n\n<img src=\"preview.jpg\" alt=\"Preview of the {} theme\" width=\"{width}\" height=\"{height}\">\n\n## Theme information\n\n| Field | Value |\n|---|---|\n| Name | `{}` |\n| Author | {author} |\n| License | {} |\n| Theme API | {} |\n| Entrypoint | `theme.typ` |\n",
        escape_markdown(&theme.description),
        escape_html(&title),
        theme.name,
        escape_markdown(&theme.license),
        theme.theme_api
    )
}

fn capitalize(value: &str) -> String {
    let mut chars = value.chars();
    chars.next().map_or_else(String::new, |first| {
        first.to_uppercase().chain(chars).collect()
    })
}

fn escape_markdown(value: &str) -> String {
    value.chars().fold(String::new(), |mut output, character| {
        if matches!(character, '\\' | '|' | '[' | ']' | '`' | '*' | '_') {
            output.push('\\');
        }
        output.push(character);
        output
    })
}

fn escape_html(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn theme(author_url: Option<url::Url>) -> ThemeMetadata {
        ThemeMetadata {
            name: "my-theme".into(),
            description: "Clear & compact".into(),
            author: "Ada | Team".into(),
            author_url,
            license: "MIT".into(),
            theme_api: cac_render::THEME_API_VERSION,
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
}
