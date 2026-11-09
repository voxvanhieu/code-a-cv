//! Phase two of the FieldMark parser: mapping structure onto the model.
//!
//! The rules live in `docs/authoring/markdown.md`. The short version: a
//! paragraph after a heading is a field block when every line is a field, a
//! valid date or period, or a contact line, and at least one key is known —
//! blank lines before, inside, or after it mean nothing. Undeclared keys in
//! an entry field block become open fields. Heading sugar fills the subtitle
//! per field, never per entry: explicit fields win, sugar fills gaps.

use std::collections::BTreeMap;
use std::collections::BTreeSet;

use super::ast::{Block, BlockKind};
use super::fields::{
    parse_date_point, parse_period, parse_section_annotation, parse_tags, slugify,
};
use super::markdown_error;
use cac_core::{
    Contact, CvDocument, DatePoint, Entry, EntryKind, FieldMap, FieldValue, Inline, Origin,
    Profile, RichText, Section, SectionKind, TagSet, TextEntry, core_field, normalize_key,
    supported_link,
};
use serde_json::{Map, Value};
use url::Url;

use crate::ParseError;

pub(super) struct Mapped {
    pub profile: Profile,
    pub sections: Vec<Section>,
}

/// What the mapping learned about the source beyond the model: whether the
/// compact sugar was used (so `preserve` formatting keeps it), and how many
/// plain comments the model dropped (so `fmt` can refuse to lose them).
#[derive(Default)]
pub(super) struct MapInfo {
    pub sugar: bool,
    pub plain_comments: usize,
}

/// A `Key: value` line inside a field block.
struct FieldLine {
    key: String,
    value: String,
    line: usize,
}

/// One classified line of a field block.
enum FieldBlockLine {
    Field(FieldLine),
    PeriodLine { value: String, line: usize },
    DateLine { value: String, line: usize },
}

enum LineClass {
    Field(String, String),
    Period,
    Date,
    NotField,
}

fn classify_line(line: &str) -> LineClass {
    if let Some(period) = parse_period(line) {
        // A period line must fully validate; prose that merely starts with
        // a number never becomes a date (the false-date rule).
        if period.is_ok() {
            return LineClass::Period;
        }
    }
    if parse_date_point(line).is_some() {
        return LineClass::Date;
    }
    if let Some((key, value)) = metadata_field(line) {
        return LineClass::Field(key.to_ascii_lowercase(), value.to_string());
    }
    LineClass::NotField
}

/// Rule F: the paragraph is a field block when every non-empty line is a
/// field, a valid date or period, or — under the name only — a legacy
/// contact line, and at least one key is known in the given context.
/// Returns `None` when the paragraph is prose.
fn field_block(
    raw: &str,
    first_line: usize,
    known: impl Fn(&str) -> bool,
    contacts: bool,
) -> Option<Vec<FieldBlockLine>> {
    let mut lines = Vec::new();
    let mut any_known = false;
    for (offset, raw_line) in raw.split('\n').enumerate() {
        let raw_line = raw_line.trim();
        if raw_line.is_empty() {
            continue;
        }
        let number = first_line + offset;
        match classify_line(raw_line) {
            LineClass::Field(key, value) => {
                if known(&key) {
                    any_known = true;
                }
                lines.push(FieldBlockLine::Field(FieldLine {
                    key,
                    value,
                    line: number,
                }));
            }
            LineClass::Period => {
                lines.push(FieldBlockLine::PeriodLine {
                    value: raw_line.to_string(),
                    line: number,
                });
                any_known = true;
            }
            LineClass::Date => {
                lines.push(FieldBlockLine::DateLine {
                    value: raw_line.to_string(),
                    line: number,
                });
                any_known = true;
            }
            LineClass::NotField => {
                if contacts && is_contact_line(raw_line) {
                    any_known = true;
                    continue;
                }
                return None;
            }
        }
    }
    if lines.is_empty() || !any_known {
        return None;
    }
    Some(lines)
}

fn is_contact_line(line: &str) -> bool {
    if line.contains('·') {
        return true;
    }
    let plain = line
        .trim_start_matches("mailto:")
        .trim_start_matches("tel:");
    if plain
        .split_once('@')
        .is_some_and(|(left, right)| !left.is_empty() && !right.is_empty())
        && !plain.chars().any(char::is_whitespace)
        && !plain.contains(['[', ']', '<', '>', '\\'])
    {
        return true;
    }
    Url::parse(plain).is_ok_and(|url| matches!(url.scheme(), "https" | "http"))
}

