use std::collections::BTreeSet;

use cac_core::{
    CustomEntry, CvDocument, DatePoint, EducationEntry, Entry, EntryKind, ExperienceEntry, Origin,
    Period, Profile, ProjectEntry, PublicationEntry, RichText, Section, SectionKind,
    SkillGroupEntry, TagSet, TextEntry,
};
use chrono::NaiveDate;
use thiserror::Error;
use url::Url;

use crate::json_resume::import_json_resume;

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
    let cv = match format {
        InputFormat::Markdown => parse_markdown(source)?,
        InputFormat::Yaml => parse_yaml(source)?,
        InputFormat::Json => parse_json(source)?,
        InputFormat::Toml => toml::from_str(source)?,
        InputFormat::JsonResume => import_json_resume(source)?,
    };
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
    let mut ids = BTreeSet::new();
    for (section_index, section) in cv.sections.iter().enumerate() {
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
            if let Some(period) = entry.kind.period()
                && !period.is_valid()
            {
                return Err(ParseError::Validation(format!(
                    "sections[{section_index}].entries[{entry_index}].period starts after it ends"
                )));
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

pub fn parse_markdown(source: &str) -> Result<CvDocument, ParseError> {
    let mut profile = Profile::default();
    let mut sections = Vec::new();
    let mut section: Option<Section> = None;
    let mut entry: Option<EntryBuilder> = None;
    let mut last_direct_entry: Option<usize> = None;
    let mut saw_name = false;
    let mut before_sections = true;

    for (index, raw_line) in source.lines().enumerate() {
        let line_number = index + 1;
        let line = raw_line.trim();
        if line.is_empty() {
            continue;
        }
        if let Some(value) = line.strip_prefix("# ") {
            if saw_name || section.is_some() {
                return markdown_error(
                    line_number,
                    "the document must contain exactly one level-one name heading",
                );
            }
            profile.name = RichText::parse(value).plain();
            saw_name = true;
            continue;
        }
        if !saw_name {
            return markdown_error(
                line_number,
                "content must start with a level-one name heading",
            );
        }
        if let Some(value) = line.strip_prefix("## ") {
            finish_entry(&mut section, &mut entry);
            if let Some(previous) = section.take() {
                sections.push(previous);
            }
            let title = RichText::parse(value).plain();
            section = Some(Section {
                id: slugify(&title),
                kind: section_kind(&title),
                title,
                entries: Vec::new(),
                tags: TagSet::new(),
            });
            last_direct_entry = None;
            before_sections = false;
            continue;
        }
        if let Some(value) = line.strip_prefix("### ") {
            let current = section.as_mut().ok_or_else(|| ParseError::Markdown {
                line: line_number,
                message: "an entry heading must be inside a level-two section".into(),
            })?;
            finish_entry_for(current, &mut entry);
            entry = Some(EntryBuilder::new(value, line_number));
            last_direct_entry = None;
            continue;
        }
        if let Some(tags) = parse_tags(line) {
            if let Some(current) = entry.as_mut() {
                current.tags.extend(tags);
            } else if let Some(current) = section.as_mut() {
                if let Some(index) = last_direct_entry {
                    current.entries[index].tags.extend(tags);
                } else {
                    current.tags.extend(tags);
                }
            } else {
                return markdown_error(line_number, "tags must follow a section or entry heading");
            }
            continue;
        }
        if before_sections {
            parse_profile_line(&mut profile, line);
            continue;
        }
        if let Some(value) = line.strip_prefix("- ").or_else(|| line.strip_prefix("* ")) {
            if let Some(current) = entry.as_mut() {
                current.highlights.push(RichText::parse(value));
            } else {
                let current = section
                    .as_mut()
                    .expect("a section exists after before_sections");
                let kind = if current.kind == SectionKind::Skills {
                    EntryKind::SkillGroup(SkillGroupEntry {
                        name: RichText::parse(value),
                        skills: Vec::new(),
                    })
                } else {
                    EntryKind::Text(TextEntry {
                        body: RichText::parse(value),
                    })
                };
                current.entries.push(Entry {
                    kind,
                    tags: TagSet::new(),
                    origin: Origin {
                        path: format!("line {line_number}"),
                    },
                });
                last_direct_entry = Some(current.entries.len() - 1);
            }
            continue;
        }
        if let Some(current) = entry.as_mut()
            && current.period.is_none()
            && let Some(period) = parse_period(line)
        {
            current.period = Some(period.map_err(|message| ParseError::Markdown {
                line: line_number,
                message,
            })?);
            continue;
        }
        if section
            .as_ref()
            .is_some_and(|value| value.kind == SectionKind::Publications)
            && let Some(current) = entry.as_mut()
            && current.date.is_none()
            && let Some(date) = parse_date_point(line)
        {
            current.date = Some(date);
            continue;
        }
        return markdown_error(
            line_number,
            "expected a heading, date range, bullet, or tags comment",
        );
    }
    finish_entry(&mut section, &mut entry);
    if let Some(previous) = section {
        sections.push(previous);
    }
    if !saw_name {
        return markdown_error(1, "the document is missing a level-one name heading");
    }
    Ok(CvDocument { profile, sections })
}

fn markdown_error<T>(line: usize, message: &str) -> Result<T, ParseError> {
    Err(ParseError::Markdown {
        line,
        message: message.into(),
    })
}

fn parse_profile_line(profile: &mut Profile, line: &str) {
    let parts: Vec<&str> = line
        .split('·')
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .collect();
    let looks_like_contacts = parts
        .iter()
        .any(|part| part.contains('@') || part.starts_with('+') || Url::parse(part).is_ok());
    if !looks_like_contacts {
        profile.summary = Some(RichText::parse(line));
        return;
    }
    for part in parts {
        if part.contains('@') && profile.email.is_none() {
            profile.email = Some(part.trim_start_matches("mailto:").to_owned());
        } else if let Ok(url) = Url::parse(part) {
            profile.website = Some(url);
        } else if part.starts_with('+')
            || part
                .chars()
                .filter(|character| character.is_ascii_digit())
                .count()
                >= 7
        {
            profile.phone = Some(part.to_owned());
        } else if profile.location.is_none() {
            profile.location = Some(part.to_owned());
        }
    }
}

struct EntryBuilder {
    heading: String,
    period: Option<Period>,
    date: Option<DatePoint>,
    highlights: Vec<RichText>,
    tags: TagSet,
    line: usize,
}

impl EntryBuilder {
    fn new(heading: &str, line: usize) -> Self {
        Self {
            heading: heading.into(),
            period: None,
            date: None,
            highlights: Vec::new(),
            tags: TagSet::new(),
            line,
        }
    }

    fn build(self, section_kind: SectionKind) -> Vec<Entry> {
        let origin = Origin {
            path: format!("line {}", self.line),
        };
        let tags = self.tags;
        let kind = match section_kind {
            SectionKind::Experience => {
                let (first, second) = split_entry_heading(&self.heading);
                EntryKind::Experience(ExperienceEntry {
                    role: first,
                    organization: second,
                    location: None,
                    period: self.period,
                    highlights: self.highlights,
                })
            }
            SectionKind::Education => {
                let (first, second) = split_entry_heading(&self.heading);
                EntryKind::Education(EducationEntry {
                    qualification: first,
                    institution: second,
                    period: self.period,
                    highlights: self.highlights,
                })
            }
            SectionKind::Skills => EntryKind::SkillGroup(SkillGroupEntry {
                name: RichText::parse(&self.heading),
                skills: self.highlights,
            }),
            SectionKind::Projects => EntryKind::Project(ProjectEntry {
                name: RichText::parse(&self.heading),
                url: None,
                period: self.period,
                highlights: self.highlights,
            }),
            SectionKind::Publications => EntryKind::Publication(PublicationEntry {
                title: RichText::parse(&self.heading),
                publisher: None,
                date: self.date.or_else(|| self.period.map(|period| period.start)),
                url: None,
                highlights: self.highlights,
            }),
            SectionKind::Custom => EntryKind::Custom(CustomEntry {
                heading: RichText::parse(&self.heading),
                period: self.period,
                highlights: self.highlights,
            }),
        };
        vec![Entry { kind, tags, origin }]
    }
}

fn split_entry_heading(heading: &str) -> (RichText, RichText) {
    let (first, second) = heading.split_once(", ").unwrap_or((heading, ""));
    (RichText::parse(first), RichText::parse(second))
}

fn finish_entry(section: &mut Option<Section>, entry: &mut Option<EntryBuilder>) {
    if let Some(current) = section.as_mut() {
        finish_entry_for(current, entry);
    }
}

fn finish_entry_for(section: &mut Section, entry: &mut Option<EntryBuilder>) {
    if let Some(value) = entry.take() {
        section.entries.extend(value.build(section.kind));
    }
}

fn section_kind(title: &str) -> SectionKind {
    let lower = title.to_ascii_lowercase();
    if lower.contains("education") {
        SectionKind::Education
    } else if lower.contains("experience") || lower.contains("employment") {
        SectionKind::Experience
    } else if lower.contains("project") {
        SectionKind::Projects
    } else if lower.contains("publication") {
        SectionKind::Publications
    } else if lower.contains("skill") {
        SectionKind::Skills
    } else {
        SectionKind::Custom
    }
}

pub fn slugify(value: &str) -> String {
    let mut output = String::new();
    let mut separator = false;
    for character in value.chars().flat_map(char::to_lowercase) {
        if character.is_alphanumeric() {
            if separator && !output.is_empty() {
                output.push('-');
            }
            output.push(character);
            separator = false;
        } else {
            separator = true;
        }
    }
    output
}

fn parse_tags(line: &str) -> Option<TagSet> {
    let body = line.strip_prefix("<!--")?.strip_suffix("-->")?.trim();
    let values = body.strip_prefix("tags:")?;
    Some(
        values
            .split(',')
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_owned)
            .collect(),
    )
}

fn parse_period(value: &str) -> Option<Result<Period, String>> {
    let pair = value.split_once('–').or_else(|| value.split_once(" - "))?;
    let start = parse_date_point(pair.0.trim())
        .ok_or_else(|| format!("invalid start date `{}`", pair.0.trim()));
    let end = parse_date_point(pair.1.trim())
        .ok_or_else(|| format!("invalid end date `{}`", pair.1.trim()));
    Some(start.and_then(|start| {
        end.and_then(|end| Period::new(start, end).map_err(|error| error.to_string()))
    }))
}

pub fn parse_date_point(value: &str) -> Option<DatePoint> {
    if value.eq_ignore_ascii_case("present") || value.eq_ignore_ascii_case("current") {
        return Some(DatePoint::Present);
    }
    if let Ok(date) = NaiveDate::parse_from_str(value, "%Y-%m-%d") {
        return Some(DatePoint::Full(date));
    }
    if value.len() == 7 {
        let (year, month) = value.split_once('-')?;
        return DatePoint::year_month(year.parse().ok()?, month.parse().ok()?);
    }
    if let Ok(year) = value.parse() {
        return Some(DatePoint::Year(year));
    }
    for format in ["%b %Y", "%B %Y"] {
        if let Ok(date) = NaiveDate::parse_from_str(&format!("1 {value}"), &format!("%d {format}"))
        {
            return DatePoint::year_month(
                date.format("%Y").to_string().parse().ok()?,
                date.format("%m").to_string().parse().ok()?,
            );
        }
    }
    None
}

pub const STARTER_MARKDOWN: &str = include_str!("starter.md");

pub fn to_markdown(cv: &CvDocument) -> String {
    use std::fmt::Write;

    let mut output = format!("# {}\n\n", cv.profile.name);
    let contacts: Vec<String> = [
        cv.profile.email.clone(),
        cv.profile.phone.clone(),
        cv.profile.location.clone(),
        cv.profile.website.as_ref().map(ToString::to_string),
    ]
    .into_iter()
    .flatten()
    .collect();
    if !contacts.is_empty() {
        let _ = writeln!(output, "{}\n", contacts.join(" · "));
    }
    if let Some(summary) = &cv.profile.summary {
        let _ = writeln!(output, "{}\n", summary.to_markdown());
    }
    for section in &cv.sections {
        let _ = writeln!(output, "## {}\n", section.title);
        write_tags(&section.tags, &mut output);
        for entry in &section.entries {
            let (primary, secondary) = entry.kind.heading();
            if matches!(entry.kind, EntryKind::Text(_)) {
                let _ = writeln!(output, "- {}\n", primary.to_markdown());
                write_tags(&entry.tags, &mut output);
                continue;
            }
            if let EntryKind::SkillGroup(value) = &entry.kind
                && value.skills.is_empty()
            {
                let _ = writeln!(output, "- {}\n", value.name.to_markdown());
                write_tags(&entry.tags, &mut output);
                continue;
            }
            let heading = secondary
                .filter(|value| !value.is_empty())
                .map(|value| format!("{}, {}", primary.to_markdown(), value.to_markdown()))
                .unwrap_or_else(|| primary.to_markdown());
            let _ = writeln!(output, "### {heading}");
            if let Some(period) = entry.kind.period() {
                let _ = writeln!(output, "{}–{}", period.start, period.end);
            } else if let Some(date) = entry.kind.date() {
                let _ = writeln!(output, "{date}");
            }
            write_tags(&entry.tags, &mut output);
            if !entry.kind.highlights().is_empty() {
                output.push('\n');
                for highlight in entry.kind.highlights() {
                    let _ = writeln!(output, "- {}", highlight.to_markdown());
                }
            }
            output.push('\n');
        }
    }
    output
}

fn write_tags(tags: &TagSet, output: &mut String) {
    use std::fmt::Write;
    if !tags.is_empty() {
        let _ = writeln!(
            output,
            "<!-- tags: {} -->",
            tags.iter().cloned().collect::<Vec<_>>().join(", ")
        );
    }
}
