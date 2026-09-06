mod fields;
mod write;

use std::collections::{BTreeMap, BTreeSet};

use cac_core::{
    Contact, CvDocument, Entry, EntryKind, Inline, Origin, Profile, RichText, Section, SectionKind,
    TagSet, TextEntry, supported_link,
};
use serde_json::{Map, Value};
use url::Url;

use crate::ParseError;
pub use fields::{parse_date_point, slugify};
use fields::{parse_period, parse_section_annotation, parse_tags};
pub(crate) use write::difference;
pub use write::{to_markdown, to_markdown_checked};

pub const STARTER_MARKDOWN: &str = include_str!("../starter.md");

fn markdown_error<T>(line: usize, message: &str) -> Result<T, ParseError> {
    Err(ParseError::Markdown {
        line,
        message: message.into(),
    })
}

fn rich(value: &str, line: usize) -> Result<RichText, ParseError> {
    RichText::try_parse(value).map_err(|error| {
        let prefix = &value[..error.offset.min(value.len())];
        let offset = prefix.bytes().filter(|byte| *byte == b'\n').count();
        let column = prefix.rsplit('\n').next().unwrap_or("").chars().count() + 1;
        ParseError::Markdown {
            line: line + offset,
            message: format!("column {column}: {}", error.message),
        }
    })
}

fn inline(value: &str, line: usize) -> Result<RichText, ParseError> {
    let value = rich(value, line)?;
    if !value.is_inline() {
        return markdown_error(
            line,
            "this field accepts inline formatting, not paragraphs or lists",
        );
    }
    Ok(value)
}

fn heading(value: &str, line: usize) -> Result<String, ParseError> {
    let value = inline(value.trim(), line)?;
    if value.is_empty() {
        return markdown_error(line, "heading must contain visible text");
    }
    Ok(value.plain())
}

#[derive(Default)]
struct Content {
    source: String,
    line: usize,
}

impl Content {
    fn push(&mut self, raw: &str, line: usize) {
        if self.source.is_empty() {
            self.line = line;
        }
        self.source.push_str(raw);
        self.source.push('\n');
    }

    fn take(&mut self) -> Result<Option<RichText>, ParseError> {
        let source = std::mem::take(&mut self.source);
        if source.trim().is_empty() {
            Ok(None)
        } else {
            let value = rich(source.trim_end(), self.line)?;
            Ok((!value.is_empty()).then_some(value))
        }
    }
}

#[derive(Default)]
struct Metadata {
    fields: BTreeMap<String, (String, usize)>,
    explicit: bool,
}

impl Metadata {
    fn insert(&mut self, key: &str, value: &str, line: usize) -> Result<(), ParseError> {
        if let Some((_, previous)) = self.fields.get(key) {
            return markdown_error(
                line,
                &format!(
                    "duplicate `{key}` field; first supplied at line {previous}; keep one value"
                ),
            );
        }
        self.fields.insert(key.into(), (value.into(), line));
        Ok(())
    }

    fn value(&self, key: &str) -> Option<&str> {
        self.fields.get(key).map(|(value, _)| value.as_str())
    }
}

struct EntryBuilder {
    heading: String,
    line: usize,
    metadata: Metadata,
    content: Content,
    tags: TagSet,
}