fn metadata_field(line: &str) -> Option<(&str, &str)> {
    let (key, value) = line.split_once(':')?;
    let valid_key = !key.is_empty()
        && key
            .chars()
            .all(|c| c.is_alphanumeric() || matches!(c, ' ' | '_' | '-'))
        && key.chars().next().is_some_and(|c| c.is_alphabetic());
    if !valid_key || value.starts_with("//") {
        return None;
    }
    // Trim the key here, once: downstream scope checks and slot matching
    // must see the same spelling, or a known key with a stray space falls
    // out of the slot table and panics.
    Some((key.trim(), value.trim()))
}

/// An entry under construction; fields land per slot or stay open.
#[derive(Default)]
struct EntryBuilder {
    heading: String,
    line: usize,
    kind: Option<String>,
    organization: Option<(String, usize)>,
    institution: Option<(String, usize)>,
    publisher: Option<(String, usize)>,
    location: Option<(String, usize)>,
    url: Option<(String, usize)>,
    period: Option<(String, usize)>,
    date: Option<(String, usize)>,
    tags: TagSet,
    fields: FieldMap,
    /// Payload keys for language/reference entries (proficiency, role,
    /// email, phone, note) plus education's score.
    payload: Vec<(String, String, usize)>,
    content: String,
    content_line: usize,
    /// Heading sugar (`### Role, Organization`) was consumed; `preserve`
    /// formatting must keep the compact spelling.
    sugar_heading: bool,
}

impl EntryBuilder {
    fn apply(&mut self, key: &str, value: &str, line: usize) -> Result<(), ParseError> {
        let slot = match key {
            "kind" => return self.set_kind(value),
            "organization" => &mut self.organization,
            "institution" => &mut self.institution,
            "publisher" => &mut self.publisher,
            "location" => &mut self.location,
            "url" => &mut self.url,
            "period" => &mut self.period,
            "date" => &mut self.date,
            "score" | "proficiency" | "role" | "email" | "phone" | "note" => {
                self.payload
                    .push((key.to_string(), value.to_string(), line));
                return Ok(());
            }
            // The heading carries the primary slot. A field line repeating
            // it is redundant spelling; a differing one is a duplicate, and
            // neither may silently become an open field that shadows the
            // slot.
            "name" | "qualification" | "title" | "heading" => {
                let spelled = inline_plain(value, line)?;
                let current = super::heading(&self.heading, self.line)?;
                if spelled == current {
                    return Ok(());
                }
                return markdown_error(
                    line,
                    &format!(
                        "duplicate `{key}`; the heading is the {key} — drop the line or change the heading"
                    ),
                );
            }
            "skills" => {
                return markdown_error(
                    line,
                    "`Skills` are the bullets under the heading; write each skill as a list item",
                );
            }
            "highlights" => {
                return markdown_error(
                    line,
                    "`Highlights` are the bullets under the heading; write each highlight as a list item",
                );
            }
            other => {
                let normalized = normalize_key(other);
                if self.fields.get(&normalized).is_some() {
                    return markdown_error(
                        line,
                        &format!("duplicate `{other}` field; keep one value"),
                    );
                }
                self.fields
                    .insert(normalized, FieldValue::Text(value.to_string()));
                return Ok(());
            }
        };
        if slot.is_some() {
            return self.duplicate(key, line);
        }
        *slot = Some((value.to_string(), line));
        Ok(())
    }

    fn set_kind(&mut self, value: &str) -> Result<(), ParseError> {
        if self
            .kind
            .as_deref()
            .is_some_and(|existing| !existing.is_empty())
        {
            return markdown_error(self.line, "duplicate `Kind`; keep one kind per entry");
        }
        self.kind = Some(value.to_string());
        Ok(())
    }

    fn duplicate(&self, key: &str, line: usize) -> Result<(), ParseError> {
        markdown_error(line, &format!("duplicate `{key}` field; keep one value"))
    }

    fn push_content(&mut self, raw: &str, line: usize) {
        if self.content.is_empty() {
            self.content_line = line;
        }
        // Keep paragraph boundaries between blocks: two prose paragraphs
        // separated by a blank line must not merge into one.
        if !self.content.ends_with("\n\n") && !self.content.is_empty() {
            self.content.push('\n');
        }
        self.content.push_str(raw);
        self.content.push('\n');
    }

