use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};
use thiserror::Error;

pub const DEFAULT_THEME: &str = "classic";

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct PageMargins {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bottom: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub left: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub right: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct PageSettings {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub paper: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub margin: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub margins: Option<PageMargins>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct TypographySettings {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub font: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub heading_font: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub font_size: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line_spacing: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct StyleSettings {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub accent_color: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body_alignment: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub link_underline: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub header_alignment: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub highlight_bullet: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct SpacingSettings {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub list_item: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub section: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub section_heading: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub entry: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct PaginationSettings {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allow_entry_page_break: Option<bool>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct Settings {
    #[serde(rename = "$schema", skip_serializing_if = "Option::is_none")]
    pub schema: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub root: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub naming: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub theme: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page: Option<PageSettings>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub typography: Option<TypographySettings>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub style: Option<StyleSettings>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub spacing: Option<SpacingSettings>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pagination: Option<PaginationSettings>,
}

#[derive(Debug, Error)]
pub enum SettingsError {
    #[error("could not read settings `{path}`: {source}")]
    Read {
        path: String,
        source: std::io::Error,
    },
    #[error("invalid settings in `{path}`: {source}")]
    Json {
        path: String,
        source: serde_json::Error,
    },
    #[error(
        "invalid theme name `{0}`; use lowercase ASCII letters, numbers, hyphens, or underscores"
    )]
    ThemeName(String),
    #[error("invalid `{field}` value `{value}`; expected a positive Typst length")]
    Length { field: &'static str, value: String },
    #[error("invalid `page.paper` value `{0}`; expected `a4` or `us-letter`")]
    Paper(String),
    #[error("invalid `typography.font` value; expected a non-empty font family")]
    Font,
    #[error("invalid `typography.heading_font` value; expected a non-empty font family")]
    HeadingFont,
    #[error(
        "invalid `style.accent_color` value `{0}`; expected a six-digit hex color such as `#1f4e79`"
    )]
    AccentColor(String),
    #[error("invalid `{field}` value `{value}`; expected one of: {expected}")]
    Choice {
        field: &'static str,
        value: String,
        expected: &'static str,
    },
    #[error("invalid `style.highlight_bullet` value; expected a non-empty single-line string")]
    HighlightBullet,
    #[error("invalid `root` value `{0}`; expected a CV file name without directory components")]
    Root(String),
    #[error(
        "invalid `naming` value `{0}`; expected a non-empty file name pattern without directory components"
    )]
    Naming(String),
}

impl Settings {
    pub fn from_path(path: &Path) -> Result<Self, SettingsError> {
        let bytes = fs::read(path).map_err(|source| SettingsError::Read {
            path: path.display().to_string(),
            source,
        })?;
        let settings = serde_json::from_slice(&bytes).map_err(|source| SettingsError::Json {
            path: path.display().to_string(),
            source,
        })?;
        Self::validate(settings)
    }

    pub fn validate(settings: Self) -> Result<Self, SettingsError> {
        if let Some(root) = &settings.root
            && !valid_root(root)
        {
            return Err(SettingsError::Root(root.clone()));
        }
        if let Some(naming) = &settings.naming
            && !valid_root(naming)
        {
            return Err(SettingsError::Naming(naming.clone()));
        }
        if let Some(theme) = &settings.theme {
            validate_theme_name(theme)?;
        }
        let page = settings.page.as_ref();
        let typography = settings.typography.as_ref();
        let style = settings.style.as_ref();
        let spacing = settings.spacing.as_ref();
        if let Some(paper) = page.and_then(|page| page.paper.as_ref())
            && !matches!(paper.as_str(), "a4" | "us-letter")
        {
            return Err(SettingsError::Paper(paper.clone()));
        }
        if typography
            .and_then(|typography| typography.font.as_ref())
            .is_some_and(|font| font.trim().is_empty())
        {
            return Err(SettingsError::Font);
        }
        if typography
            .and_then(|typography| typography.heading_font.as_ref())
            .is_some_and(|font| font.trim().is_empty())
        {
            return Err(SettingsError::HeadingFont);
        }
        for (field, value) in [
            ("page.margin", page.and_then(|page| page.margin.as_ref())),
            (
                "typography.font_size",
                typography.and_then(|typography| typography.font_size.as_ref()),
            ),
            (
                "typography.line_spacing",
                typography.and_then(|typography| typography.line_spacing.as_ref()),
            ),
            (
                "spacing.list_item",
                spacing.and_then(|spacing| spacing.list_item.as_ref()),
            ),
            (
                "spacing.section",
                spacing.and_then(|spacing| spacing.section.as_ref()),
            ),
            (
                "spacing.entry",
                spacing.and_then(|spacing| spacing.entry.as_ref()),
            ),
            (
                "spacing.section_heading",
                spacing.and_then(|spacing| spacing.section_heading.as_ref()),
            ),
        ] {
            if let Some(value) = value
                && !valid_length(value)
            {
                return Err(SettingsError::Length {
                    field,
                    value: value.clone(),
                });
            }
        }
        if let Some(margins) = page.and_then(|page| page.margins.as_ref()) {
            for (field, value) in [
                ("page.margins.top", &margins.top),
                ("page.margins.bottom", &margins.bottom),
                ("page.margins.left", &margins.left),
                ("page.margins.right", &margins.right),
            ] {
                if let Some(value) = value
                    && !valid_length(value)
                {
                    return Err(SettingsError::Length {
                        field,
                        value: value.clone(),
                    });
                }
            }
        }
        if let Some(color) = style.and_then(|style| style.accent_color.as_ref())
            && !valid_hex_color(color)
        {
            return Err(SettingsError::AccentColor(color.clone()));
        }
        validate_choice(
            "style.body_alignment",
            style.and_then(|style| style.body_alignment.as_ref()),
            &["left", "justified"],
            "left, justified",
        )?;
        validate_choice(
            "style.header_alignment",
            style.and_then(|style| style.header_alignment.as_ref()),
            &["left", "center", "right"],
            "left, center, right",
        )?;
        if style
            .and_then(|style| style.highlight_bullet.as_ref())
            .is_some_and(|bullet| bullet.trim().is_empty() || bullet.contains(['\n', '\r']))
        {
            return Err(SettingsError::HighlightBullet);
        }
        Ok(settings)
    }

    pub fn theme_name(&self) -> &str {
        self.theme.as_deref().unwrap_or(DEFAULT_THEME)
    }
}

fn validate_choice(
    field: &'static str,
    value: Option<&String>,
    choices: &[&str],
    expected: &'static str,
) -> Result<(), SettingsError> {
    if let Some(value) = value
        && !choices.contains(&value.as_str())
    {
        return Err(SettingsError::Choice {
            field,
            value: value.clone(),
            expected,
        });
    }
    Ok(())
}

fn valid_hex_color(value: &str) -> bool {
    value.len() == 7
        && value.starts_with('#')
        && value.bytes().skip(1).all(|byte| byte.is_ascii_hexdigit())
}

fn valid_root(root: &str) -> bool {
    !root.is_empty() && !matches!(root, "." | "..") && !root.contains(['/', '\\'])
}

pub fn validate_theme_name(name: &str) -> Result<(), SettingsError> {
    let valid = !name.is_empty()
        && name.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'-' | b'_')
        });
    if valid {
        Ok(())
    } else {
        Err(SettingsError::ThemeName(name.into()))
    }
}