impl EntryBuilder {
    fn build(mut self, section_kind: &SectionKind) -> Result<Entry, ParseError> {
        let kind = self
            .metadata
            .value("kind")
            .filter(|value| !value.is_empty())
            .unwrap_or(default_entry_kind(section_kind))
            .to_owned();
        let mut object = Map::new();
        object.insert("type".into(), Value::String(kind.clone()));
        let (primary, secondary) = match kind.as_str() {
            "experience" => ("role", Some("organization")),
            "education" => ("qualification", Some("institution")),
            "project" | "skill-group" => ("name", None),
            "publication" => ("title", Some("publisher")),
            "custom" => ("heading", None),
            "text" | "prose" => ("body", None),
            _ => {
                return markdown_error(
                    self.line,
                    "invalid entry Kind; use experience, education, project, publication, skill-group, custom, text, or prose",
                );
            }
        };
        let mut title = self.heading.as_str();
        if !self.metadata.explicit
            && matches!(kind.as_str(), "experience" | "education")
            && let Some((first, second)) = title.split_once(", ")
        {
            title = first;
            object.insert(
                secondary.unwrap().into(),
                serde_json::to_value(inline(second, self.line)?).unwrap(),
            );
        }
        let title = inline(title, self.line)?;
        if title.is_empty() {
            return markdown_error(self.line, "entry heading must contain visible text");
        }
        object.insert(primary.into(), serde_json::to_value(title).unwrap());
        for (key, (value, line)) in &self.metadata.fields {
            if key == "kind" {
                continue;
            }
            let allowed = match key.as_str() {
                "organization" => kind == "experience",
                "institution" => kind == "education",
                "publisher" => kind == "publication",
                "location" => kind == "experience",
                "url" => matches!(kind.as_str(), "project" | "publication"),
                "period" => matches!(
                    kind.as_str(),
                    "experience" | "education" | "project" | "custom"
                ),
                "date" => matches!(
                    kind.as_str(),
                    "experience" | "education" | "project" | "custom" | "publication"
                ),
                _ => false,
            };
            if !allowed {
                return markdown_error(
                    *line,
                    &format!(
                        "`{key}` is not supported by `{kind}`; choose an appropriate Kind or put the information in prose or a bullet"
                    ),
                );
            }
            if value.is_empty() {
                continue;
            }
            let value = match key.as_str() {
                "period" => serde_json::to_value(parse_period(value).ok_or_else(|| ParseError::Markdown { line: *line, message: "Period needs a range such as 2022–Present, 2022–, or –2024; use Date for a single date".into() })?.map_err(|message| ParseError::Markdown { line: *line, message })?).unwrap(),
                "date" => serde_json::to_value(parse_date_point(value).filter(|date| !matches!(date, cac_core::DatePoint::Present)).ok_or_else(|| ParseError::Markdown { line: *line, message: "invalid Date; use YYYY, YYYY-MM, YYYY-MM-DD, or an English month and year; omit Date if unknown".into() })?).unwrap(),
                "url" => Value::String(parse_url(value, *line)?.to_string()),
                "location" => Value::String(inline(value, *line)?.plain()),
                _ => serde_json::to_value(inline(value, *line)?).unwrap(),
            };
            object.insert(key.clone(), value);
        }
        if object.contains_key("date") && object.contains_key("period") {
            return markdown_error(self.line, "Date and Period conflict; supply only one");
        }
        let mut body = self.content.take()?;
        if let Some(content) = &body
            && let [Inline::List { start: None, items }] = content.0.as_slice()
            && !matches!(kind.as_str(), "text" | "prose")
        {
            object.insert(
                if kind == "skill-group" {
                    "skills"
                } else {
                    "highlights"
                }
                .into(),
                serde_json::to_value(items).unwrap(),
            );
            body = None;
        }
        if matches!(kind.as_str(), "text" | "prose") && body.is_some() {
            return markdown_error(
                self.line,
                "text/prose heading entries cannot have a second body; write standalone prose or use Kind: custom",
            );
        }
        let kind = serde_json::from_value(Value::Object(object)).map_err(|error| {
            ParseError::Markdown {
                line: self.line,
                message: error.to_string(),
            }
        })?;
        Ok(Entry {
            kind,
            content: body,
            tags: self.tags,
            origin: Origin {
                path: format!("line {}", self.line),
            },
        })
    }
}

fn default_entry_kind(kind: &SectionKind) -> &'static str {
    match kind {
        SectionKind::Experience => "experience",
        SectionKind::Education => "education",
        SectionKind::Projects => "project",
        SectionKind::Publications => "publication",
        SectionKind::Skills => "skill-group",
        SectionKind::Custom | SectionKind::Named(_) => "custom",
    }
}