    fn build(
        mut self,
        section_kind: &SectionKind,
        info: &mut MapInfo,
    ) -> Result<Entry, ParseError> {
        let kind = self
            .kind
            .as_deref()
            .filter(|value| !value.is_empty())
            .unwrap_or(super::default_entry_kind(section_kind))
            .to_owned();
        if self.sugar_heading {
            info.sugar = true;
        }
        let mut object = Map::new();
        object.insert("type".into(), Value::String(kind.clone()));
        let (primary, secondary) = match kind.as_str() {
            "experience" => ("role", Some("organization")),
            "education" => ("qualification", Some("institution")),
            "project" | "skill-group" | "language" | "reference" => ("name", None),
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
        // Rule G: sugar fills the subtitle only when the explicit slot is
        // empty, and the same rule applies to every entry in the file.
        let explicit_subtitle = match kind.as_str() {
            "experience" => self
                .organization
                .as_ref()
                .is_some_and(|(value, _)| !value.is_empty()),
            "education" => self
                .institution
                .as_ref()
                .is_some_and(|(value, _)| !value.is_empty()),
            _ => true,
        };
        let mut title = self.heading.as_str();
        if !explicit_subtitle
            && matches!(kind.as_str(), "experience" | "education")
            && let Some((first, second)) = title.split_once(", ")
        {
            title = first;
            self.sugar_heading = true;
            let filled = inline(second, self.line)?;
            object.insert(
                secondary.expect("sugar only where a slot exists").into(),
                serde_json::to_value(filled).unwrap(),
            );
        }
        let title = inline(title, self.line)?;
        if title.is_empty() {
            return markdown_error(self.line, "entry heading must contain visible text");
        }
        object.insert(primary.into(), serde_json::to_value(title).unwrap());
        type EntrySlot<'a> = (&'a str, Option<(String, usize)>, &'a dyn Fn(&str) -> bool);
        let mut slots: Vec<EntrySlot<'_>> = Vec::new();
        slots.push(("organization", self.organization.take(), &|kind: &str| {
            matches!(kind, "experience" | "reference")
        }));
        slots.push(("institution", self.institution.take(), &|kind: &str| {
            kind == "education"
        }));
        slots.push(("publisher", self.publisher.take(), &|kind: &str| {
            kind == "publication"
        }));
        slots.push(("location", self.location.take(), &|kind: &str| {
            kind == "experience"
        }));
        slots.push(("url", self.url.take(), &|kind: &str| {
            matches!(kind, "project" | "publication")
        }));
        slots.push(("period", self.period.take(), &|kind: &str| {
            matches!(kind, "experience" | "education" | "project" | "custom")
        }));
        slots.push(("date", self.date.take(), &|kind: &str| {
            matches!(
                kind,
                "experience" | "education" | "project" | "custom" | "publication"
            )
        }));
        let mut period_applied = false;
        let mut date_applied = false;
        for (key, slot, allowed) in slots {
            let Some((value, line)) = slot else { continue };
            if !allowed(&kind) {
                return markdown_error(
                    line,
                    &format!(
                        "`{key}` is not supported by `{kind}`; choose an appropriate Kind or put the information in prose or a bullet"
                    ),
                );
            }
            if value.is_empty() {
                continue;
            }
            let value = match key {
                "period" => {
                    let period = parse_period(&value)
                        .ok_or_else(|| ParseError::Markdown {
                            line,
                            message: "Period needs a range such as 2022–Present, 2022–, or –2024; use Date for a single date".into(),
                        })?
                        .map_err(|message| ParseError::Markdown { line, message })?;
                    period_applied = true;
                    serde_json::to_value(period).unwrap()
                }
                "date" => {
                    let date = parse_date_point(&value)
                        .filter(|date| !matches!(date, DatePoint::Present))
                        .ok_or_else(|| ParseError::Markdown {
                            line,
                            message: "invalid Date; use YYYY, YYYY-MM, YYYY-MM-DD, or an English month and year; omit Date if unknown".into(),
                        })?;
                    date_applied = true;
                    serde_json::to_value(date).unwrap()
                }
                "url" => Value::String(parse_url(&value, line)?.to_string()),
                "location" => Value::String(inline(&value, line)?.plain()),
                _ => serde_json::to_value(inline(&value, line)?).unwrap(),
            };
            object.insert(key.into(), value);
        }
        if period_applied && date_applied {
            return markdown_error(self.line, "Date and Period conflict; supply only one");
        }
        for (key, value, line) in std::mem::take(&mut self.payload) {
            let allowed = match key.as_str() {
                "score" => kind == "education",
                "proficiency" => kind == "language",
                "role" | "email" | "phone" | "note" => kind == "reference",
                _ => false,
            };
            if !allowed {
                return markdown_error(
                    line,
                    &format!(
                        "`{key}` is not supported by `{kind}`; choose an appropriate Kind or put the information in prose or a bullet"
                    ),
                );
            }
            if value.is_empty() {
                continue;
            }
            let value = if key == "score" || key == "proficiency" {
                // SkillLevel accepts named scales and 0–5 in string form.
                Value::String(value)
            } else {
                Value::String(value)
            };
            object.insert(key, value);
        }
        let mut body = take_rich(&mut self.content, self.content_line)?;
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
            fields: std::mem::take(&mut self.fields),
            tags: std::mem::take(&mut self.tags),
            origin: Origin {
                path: format!("line {}", self.line),
            },
        })
    }
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
    let value = rich(value.trim(), line)?;
    if !value.is_empty() && !value.is_inline() {
        return markdown_error(
            line,
            "this field accepts inline formatting, not paragraphs or lists",
        );
    }
    Ok(value)
}

