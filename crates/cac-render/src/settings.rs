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
pub struct Settings {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub root: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub theme: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub paper: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page_margin: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page_margins: Option<PageMargins>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub font: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub heading_font: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub font_size: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line_spacing: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub list_item_spacing: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub section_spacing: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub entry_spacing: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub accent_color: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body_alignment: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub link_underline: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub header_alignment: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub section_heading_spacing: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub highlight_bullet: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allow_entry_page_break: Option<bool>,
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
    #[error("invalid `paper` value `{0}`; expected `a4` or `us-letter`")]
    Paper(String),
    #[error("invalid `font` value; expected a non-empty font family")]
    Font,
    #[error("invalid `heading_font` value; expected a non-empty font family")]
    HeadingFont,
    #[error("invalid `accent_color` value `{0}`; expected a six-digit hex color such as `#1f4e79`")]
    AccentColor(String),
    #[error("invalid `{field}` value `{value}`; expected one of: {expected}")]
    Choice {
        field: &'static str,
        value: String,
        expected: &'static str,
    },
    #[error("invalid `highlight_bullet` value; expected a non-empty single-line string")]
    HighlightBullet,
    #[error("invalid `root` value `{0}`; expected a CV file name without directory components")]
    Root(String),
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
        if let Some(theme) = &settings.theme {
            validate_theme_name(theme)?;
        }
        if let Some(paper) = &settings.paper
            && !matches!(paper.as_str(), "a4" | "us-letter")
        {
            return Err(SettingsError::Paper(paper.clone()));
        }
        if settings
            .font
            .as_ref()
            .is_some_and(|font| font.trim().is_empty())
        {
            return Err(SettingsError::Font);
        }
        if settings
            .heading_font
            .as_ref()
            .is_some_and(|font| font.trim().is_empty())
        {
            return Err(SettingsError::HeadingFont);
        }
        for (field, value) in [
            ("page_margin", &settings.page_margin),
            ("font_size", &settings.font_size),
            ("line_spacing", &settings.line_spacing),
            ("list_item_spacing", &settings.list_item_spacing),
            ("section_spacing", &settings.section_spacing),
            ("entry_spacing", &settings.entry_spacing),
            ("section_heading_spacing", &settings.section_heading_spacing),
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
        if let Some(margins) = &settings.page_margins {
            for (field, value) in [
                ("page_margins.top", &margins.top),
                ("page_margins.bottom", &margins.bottom),
                ("page_margins.left", &margins.left),
                ("page_margins.right", &margins.right),
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
        if let Some(color) = &settings.accent_color
            && !valid_hex_color(color)
        {
            return Err(SettingsError::AccentColor(color.clone()));
        }
        validate_choice(
            "body_alignment",
            &settings.body_alignment,
            &["left", "justified"],
            "left, justified",
        )?;
        validate_choice(
            "header_alignment",
            &settings.header_alignment,
            &["left", "center", "right"],
            "left, center, right",
        )?;
        if settings
            .highlight_bullet
            .as_ref()
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
    value: &Option<String>,
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