fn section_kind(title: &str) -> SectionKind {
    match title.to_ascii_lowercase().as_str() {
        "experience"
        | "work experience"
        | "professional experience"
        | "employment"
        | "employment history" => SectionKind::Experience,
        "education" => SectionKind::Education,
        "project" | "projects" => SectionKind::Projects,
        "publication" | "publications" => SectionKind::Publications,
        "skill" | "skills" | "technical skills" => SectionKind::Skills,
        _ => SectionKind::Custom,
    }
}

fn metadata_field(line: &str) -> Option<(&str, &str)> {
    let (key, value) = line.split_once(':')?;
    if key.is_empty() || !key.chars().all(|c| c.is_ascii_alphabetic()) || value.starts_with("//") {
        return None;
    }
    Some((key, value.trim()))
}

fn parse_url(value: &str, line: usize) -> Result<Url, ParseError> {
    let value = inline(value, line)?.plain();
    Url::parse(&value)
        .ok()
        .filter(supported_link)
        .ok_or_else(|| ParseError::Markdown {
            line,
            message: "invalid URL; use an absolute https:, http:, mailto:, or tel: URL".into(),
        })
}

fn profile_field(
    profile: &mut Profile,
    fields: &mut Metadata,
    key: &str,
    value: &str,
    line: usize,
) -> Result<(), ParseError> {
    let key = key.to_ascii_lowercase();
    if key == "contact" {
        let rich = inline(value, line)?;
        let (label, href) = match rich.0.as_slice() {
            [Inline::Link { href, body }] => (RichText(body.clone()).plain(), Some(href.clone())),
            _ => (rich.plain(), None),
        };
        if label.trim().is_empty() {
            return markdown_error(line, "Contact requires a label or a labeled link");
        }
        profile.contacts.push(Contact { label, href });
        return Ok(());
    }
    if !["email", "phone", "location", "website"].contains(&key.as_str()) {
        return markdown_error(
            line,
            &format!(
                "unknown profile field `{key}`; use Email, Phone, Location, Website, or Contact; insert a blank line before ordinary prose"
            ),
        );
    }
    fields.insert(&key, value, line)?;
    if value.is_empty() {
        return Ok(());
    }
    let text = inline(value, line)?.plain();
    match key.as_str() {
        "email" => profile.email = Some(text),
        "phone" => profile.phone = Some(text),
        "location" => profile.location = Some(text),
        "website" => profile.website = Some(parse_url(value, line)?),
        _ => unreachable!(),
    }
    Ok(())
}

fn legacy_contacts(
    profile: &mut Profile,
    fields: &mut Metadata,
    line: &str,
    number: usize,
) -> Result<bool, ParseError> {
    let parts: Vec<_> = line
        .split('·')
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .collect();
    let classify = |part: &str| {
        if part
            .split_once('@')
            .is_some_and(|(left, right)| !left.is_empty() && !right.is_empty())
            && !part.chars().any(char::is_whitespace)
            && !part.contains(['[', ']', '<', '>', '\\'])
        {
            "email"
        } else if !part.chars().any(char::is_whitespace)
            && Url::parse(part).is_ok_and(|url| matches!(url.scheme(), "https" | "http"))
        {
            "website"
        } else if part.chars().filter(char::is_ascii_digit).count() >= 5
            && part
                .chars()
                .all(|c| c.is_ascii_digit() || " +()-./".contains(c))
        {
            "phone"
        } else {
            "location"
        }
    };
    if !parts.iter().any(|part| classify(part) != "location") {
        return Ok(false);
    }
    if parts.len() == 1 && classify(parts[0]) == "location" {
        return Ok(false);
    }
    for part in parts {
        let kind = classify(part);
        profile_field(
            profile,
            fields,
            kind,
            part.trim_start_matches("mailto:"),
            number,
        )?;
    }
    Ok(true)
}

fn finish_entry(
    section: &mut Option<Section>,
    entry: &mut Option<EntryBuilder>,
) -> Result<(), ParseError> {
    if let Some(entry) = entry.take() {
        let section = section.as_mut().expect("entry belongs to a section");
        section.entries.push(entry.build(&section.kind)?);
    }
    Ok(())
}