fn take_rich(source: &mut String, line: usize) -> Result<Option<RichText>, ParseError> {
    let source = std::mem::take(source);
    if source.trim().is_empty() {
        return Ok(None);
    }
    let value = rich(source.trim_end(), line)?;
    Ok((!value.is_empty()).then_some(value))
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

fn parse_tag_values(value: &str) -> TagSet {
    value
        .split(',')
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
        .collect()
}

/// Rule F, entry context: a key is known when the field table gives it an
/// entry scope **for the entry's kind** — the section's default, since a
/// `Kind:` line inside the same block can still override it. Profile-only
/// keys (`Summary`, `Photo`, …) and other kinds' payloads (`Note:` under an
/// experience entry) are not known, so such prose stays prose instead of
/// vanishing into an open field or erroring. `body`/`content` are model
/// spellings for directive-carried prose, never field lines.
fn entry_key_known(default_kind: &str) -> impl Fn(&str) -> bool + '_ {
    move |key: &str| {
        if matches!(key, "body" | "content") {
            return false;
        }
        core_field(key).is_some_and(|field| field.applies_to_entry_kind(default_kind))
    }
}

fn section_key_known(key: &str) -> bool {
    matches!(key, "kind" | "id" | "tags")
}

/// Map the structural blocks onto the document.
pub(super) fn map(blocks: Vec<Block>, info: &mut MapInfo) -> Result<Mapped, ParseError> {
    let mut profile = Profile::default();
    let mut sections: Vec<Section> = Vec::new();
    let mut section: Option<Section> = None;
    let mut entry: Option<EntryBuilder> = None;
    let mut direct = String::new();
    let mut direct_line = 0usize;
    let mut direct_kind: Option<String> = None;
    let mut last_direct_entry: Option<usize> = None;
    let mut summary = String::new();
    let mut summary_line = 0usize;
    let mut summary_started = false;
    let mut saw_name = false;
    let mut annotation_allowed = false;
    let mut section_ids: Vec<Option<usize>> = Vec::new();
    let mut annotation_kinds: Vec<bool> = Vec::new();

    for block in blocks {
        let line = block.line;
        // Only the section annotation itself may follow its heading
        // directly; every other block ends the annotation's eligibility.
        if !matches!(block.kind, BlockKind::Comment(_)) {
            annotation_allowed = false;
        }
        match block.kind {
            BlockKind::FrontMatter(_) => {
                return markdown_error(line, "front matter must appear before the name heading");
            }
            BlockKind::Heading { level, text } => match level {
                1 => {
                    if saw_name {
                        return markdown_error(
                            line,
                            "the document must contain exactly one level-one name heading",
                        );
                    }
                    profile.name = super::heading(&text, line)?;
                    saw_name = true;
                }
                2 => {
                    if !saw_name {
                        return markdown_error(
                            line,
                            "content must start with a level-one name heading",
                        );
                    }
                    finish_entry(&mut section, &mut entry, info)?;
                    if section.is_some() {
                        flush_direct(
                            &mut section,
                            &mut direct,
                            &mut direct_line,
                            &mut direct_kind,
                        )?;
                    }
                    if let Some(previous) = section.take() {
                        sections.push(previous);
                    }
                    let title = super::heading(&text, line)?;
                    section = Some(Section {
                        id: slugify(&title),
                        kind: super::section_kind(&title),
                        title,
                        entries: Vec::new(),
                        tags: TagSet::new(),
                    });
                    section_ids.push(None);
                    annotation_kinds.push(false);
                    last_direct_entry = None;
                    annotation_allowed = true;
                }
                3 => {
                    if section.is_none() {
                        return markdown_error(
                            line,
                            "an entry heading must be inside a level-two section",
                        );
                    }
                    finish_entry(&mut section, &mut entry, info)?;
                    flush_direct(
                        &mut section,
                        &mut direct,
                        &mut direct_line,
                        &mut direct_kind,
                    )?;
                    last_direct_entry = None;
                    entry = Some(EntryBuilder {
                        heading: text,
                        line,
                        ..Default::default()
                    });
                }
                deeper => {
                    if !saw_name {
                        return markdown_error(
                            line,
                            "content must start with a level-one name heading",
                        );
                    }
                    let raw = format!("{} {}", "#".repeat(deeper as usize), text);
                    if let Some(builder) = &mut entry {
                        builder.push_content(&raw, line);
                    } else if section.is_some() {
                        push_direct(&mut direct, &mut direct_line, &raw, line);
                    } else {
                        push_summary(&mut summary, &mut summary_line, &raw, line);
                    }
                }
            },
            BlockKind::Paragraph(raw) => {
                if !saw_name {
                    return markdown_error(
                        line,
                        "content must start with a level-one name heading",
                    );
                }
                annotation_allowed = false;
                if let Some(builder) = &mut entry {
                    let default_kind = section
                        .as_ref()
                        .map(|active| super::default_entry_kind(&active.kind))
                        .unwrap_or("custom");
                    if let Some(lines) =
                        field_block(&raw, line, entry_key_known(default_kind), false)
                    {
                        for block_line in lines {
                            match block_line {
                                FieldBlockLine::Field(field) => {
                                    if field.key == "tags" {
                                        builder.tags.extend(parse_tag_values(&field.value));
                                    } else {
                                        builder.apply(&field.key, &field.value, field.line)?;
                                    }
                                }
                                FieldBlockLine::PeriodLine {
                                    value,
                                    line: number,
                                } => {
                                    info.sugar = true;
                                    if builder.period.is_some() {
                                        return markdown_error(
                                            number,
                                            "duplicate `Period`; keep one range",
                                        );
                                    }
                                    builder.period = Some((value, number));
                                }
                                FieldBlockLine::DateLine {
                                    value,
                                    line: number,
                                } => {
                                    info.sugar = true;
                                    if builder.date.is_some() {
                                        return markdown_error(
                                            number,
                                            "duplicate `Date`; keep one date",
                                        );
                                    }
                                    builder.date = Some((value, number));
                                }
                            }
                        }
                    } else {
                        builder.push_content(&raw, line);
                    }
                } else if let Some(active) = &mut section {
                    if let Some(lines) = field_block(&raw, line, section_key_known, false) {
                        let mut applied: Vec<&'static str> = Vec::new();
                        for block_line in lines {
                            let FieldBlockLine::Field(field) = block_line else {
                                return markdown_error(
                                    line,
                                    "section field blocks carry fields only; use a Period or Date under an entry",
                                );
                            };
                            match field.key.as_str() {
                                "kind" if !field.value.is_empty() => {
                                    let from_annotation =
                                        annotation_kinds.last().copied().unwrap_or(false);
                                    if applied.contains(&"kind") || from_annotation {
                                        return markdown_error(
                                            field.line,
                                            "duplicate `Kind`; keep one kind per section",
                                        );
                                    }
                                    applied.push("kind");
                                    active.kind =
                                        serde_json::from_value(Value::String(field.value))
                                            .map_err(|error| ParseError::Markdown {
                                                line: field.line,
                                                message: error.to_string(),
                                            })?;
                                }
                                "id" if !field.value.is_empty() => {
                                    if applied.contains(&"id") {
                                        return markdown_error(
                                            field.line,
                                            "duplicate `Id`; keep one id per section",
                                        );
                                    }
                                    applied.push("id");
                                    active.id = super::heading(&field.value, field.line)?;
                                    *section_ids.last_mut().expect("section pushed") =
                                        Some(field.line);
                                }
                                "tags" => active.tags.extend(parse_tag_values(&field.value)),
                                "kind" | "id" => {}
                                _ => {
                                    return markdown_error(
                                        field.line,
                                        "unknown section field; use Kind or Id; insert a blank line before ordinary prose",
                                    );
                                }
                            }
                        }
                    } else {
                        push_direct(&mut direct, &mut direct_line, &raw, line);
                    }
                } else {
                    // Profile context: field blocks, legacy contacts, summary.
                    let profile_known = |key: &str| {
                        key == "contact"
                            || core_field(key).is_some_and(|field| {
                                field.applies_to(cac_core::FieldScope::Profile)
                            })
                    };
                    if let Some(lines) = field_block(&raw, line, profile_known, true) {
                        for block_line in lines {
                            let FieldBlockLine::Field(field) = block_line else {
                                continue;
                            };
                            apply_profile_field(
                                &mut profile,
                                &field.key,
                                &field.value,
                                field.line,
                            )?;
                        }
                    } else {
                        // The whole paragraph lands in the summary with its
                        // line structure intact; legacy contact lines are a
                        // single-line spelling tried before prose begins.
                        let single_line = raw.lines().count() == 1;
                        let first = raw.trim();
                        if single_line
                            && !summary_started
                            && summary.is_empty()
                            && legacy_contacts(&mut profile, first, line)?
                        {
                            // Only the `·` separator line is compact sugar;
                            // a bare email under the name is neutral.
                            if first.contains('·') {
                                info.sugar = true;
                            }
                            continue;
                        }
                        summary_started = true;
                        push_summary(&mut summary, &mut summary_line, first, line);
                    }
                }
            }
            BlockKind::Rule => {
                if !saw_name {
                    return markdown_error(
                        line,
                        "content must start with a level-one name heading",
                    );
                }
                annotation_allowed = false;
                if let Some(builder) = &mut entry {
                    builder.push_content("---", line);
                } else if section.is_some() {
                    push_direct(&mut direct, &mut direct_line, "---", line);
                } else {
                    push_summary(&mut summary, &mut summary_line, "---", line);
                }
            }
            BlockKind::Comment(raw) => {
                apply_comment(
                    &raw,
                    line,
                    &mut saw_name,
                    &mut annotation_allowed,
                    &mut section,
                    &mut section_ids,
                    &mut annotation_kinds,
                    &mut entry,
                    &mut direct,
                    &mut direct_line,
                    &mut direct_kind,
                    &mut last_direct_entry,
                    &mut summary,
                    &mut summary_line,
                    &mut summary_started,
                    info,
                )?;
            }
        }
    }

    finish_entry(&mut section, &mut entry, info)?;
    if section.is_some() {
        flush_direct(
            &mut section,
            &mut direct,
            &mut direct_line,
            &mut direct_kind,
        )?;
    }
    if let Some(section) = section {
        sections.push(section);
    }
    // Prose paragraphs under the name fill the summary only when the
    // explicit `Summary:` field did not; both spellings together are a
    // duplicate the model cannot hold, so refuse instead of picking one.
    if !summary.is_empty() {
        if profile.summary.is_some() {
            return markdown_error(
                summary_line,
                "duplicate summary; keep the `Summary:` field or the prose paragraph under the name, not both",
            );
        }
        profile.summary = take_rich(&mut summary, summary_line)?;
    }
    let mut cv = CvDocument { profile, sections };
    deduplicate_ids(&mut cv, &section_ids)?;
    Ok(Mapped {
        profile: cv.profile,
        sections: cv.sections,
    })
}

