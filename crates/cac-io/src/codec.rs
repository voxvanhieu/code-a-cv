use std::collections::BTreeSet;

use crate::json_resume::import_json_resume;
use crate::markdown::parse_markdown;
use cac_core::{CvDocument, DatePoint, EntryKind, Inline, RichText, supported_link};
use thiserror::Error;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InputFormat {
    Markdown,
    Yaml,
    Json,
    Toml,
    JsonResume,
}

impl InputFormat {
    pub fn from_extension(extension: &str) -> Option<Self> {
        match extension.to_ascii_lowercase().as_str() {
            "md" | "markdown" => Some(Self::Markdown),
            "yaml" | "yml" => Some(Self::Yaml),
            "json" => Some(Self::Json),
            "toml" => Some(Self::Toml),
            _ => None,
        }
    }
}

#[derive(Debug, Error)]
pub enum ParseError {
    #[error("line {line}: {message}")]
    Markdown { line: usize, message: String },
    #[error("invalid YAML at {path}: {message}")]
    Yaml { path: String, message: String },
    #[error("invalid JSON at {path}: {message}")]
    Json { path: String, message: String },
    #[error("invalid TOML: {0}")]
    Toml(#[from] toml::de::Error),
    #[error("invalid CV: {0}")]
    Validation(String),
}

pub fn parse(source: &str, format: InputFormat) -> Result<CvDocument, ParseError> {
    let mut cv = match format {
        InputFormat::Markdown => parse_markdown(source)?,
        InputFormat::Yaml => parse_yaml(source)?,
        InputFormat::Json => parse_json(source)?,
        InputFormat::Toml => toml::from_str(source)?,
        InputFormat::JsonResume => import_json_resume(source)?,
    };
    normalize(&mut cv);
    validate(&cv)?;
    Ok(cv)
}

fn parse_json(source: &str) -> Result<CvDocument, ParseError> {
    let mut deserializer = serde_json::Deserializer::from_str(source);
    serde_path_to_error::deserialize(&mut deserializer).map_err(|error| ParseError::Json {
        path: error.path().to_string(),
        message: error.inner().to_string(),
    })
}

fn parse_yaml(source: &str) -> Result<CvDocument, ParseError> {
    let deserializer = serde_yaml_ng::Deserializer::from_str(source);
    serde_path_to_error::deserialize(deserializer).map_err(|error| ParseError::Yaml {
        path: error.path().to_string(),
        message: error.inner().to_string(),
    })
}

pub fn validate(cv: &CvDocument) -> Result<(), ParseError> {
    if cv.profile.name.trim().is_empty() {
        return Err(ParseError::Validation(
            "profile.name must not be empty".into(),
        ));
    }
    if cv.profile.name.contains(['\n', '\r']) {
        return Err(ParseError::Validation(
            "profile.name must be a single line".into(),
        ));
    }
    for (index, contact) in cv.profile.contacts.iter().enumerate() {
        if contact.label.trim().is_empty() || contact.label.contains(['\n', '\r']) {
            return Err(ParseError::Validation(format!(
                "profile.contacts[{index}].label must contain a single nonempty line"
            )));
        }
        if contact
            .href
            .as_ref()
            .is_some_and(|url| !supported_link(url))
        {
            return Err(ParseError::Validation(format!(
                "profile.contacts[{index}].href has an unsupported URL scheme"
            )));
        }
    }
    if cv
        .profile
        .website
        .as_ref()
        .is_some_and(|url| !supported_link(url))
    {
        return Err(ParseError::Validation(
            "profile.website has an unsupported URL scheme".into(),
        ));
    }
    let mut ids = BTreeSet::new();
    for (section_index, section) in cv.sections.iter().enumerate() {
        if !section.kind.is_valid() {
            return Err(ParseError::Validation(format!(
                "sections[{section_index}].kind must be a nonempty identifier containing letters, numbers, hyphens, underscores, or dots"
            )));
        }
        if section.title.trim().is_empty() || section.title.contains(['\n', '\r']) {
            return Err(ParseError::Validation(format!(
                "sections[{section_index}].title must contain a single nonempty line"
            )));
        }
        if section.id.trim().is_empty() {
            return Err(ParseError::Validation(format!(
                "sections[{section_index}].id must not be empty"
            )));
        }
        if !ids.insert(section.id.as_str()) {
            return Err(ParseError::Validation(format!(
                "duplicate section id `{}`",
                section.id
            )));
        }
        for (entry_index, entry) in section.entries.iter().enumerate() {
            let path = format!("sections[{section_index}].entries[{entry_index}]");
            if entry.kind.date() == Some(&DatePoint::Present) {
                return Err(ParseError::Validation(format!(
                    "{path}.date cannot be Present; use a period for ongoing work"
                )));
            }
            if entry.kind.date().is_some() && entry.kind.period().is_some() {
                return Err(ParseError::Validation(format!(
                    "{path}: date and period conflict"
                )));
            }
            if entry.content.is_some()
                && (!entry.kind.highlights().is_empty()
                    || matches!(entry.kind, EntryKind::Text(_) | EntryKind::Prose(_)))
            {
                return Err(ParseError::Validation(format!(
                    "{path}: use either content or highlights/skills/body; put ordered prose and lists together in content"
                )));
            }
            if !matches!(entry.kind, EntryKind::Text(_) | EntryKind::Prose(_))
                && !entry.kind.heading().0.is_inline()
            {
                return Err(ParseError::Validation(format!(
                    "{path}: heading must use inline formatting; put paragraphs and lists in content"
                )));
            }
            let url = match &entry.kind {
                EntryKind::Project(value) => &value.url,
                EntryKind::Publication(value) => &value.url,
                _ => &None,
            };
            if url.as_ref().is_some_and(|url| !supported_link(url)) {
                return Err(ParseError::Validation(format!(
                    "{path}.url has an unsupported URL scheme"
                )));
            }
            if let Some(period) = entry.kind.period()
                && !period.is_valid()
            {
                return Err(ParseError::Validation(format!(
                    "sections[{section_index}].entries[{entry_index}].period is invalid: provide a known endpoint, use Present only as the end, and do not start after the end"
                )));
            }
            for (index, highlight) in entry.kind.highlights().iter().enumerate() {
                if highlight.is_empty() {
                    return Err(ParseError::Validation(format!(
                        "{path}: highlight/skill {index} must contain visible text; remove empty items"
                    )));
                }
            }
            if entry.kind.heading().0.is_empty() {
                return Err(ParseError::Validation(format!(
                    "sections[{section_index}].entries[{entry_index}] has an empty heading"
                )));
            }
        }
    }
    Ok(())
}

pub(crate) fn normalize(cv: &mut CvDocument) {
    for field in [
        &mut cv.profile.email,
        &mut cv.profile.phone,
        &mut cv.profile.location,
    ] {
        if field.as_ref().is_some_and(|value| value.trim().is_empty()) {
            *field = None;
        }
    }
    if cv.profile.summary.as_ref().is_some_and(RichText::is_empty) {
        cv.profile.summary = None;
    }
    for entry in cv
        .sections
        .iter_mut()
        .flat_map(|section| &mut section.entries)
    {
        if entry.content.as_ref().is_some_and(RichText::is_empty) {
            entry.content = None;
        }
        if let EntryKind::Experience(value) = &mut entry.kind
            && value
                .location
                .as_ref()
                .is_some_and(|value| value.trim().is_empty())
        {
            value.location = None;
        }
        if let EntryKind::Publication(value) = &mut entry.kind
            && value.publisher.as_ref().is_some_and(RichText::is_empty)
        {
            value.publisher = None;
        }
        if let Some(content) = &entry.content
            && let [Inline::List { start: None, items }] = content.0.as_slice()
            && entry.kind.highlights().is_empty()
        {
            let highlights = match &mut entry.kind {
                EntryKind::Experience(value) => Some(&mut value.highlights),
                EntryKind::Education(value) => Some(&mut value.highlights),
                EntryKind::Project(value) => Some(&mut value.highlights),
                EntryKind::Publication(value) => Some(&mut value.highlights),
                EntryKind::Custom(value) => Some(&mut value.highlights),
                EntryKind::SkillGroup(value) => Some(&mut value.skills),
                _ => None,
            };
            if let Some(highlights) = highlights {
                *highlights = items.clone();
                entry.content = None;
            }
        }
    }
}