fn flush_direct(
    section: &mut Option<Section>,
    content: &mut Content,
    override_kind: &mut Option<String>,
) -> Result<(), ParseError> {
    let line = content.line;
    let Some(value) = content.take()? else {
        if override_kind.is_some() {
            return markdown_error(line, "cac:entry must be followed by nonempty content");
        }
        return Ok(());
    };
    let section = section
        .as_mut()
        .expect("direct content belongs to a section");
    let forced = override_kind.take();
    let entries = if forced.is_none() {
        if let [Inline::List { start: None, items }] = value.0.as_slice()
            && items.iter().all(RichText::is_inline)
        {
            items
                .iter()
                .map(|item| {
                    if section.kind == SectionKind::Skills {
                        EntryKind::SkillGroup(cac_core::SkillGroupEntry {
                            name: item.clone(),
                            skills: Vec::new(),
                        })
                    } else {
                        EntryKind::Text(TextEntry { body: item.clone() })
                    }
                })
                .collect()
        } else {
            vec![EntryKind::Prose(TextEntry { body: value })]
        }
    } else if forced.as_deref() == Some("text") {
        vec![EntryKind::Text(TextEntry { body: value })]
    } else {
        vec![EntryKind::Prose(TextEntry { body: value })]
    };
    section
        .entries
        .extend(entries.into_iter().map(|kind| Entry {
            kind,
            content: None,
            tags: TagSet::new(),
            origin: Origin {
                path: format!("line {line}"),
            },
        }));
    Ok(())
}