pub(super) fn apply_profile_field(
    profile: &mut Profile,
    key: &str,
    value: &str,
    line: usize,
) -> Result<(), ParseError> {
    let known = key == "contact"
        || core_field(key).is_some_and(|field| field.applies_to(cac_core::FieldScope::Profile));
    if key != "contact" && value.is_empty() {
        // Blank optional values omit the slot instead of storing emptiness.
        return Ok(());
    }
    // Match on the normalized key: `core_field` folds case and separators,
    // so the arms must fold the same way or a known key with a stray
    // spelling would fall through this table.
    let key: &str = &normalize_key(key);
    match key {
        "email" if profile.email.is_some() => return profile_duplicate(key, line),
        "phone" if profile.phone.is_some() => return profile_duplicate(key, line),
        "location" if profile.location.is_some() => return profile_duplicate(key, line),
        "website" if profile.website.is_some() => return profile_duplicate(key, line),
        "summary" if profile.summary.is_some() => return profile_duplicate(key, line),
        "photo" if profile.photo.is_some() => return profile_duplicate(key, line),
        "headline" if profile.headline.is_some() => return profile_duplicate(key, line),
        "address" if profile.address.is_some() => return profile_duplicate(key, line),
        _ => {}
    }
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
    if known {
        match key {
            "email" => profile.email = Some(inline_plain(value, line)?),
            "phone" => profile.phone = Some(inline_plain(value, line)?),
            "location" => profile.location = Some(inline_plain(value, line)?),
            "website" => profile.website = Some(parse_url(value, line)?),
            "summary" => {
                let mut summary = String::new();
                if !value.is_empty() {
                    push_summary(&mut summary, &mut 0, value, line);
                }
                profile.summary = take_rich(&mut summary, line)?;
            }
            "photo" => profile.photo = Some(inline_plain(value, line)?),
            "headline" => profile.headline = Some(inline_plain(value, line)?),
            "address" => profile.address = parse_address(value, line)?,
            // The table said this key is profile-scoped but no arm handles
            // it — a table/arm drift bug. Refuse loudly instead of panicking
            // or storing the value under the wrong slot.
            _ => {
                return markdown_error(
                    line,
                    &format!(
                        "`{key}` is not supported in the profile block; put it in prose or an entry"
                    ),
                );
            }
        }
        return Ok(());
    }
    // Undeclared keys become open profile fields, never errors — but a
    // repeated one is a duplicate, not a replacement.
    let normalized = normalize_key(key);
    if profile.fields.get(&normalized).is_some() {
        return profile_duplicate(key, line);
    }
    profile
        .fields
        .insert(normalized, FieldValue::Text(value.to_string()));
    Ok(())
}