pub fn settings_schema() -> serde_json::Value {
    serde_json::json!({
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "$id": "https://raw.githubusercontent.com/voxvanhieu/code-a-cv/main/settings.schema.json",
        "title": "Code a CV settings",
        "description": "Rendering and project settings for the cac command-line application.",
        "type": "object",
        "additionalProperties": false,
        "properties": {
            "$schema": {
                "type": "string",
                "format": "uri-reference",
                "description": "Location of this JSON Schema for editor validation and completion."
            },
            "root": {
                "type": "string",
                "pattern": "^(?!\\.{1,2}$)[^/\\\\]+$",
                "description": "CV source filename in the same directory as settings.json."
            },
            "naming": {
                "type": "string",
                "pattern": "^(?!\\.{1,2}$)[^/\\\\]+$",
                "description": "Build artifact filename. The file extension is added automatically."
            },
            "theme": {
                "type": "string",
                "pattern": "^[a-z0-9_-]+$",
                "description": "Embedded, project-local, or user-installed theme name."
            },
            "page": { "$ref": "#/$defs/page" },
            "typography": { "$ref": "#/$defs/typography" },
            "style": { "$ref": "#/$defs/style" },
            "spacing": { "$ref": "#/$defs/spacing" },
            "pagination": { "$ref": "#/$defs/pagination" }
        },
        "$defs": {
            "positiveLength": {
                "type": "string",
                "pattern": "^(?=.*[1-9])(?:[0-9]+(?:\\.[0-9]+)?|\\.[0-9]+)(?:pt|mm|cm|in|em|rem|%)$",
                "description": "A positive Typst length with an explicit unit."
            },
            "page": {
                "type": "object",
                "additionalProperties": false,
                "properties": {
                    "paper": { "type": "string", "enum": ["a4", "us-letter"] },
                    "margin": { "$ref": "#/$defs/positiveLength" },
                    "margins": {
                        "type": "object",
                        "additionalProperties": false,
                        "properties": {
                            "top": { "$ref": "#/$defs/positiveLength" },
                            "bottom": { "$ref": "#/$defs/positiveLength" },
                            "left": { "$ref": "#/$defs/positiveLength" },
                            "right": { "$ref": "#/$defs/positiveLength" }
                        }
                    }
                }
            },
            "typography": {
                "type": "object",
                "additionalProperties": false,
                "properties": {
                    "font": { "type": "string", "minLength": 1 },
                    "heading_font": { "type": "string", "minLength": 1 },
                    "font_size": { "$ref": "#/$defs/positiveLength" },
                    "line_spacing": { "$ref": "#/$defs/positiveLength" }
                }
            },
            "style": {
                "type": "object",
                "additionalProperties": false,
                "properties": {
                    "accent_color": { "type": "string", "pattern": "^#[0-9A-Fa-f]{6}$" },
                    "body_alignment": { "type": "string", "enum": ["left", "justified"] },
                    "header_alignment": { "type": "string", "enum": ["left", "center", "right"] },
                    "link_underline": { "type": "boolean" },
                    "highlight_bullet": {
                        "type": "string",
                        "minLength": 1,
                        "pattern": "^[^\\r\\n]+$"
                    }
                }
            },
            "spacing": {
                "type": "object",
                "additionalProperties": false,
                "properties": {
                    "list_item": { "$ref": "#/$defs/positiveLength" },
                    "section": { "$ref": "#/$defs/positiveLength" },
                    "section_heading": { "$ref": "#/$defs/positiveLength" },
                    "entry": { "$ref": "#/$defs/positiveLength" }
                }
            },
            "pagination": {
                "type": "object",
                "additionalProperties": false,
                "properties": {
                    "allow_entry_page_break": { "type": "boolean" }
                }
            }
        }
    })
}

fn valid_length(value: &str) -> bool {
    const UNITS: [&str; 7] = ["pt", "mm", "cm", "in", "em", "rem", "%"];
    let value = value.trim();
    UNITS.iter().any(|unit| {
        value
            .strip_suffix(unit)
            .and_then(|number| number.parse::<f64>().ok())
            .is_some_and(|number| number.is_finite() && number > 0.0)
    })
}