pub fn parse_markdown(source: &str) -> Result<CvDocument, ParseError> {
    let mut profile = Profile::default();
    let mut profile_fields = Metadata::default();
    let mut section_fields = Metadata::default();
    let mut summary = Content::default();
    let mut sections = Vec::new();
    let mut section: Option<Section> = None;
    let mut entry: Option<EntryBuilder> = None;
    let mut direct = Content::default();
    let mut direct_kind = None;
    let mut last_direct_entry: Option<usize> = None;
    let mut metadata_open = false;
    let mut annotation_allowed = false;
    let mut saw_name = false;
    let mut section_ids: Vec<Option<usize>> = Vec::new();
    let mut in_comment = false;
    let mut summary_started = false;
    for (index, raw) in source.lines().enumerate() {
        let number = index + 1;
        let line = raw.trim();
        if in_comment {
            if line.ends_with("-->") {
                in_comment = false;
            }
            continue;
        }
        if line.starts_with("<!-- tags:") && parse_tags(line).is_none() {
            return markdown_error(
                number,
                "invalid tags comment; use a comma-separated list or a JSON array inside <!-- tags: ... -->",
            );
        }
        if line.starts_with("<!--") && !line.starts_with("<!-- cac:") && parse_tags(line).is_none()
        {
            if !line.ends_with("-->") {
                in_comment = true;
            }
            continue;
        }
        if line.is_empty() {
            metadata_open = false;
            if let Some(entry) = &mut entry {
                entry.content.push("", number);
            } else if section.is_some() {
                direct.push("", number);
            } else if saw_name {
                summary.push("", number);
            }
            continue;
        }
        if line.starts_with("<!-- cac:section") {
            if !annotation_allowed {
                return markdown_error(
                    number,
                    "section annotation must immediately follow a section heading and may appear only once",
                );
            }
            parse_section_annotation(line, number, section.as_mut().unwrap())?;
            for key in ["id", "kind"] {
                if line
                    .split_whitespace()
                    .any(|part| part.starts_with(&format!("{key}=")))
                {
                    section_fields.insert(key, "", number)?;
                }
            }
            if line.split_whitespace().any(|part| part.starts_with("id=")) {
                *section_ids.last_mut().unwrap() = Some(number);
            }
            annotation_allowed = false;
            continue;
        }
        annotation_allowed = false;
        let indent = raw
            .chars()
            .take_while(|c| matches!(c, ' ' | '\t'))
            .fold(0, |column, c| {
                if c == '\t' {
                    column + 4 - column % 4
                } else {
                    column + 1
                }
            });
        let header = if indent < 4 {
            line.split_once(' ')
                .filter(|(prefix, _)| matches!(*prefix, "#" | "##" | "###"))
        } else {
            None
        };
        if matches!(line, "#" | "##" | "###") {
            return markdown_error(number, "heading must contain visible text");
        }
        if let Some((level, value)) = header {
            match level {
                "#" => {
                    if saw_name {
                        return markdown_error(
                            number,
                            "the document must contain exactly one level-one name heading",
                        );
                    }
                    profile.name = heading(value, number)?;
                    saw_name = true;
                    metadata_open = true;
                }
                "##" => {
                    if !saw_name {
                        return markdown_error(
                            number,
                            "content must start with a level-one name heading",
                        );
                    }
                    finish_entry(&mut section, &mut entry)?;
                    if section.is_some() {
                        flush_direct(&mut section, &mut direct, &mut direct_kind)?;
                    }
                    if let Some(previous) = section.take() {
                        sections.push(previous);
                    }
                    let title = heading(value, number)?;
                    section = Some(Section {
                        id: slugify(&title),
                        kind: section_kind(&title),
                        title,
                        entries: Vec::new(),
                        tags: TagSet::new(),
                    });
                    section_ids.push(None);
                    last_direct_entry = None;
                    annotation_allowed = true;
                    metadata_open = true;
                    section_fields = Metadata::default();
                }
                "###" => {
                    if section.is_none() {
                        return markdown_error(
                            number,
                            "an entry heading must be inside a level-two section",
                        );
                    }
                    finish_entry(&mut section, &mut entry)?;
                    flush_direct(&mut section, &mut direct, &mut direct_kind)?;
                    last_direct_entry = None;
                    entry = Some(EntryBuilder {
                        heading: value.into(),
                        line: number,
                        metadata: Metadata::default(),
                        content: Content::default(),
                        tags: TagSet::new(),
                    });
                    metadata_open = true;
                }
                _ => unreachable!(),
            }
            continue;
        }
        if !saw_name {
            return markdown_error(number, "content must start with a level-one name heading");
        }
        if let Some(tags) = parse_tags(line) {
            if let Some(entry) = &mut entry {
                entry.tags.extend(tags);
            } else if section.is_some() {
                let has_content = !direct.source.trim().is_empty();
                if has_content {
                    flush_direct(&mut section, &mut direct, &mut direct_kind)?;
                }
                let section = section.as_mut().unwrap();
                if has_content {
                    last_direct_entry = Some(section.entries.len() - 1);
                }
                if let Some(index) = last_direct_entry {
                    section.entries[index].tags.extend(tags);
                } else {
                    section.tags.extend(tags);
                }
            } else {
                return markdown_error(number, "tags must follow a section or entry");
            }
            continue;
        }
        if line.starts_with("<!-- cac:entry") {
            if section.is_none() {
                return markdown_error(number, "cac:entry must be inside a section");
            }
            finish_entry(&mut section, &mut entry)?;
            flush_direct(&mut section, &mut direct, &mut direct_kind)?;
            last_direct_entry = None;
            direct_kind = match line {
                "<!-- cac:entry kind=prose -->" => Some("prose".into()),
                "<!-- cac:entry kind=text -->" => Some("text".into()),
                _ => {
                    return markdown_error(
                        number,
                        "expected <!-- cac:entry kind=prose --> or <!-- cac:entry kind=text -->",
                    );
                }
            };
            metadata_open = false;
            direct.line = number;
            continue;
        }
        if line == "<!-- cac:summary -->" {
            if section.is_some() || summary_started {
                return markdown_error(
                    number,
                    "cac:summary must appear once before summary content and sections",
                );
            }
            summary_started = true;
            metadata_open = false;
            continue;
        }
        if line.starts_with("<!-- cac:") {
            return markdown_error(
                number,
                "unknown cac directive; expected cac:section or cac:entry",
            );
        }
        if metadata_open && let Some((key, value)) = metadata_field(line) {
            if let Some(entry) = &mut entry {
                let key = key.to_ascii_lowercase();
                if ![
                    "kind",
                    "organization",
                    "institution",
                    "publisher",
                    "location",
                    "url",
                    "period",
                    "date",
                ]
                .contains(&key.as_str())
                {
                    return markdown_error(
                        number,
                        &format!(
                            "unknown entry field `{key}`; use Kind, Organization, Institution, Publisher, Location, URL, Period, or Date; insert a blank line before ordinary prose"
                        ),
                    );
                }
                entry.metadata.explicit = true;
                entry.metadata.insert(&key, value, number)?;
            } else if let Some(section) = &mut section {
                let key = key.to_ascii_lowercase();
                if !matches!(key.as_str(), "kind" | "id") {
                    return markdown_error(
                        number,
                        "unknown section field; use Kind or Id; insert a blank line before ordinary prose",
                    );
                }
                section_fields.insert(&key, value, number)?;
                if !value.is_empty() {
                    if key == "kind" {
                        section.kind = serde_json::from_value(Value::String(value.into()))
                            .map_err(|error| ParseError::Markdown {
                                line: number,
                                message: error.to_string(),
                            })?;
                    } else {
                        section.id = heading(value, number)?;
                        *section_ids.last_mut().unwrap() = Some(number);
                    }
                }
            } else {
                profile_field(&mut profile, &mut profile_fields, key, value, number)?;
            }
            continue;
        }
        metadata_open = false;
        if section.is_none() {
            if !summary_started
                && summary.source.trim().is_empty()
                && legacy_contacts(&mut profile, &mut profile_fields, line, number)?
            {
                continue;
            }
            summary_started = true;
            summary.push(raw, number);
            continue;
        }
        if let Some(entry) = &mut entry {
            if entry.content.source.trim().is_empty() {
                let range_like = line.chars().next().is_some_and(|c| c.is_ascii_digit())
                    || line.starts_with('–')
                    || parse_date_point(line.split('–').next().unwrap_or("").trim()).is_some();
                if range_like && parse_period(line).is_some() {
                    entry.metadata.insert("period", line, number)?;
                    continue;
                }
                if parse_date_point(line).is_some() {
                    entry.metadata.insert("date", line, number)?;
                    continue;
                }
            }
            entry.content.push(raw, number);
        } else {
            direct.push(raw, number);
        }
    }
    if in_comment {
        return markdown_error(source.lines().count(), "unterminated HTML comment; add -->");
    }
    if !saw_name {
        return markdown_error(1, "the document is missing a level-one name heading");
    }
    finish_entry(&mut section, &mut entry)?;
    if section.is_some() {
        flush_direct(&mut section, &mut direct, &mut direct_kind)?;
    }
    if let Some(section) = section {
        sections.push(section);
    }
    profile.summary = summary.take()?;
    let mut reserved = BTreeMap::new();
    for (section, explicit) in sections.iter().zip(&section_ids) {
        if let Some(line) = explicit
            && let Some(previous) = reserved.insert(section.id.clone(), *line)
        {
            return markdown_error(
                *line,
                &format!(
                    "duplicate section id `{}`; first supplied at line {previous}",
                    section.id
                ),
            );
        }
    }
    let mut used: BTreeSet<_> = reserved.into_keys().collect();
    for (section, explicit) in sections.iter_mut().zip(section_ids) {
        if explicit.is_some() {
            continue;
        }
        let base = if section.id.is_empty() {
            "section".to_owned()
        } else {
            section.id.clone()
        };
        let mut id = base.clone();
        let mut suffix = 2;
        while used.contains(&id) {
            id = format!("{base}-{suffix}");
            suffix += 1;
        }
        used.insert(id.clone());
        section.id = id;
    }
    let mut cv = CvDocument { profile, sections };
    crate::codec::normalize(&mut cv);
    crate::validate(&cv)?;
    Ok(cv)
}