fn inline_plain(value: &str, line: usize) -> Result<String, ParseError> {
    Ok(inline(value, line)?.plain())
}

fn profile_duplicate(key: &str, line: usize) -> Result<(), ParseError> {
    markdown_error(line, &format!("duplicate `{key}` field; keep one value"))
}

/// Parse an `Address:` value of comma-separated parts in the order street,
/// city, region, postal code, country; a single part fills the street. An
/// empty segment leaves its slot unset, so `{city, country}` round-trips
/// without shifting into street and city.
fn parse_address(value: &str, line: usize) -> Result<Option<cac_core::Address>, ParseError> {
    let text = inline_plain(value, line)?;
    if text.is_empty() {
        return Ok(None);
    }
    let segments: Vec<&str> = text.split(',').map(str::trim).collect();
    if segments.len() > 5 {
        return markdown_error(
            line,
            "Address carries at most street, city, region, postal code, and country, separated by commas",
        );
    }
    let at = |index: usize| {
        segments
            .get(index)
            .filter(|part| !part.is_empty())
            .map(|part| (*part).to_owned())
    };
    if segments.iter().all(|part| part.is_empty()) {
        return Ok(None);
    }
    Ok(Some(cac_core::Address {
        street: at(0),
        city: at(1),
        region: at(2),
        postal_code: at(3),
        country: at(4),
    }))
}

