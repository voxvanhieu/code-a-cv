use std::collections::BTreeSet;
use std::path::Path;

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
        if file.path == "theme.json"
            || !safe_path(&file.path)
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
    !value.is_empty()
        && !value.contains(['\\', ':'])
        && !value.chars().any(char::is_control)
        && value
            .split('/')
            .all(|part| !part.is_empty() && part != "." && part != "..")
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
        format!(
            "[{author}]({})",
            url.as_str()
                .replace('(', "%28")
                .replace(')', "%29")
                .replace('|', "%7C")
                .replace('<', "%3C")
                .replace('>', "%3E")
        )
    });
    format!(
        "# {title}\n\n{}\n\n<img src=\"preview.jpg\" alt=\"Preview of the {} theme\" width=\"{width}\" height=\"{height}\">\n\n## Theme information\n\n| Field | Value |\n|---|---|\n| Name | `{}` |\n| Author | {author} |\n| License | {} |\n| Entrypoint | `theme.typ` |\n",
        escape_markdown(&theme.description),
        escape_html(&title),
        theme.name,
        escape_markdown(&theme.license)
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
        match character {
            '&' => {
                output.push_str("&amp;");
                return output;
            }
            '<' => {
                output.push_str("&lt;");
                return output;
            }
            '>' => {
                output.push_str("&gt;");
                return output;
            }
            '\n' | '\r' => {
                output.push(' ');
                return output;
            }
            _ => {}
        }
        if matches!(
            character,
            '\\' | '|' | '[' | ']' | '`' | '*' | '_' | '#' | '!' | '+' | '-' | '.' | '(' | ')'
        ) {
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
#[path = "../../../tests/unit/theme_metadata.rs"]
mod tests;
