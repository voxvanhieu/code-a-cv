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
}