fn legacy_contacts(profile: &mut Profile, line: &str, number: usize) -> Result<bool, ParseError> {
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
    // Two unclassifiable parts are words, not a second home town: `Rust ·
    // Go · https://…` is a skill line, not contacts.
    if parts
        .iter()
        .filter(|part| classify(part) == "location")
        .count()
        > 1
    {
        return Ok(false);
    }
    for part in parts {
        let kind = classify(part);
        let value = part
            .trim_start_matches("mailto:")
            .trim_start_matches("tel:");
        apply_profile_field(profile, kind, value, number)?;
    }
    Ok(true)
}

fn push_direct(direct: &mut String, direct_line: &mut usize, raw: &str, line: usize) {
    if direct.is_empty() {
        *direct_line = line;
    }
    if !direct.ends_with("\n\n") && !direct.is_empty() {
        direct.push('\n');
    }
    direct.push_str(raw);
    direct.push('\n');
}

fn push_summary(summary: &mut String, summary_line: &mut usize, raw: &str, line: usize) {
    if summary.is_empty() {
        *summary_line = line;
    }
    if !summary.ends_with("\n\n") && !summary.is_empty() {
        summary.push('\n');
    }
    summary.push_str(raw);
    summary.push('\n');
}

fn finish_entry(
    section: &mut Option<Section>,
    entry: &mut Option<EntryBuilder>,
    info: &mut MapInfo,
) -> Result<(), ParseError> {
    if let Some(builder) = entry.take() {
        let active = section.as_mut().expect("entry belongs to a section");
        let built = builder.build(&active.kind, info)?;
        active.entries.push(built);
    }
    Ok(())
}

fn flush_direct(
    section: &mut Option<Section>,
    direct: &mut String,
    direct_line: &mut usize,
    direct_kind: &mut Option<String>,
) -> Result<(), ParseError> {
    let line = *direct_line;
    let Some(value) = take_rich(direct, *direct_line)? else {
        if direct_kind.is_some() {
            return markdown_error(line, "cac:entry must be followed by nonempty content");
        }
        return Ok(());
    };
    let active = section
        .as_mut()
        .expect("direct content belongs to a section");
    let forced = direct_kind.take();
    let entries = if forced.is_none() {
        if let [Inline::List { start: None, items }] = value.0.as_slice()
            && items.iter().all(RichText::is_inline)
        {
            items
                .iter()
                .map(|item| {
                    if active.kind == SectionKind::Skills {
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
    active.entries.extend(entries.into_iter().map(|kind| Entry {
        kind,
        content: None,
        fields: FieldMap::new(),
        tags: TagSet::new(),
        origin: Origin {
            path: format!("line {line}"),
        },
    }));
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn apply_comment(
    raw: &str,
    line: usize,
    saw_name: &mut bool,
    annotation_allowed: &mut bool,
    section: &mut Option<Section>,
    section_ids: &mut [Option<usize>],
    annotation_kinds: &mut [bool],
    entry: &mut Option<EntryBuilder>,
    direct: &mut String,
    direct_line: &mut usize,
    direct_kind: &mut Option<String>,
    last_direct_entry: &mut Option<usize>,
    _summary: &mut String,
    _summary_line: &mut usize,
    summary_started: &mut bool,
    info: &mut MapInfo,
) -> Result<(), ParseError> {
    if parse_tags(raw).is_some() {
        // Anything but the section annotation itself ends its eligibility,
        // matching the "immediately follow a section heading" rule.
        *annotation_allowed = false;
        let tags = parse_tags(raw).expect("checked");
        if let Some(builder) = entry.as_mut() {
            builder.tags.extend(tags);
            return Ok(());
        }
        let has_content = !direct.trim().is_empty();
        if has_content {
            flush_direct(section, direct, direct_line, direct_kind)?;
            *last_direct_entry = Some(section.as_ref().expect("flushed").entries.len() - 1);
        }
        let Some(active) = section else {
            if !*saw_name {
                return markdown_error(line, "content must start with a level-one name heading");
            }
            return markdown_error(line, "tags must follow a section or entry");
        };
        if let Some(index) = last_direct_entry {
            active.entries[*index].tags.extend(tags);
        } else {
            active.tags.extend(tags);
        }
        return Ok(());
    }
    if raw.starts_with("<!-- tags:") {
        return markdown_error(
            line,
            "invalid tags comment; use a comma-separated list or a JSON array inside <!-- tags: ... -->",
        );
    }
    if raw.starts_with("<!-- cac:section") {
        if !*annotation_allowed {
            return markdown_error(
                line,
                "section annotation must immediately follow a section heading and may appear only once",
            );
        }
        *annotation_allowed = false;
        let active = section.as_mut().expect("annotation follows a section");
        parse_section_annotation(raw, line, active)?;
        if raw.split_whitespace().any(|part| part.starts_with("id="))
            && let Some(slot) = section_ids.last_mut()
        {
            *slot = Some(line);
        }
        if raw.split_whitespace().any(|part| part.starts_with("kind="))
            && let Some(flag) = annotation_kinds.last_mut()
        {
            *flag = true;
        }
        return Ok(());
    }
    *annotation_allowed = false;
    if raw.starts_with("<!-- cac:entry") {
        if section.is_none() {
            return markdown_error(line, "cac:entry must be inside a section");
        }
        finish_entry(section, entry, info)?;
        flush_direct(section, direct, direct_line, direct_kind)?;
        *last_direct_entry = None;
        *direct_kind = match raw {
            "<!-- cac:entry kind=prose -->" => Some("prose".into()),
            "<!-- cac:entry kind=text -->" => Some("text".into()),
            _ => {
                return markdown_error(
                    line,
                    "expected <!-- cac:entry kind=prose --> or <!-- cac:entry kind=text -->",
                );
            }
        };
        *direct_line = line;
        return Ok(());
    }
    if raw == "<!-- cac:summary -->" {
        if section.is_some() || *summary_started {
            return markdown_error(
                line,
                "cac:summary must appear once before summary content and sections",
            );
        }
        *summary_started = true;
        return Ok(());
    }
    if raw.starts_with("<!-- cac:") {
        return markdown_error(
            line,
            "unknown cac directive; expected cac:section or cac:entry",
        );
    }
    // A plain comment carries no directive; the model has no slot for it,
    // so `fmt` must refuse to rewrite the file rather than lose it.
    info.plain_comments += 1;
    Ok(())
}

fn deduplicate_ids(cv: &mut CvDocument, section_ids: &[Option<usize>]) -> Result<(), ParseError> {
    let mut reserved: BTreeMap<String, usize> = BTreeMap::new();
    for (section, explicit) in cv.sections.iter().zip(section_ids) {
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
    for (section, explicit) in cv.sections.iter_mut().zip(section_ids) {
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
    Ok(())
}
