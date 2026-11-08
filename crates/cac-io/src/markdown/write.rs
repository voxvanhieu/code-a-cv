use std::fmt::Write as _;

use cac_core::{CvDocument, EntryKind, RichText, TagSet, escape_markdown};

use crate::ParseError;
use url::Url;

/// The two spellings of FieldMark (rule G in `docs/authoring/markdown.md`).
/// `Preserve` is a formatting-time choice resolved against the source.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum MarkdownStyle {
    /// Every field spelled; the diff-friendly canonical form.
    Explicit,
    /// Sugar where unambiguous: comma headings, bare period lines, implied
    /// section kinds and slug ids omitted.
    Compact,
    /// Keep the spelling the source used; the default for `cac fmt`.
    #[default]
    Preserve,
}

impl MarkdownStyle {
    /// Parse the `--style` spelling.
    pub fn parse_flag(value: &str) -> Option<Self> {
        match value {
            "explicit" => Some(Self::Explicit),
            "compact" => Some(Self::Compact),
            "preserve" => Some(Self::Preserve),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Explicit => "explicit",
            Self::Compact => "compact",
            Self::Preserve => "preserve",
        }
    }
}

/// Detect the spelling a source used, so `preserve` keeps it.
///
/// Compact markers: comma sugar in an entry heading, a bare period or date
/// line directly under an entry heading, or a `·` contact line under the
/// name. Everything else reads as explicit.
pub fn detect_style(source: &str) -> MarkdownStyle {
    let mut previous_was_entry_heading = false;
    let mut previous_was_name_heading = false;
    for raw in source.lines() {
        let line = raw.trim();
        if line.starts_with("### ") {
            if line.contains(", ") {
                return MarkdownStyle::Compact;
            }
            previous_was_entry_heading = true;
            previous_was_name_heading = false;
            continue;
        }
        if line.starts_with('#') {
            previous_was_entry_heading = false;
            previous_was_name_heading = line.starts_with("# ");
            continue;
        }
        if line.is_empty() || line.starts_with("<!--") || line.contains(": ") {
            continue;
        }
        if previous_was_entry_heading {
            return MarkdownStyle::Compact;
        }
        if previous_was_name_heading && line.contains(" · ") {
            return MarkdownStyle::Compact;
        }
    }
    MarkdownStyle::Explicit
}

/// Render the document in the given spelling.
pub fn to_markdown_with_style(cv: &CvDocument, style: MarkdownStyle) -> String {
    let compact = style == MarkdownStyle::Compact;
    let mut output = heading_line(1, &escape_markdown(&cv.profile.name));
    let mut profile = String::new();
    for (key, value) in [
        ("Email", cv.profile.email.as_deref()),
        ("Phone", cv.profile.phone.as_deref()),
        ("Headline", cv.profile.headline.as_deref()),
        ("Photo", cv.profile.photo.as_deref()),
        ("Location", cv.profile.location.as_deref()),
        (
            "Website",
            cv.profile.website.as_ref().map(|url| url.as_str()),
        ),
    ] {
        if let Some(value) = value {
            let _ = writeln!(profile, "{key}: {}", escape_markdown(value));
        }
    }
    if let Some(address) = &cv.profile.address {
        let parts: Vec<&str> = [
            address.street.as_deref(),
            address.city.as_deref(),
            address.region.as_deref(),
            address.postal_code.as_deref(),
            address.country.as_deref(),
        ]
        .into_iter()
        .flatten()
        .collect();
        if !parts.is_empty() {
            let joined = parts.join(", ");
            let _ = writeln!(output, "Address: {}", escape_markdown(&joined));
        }
    }
    for contact in &cv.profile.contacts {
        let _ = writeln!(
            profile,
            "Contact: {}",
            contact.href.as_ref().map_or_else(
                || escape_markdown(&contact.label),
                |href| format!(
                    "[{}]({})",
                    escape_markdown(&contact.label),
                    styled_href(href)
                ),
            )
        );
    }
    write_open_fields(&cv.profile.fields, &mut profile);
    if !profile.is_empty() {
        output.push('\n');
        output.push_str(&profile);
    }
    if let Some(summary) = &cv.profile.summary {
        let rendered = summary.to_markdown();
        if summary_needs_directive(&rendered) {
            let _ = write!(output, "\n<!-- cac:summary -->\n\n{rendered}");
        } else {
            let _ = write!(output, "\n{rendered}");
        }
        if !output.ends_with('\n') {
            output.push('\n');
        }
    }
    for section in &cv.sections {
        let _ = write!(
            output,
            "\n{}",
            heading_line(2, &escape_markdown(&section.title))
        );
        let mut section_fields = String::new();
        if !compact || super::section_kind(&section.title) != section.kind {
            let _ = writeln!(section_fields, "Kind: {}", section.kind.as_str());
        }
        if !compact || section.id != super::slugify(&section.title) {
            let _ = writeln!(section_fields, "Id: {}", escape_markdown(&section.id));
        }
        write_tag_field(&section.tags, &mut section_fields);
        if !section_fields.is_empty() {
            output.push('\n');
            output.push_str(&section_fields);
        }
        for entry in &section.entries {
            write_entry(entry, compact, &mut output);
        }
    }
    // Blocks join with exactly one blank line; directives that carry their
    // own trailing blank must not double it.
    while output.contains("\n\n\n") {
        output = output.replace("\n\n\n", "\n\n");
    }
    if !output.ends_with('\n') {
        output.push('\n');
    }
    output
}

/// `to_markdown` is the explicit spelling: the canonical, diff-friendly form.
pub fn to_markdown(cv: &CvDocument) -> String {
    to_markdown_with_style(cv, MarkdownStyle::Explicit)
}

/// Wrap a link target in angle brackets only when its characters would
/// otherwise terminate the target early; Prettier strips them otherwise.
fn styled_href(href: &Url) -> String {
    let url = href.as_str();
    if url.chars().any(|c| matches!(c, ' ' | '(' | ')')) {
        format!("<{}>", url.replace('<', "%3C").replace('>', "%3E"))
    } else {
        url.to_string()
    }
}

fn heading_line(level: usize, text: &str) -> String {
    format!("{} {}\n", "#".repeat(level), text)
}

fn write_entry(entry: &cac_core::Entry, compact: bool, output: &mut String) {
    // Text and prose entries with a body that only a directive can carry.
    if let EntryKind::Prose(value) = &entry.kind {
        let rendered = value.body.to_markdown();
        if !value.body.is_inline() || body_needs_directive(&rendered) {
            let _ = write!(output, "\n<!-- cac:entry kind=prose -->\n\n{rendered}\n");
            write_tag_field(&entry.tags, output);
            if !output.ends_with('\n') {
                output.push('\n');
            }
            return;
        }
    }
    if let EntryKind::Text(value) = &entry.kind
        && (!value.body.is_inline() || body_needs_directive(&value.body.to_markdown()))
    {
        let rendered = value.body.to_markdown();
        let _ = write!(output, "\n<!-- cac:entry kind=text -->\n\n{rendered}\n");
        write_tag_field(&entry.tags, output);
        if !output.ends_with('\n') {
            output.push('\n');
        }
        return;
    }
    let fields = serde_json::to_value(&entry.kind).unwrap();
    let kind = fields["type"].as_str().unwrap().to_owned();
    let mut heading = entry.kind.heading().0.to_markdown();
    let mut block = String::new();
    // Compact sugar: the subtitle rides in the heading when the slot is
    // present, filled, and free of the separator itself.
    let mut sugared_secondary = false;
    if compact {
        let (primary, secondary) = entry.kind.heading();
        let slot = match &entry.kind {
            EntryKind::Experience(value) => Some(&value.organization),
            EntryKind::Education(value) => Some(&value.institution),
            _ => None,
        };
        if let (Some(slot), Some(secondary)) = (slot, secondary)
            && !slot.is_empty()
            && !secondary.is_empty()
            && !primary.to_markdown().contains(", ")
        {
            heading = format!("{}, {}", primary.to_markdown(), secondary.to_markdown());
            sugared_secondary = true;
        }
    }
    let heading_text = heading;
    let _ = write!(output, "\n{}", heading_line(3, &heading_text));
    if !compact || !sugared_kind_implied(&kind) {
        block.push_str(&format!("Kind: {kind}\n"));
    }
    write_open_fields(&entry.fields, &mut block);
    // The primary slot is already the heading; never re-emit it.
    let primary_key = match kind.as_str() {
        "experience" => "role",
        "education" => "qualification",
        "publication" => "title",
        "custom" => "heading",
        "project" | "skill-group" | "language" | "reference" => "name",
        _ => "",
    };
    for key in [
        "organization",
        "institution",
        "publisher",
        "location",
        "url",
        "proficiency",
        "score",
        "role",
        "email",
        "phone",
        "note",
    ] {
        if key == primary_key {
            continue;
        }
        if sugared_secondary && matches!(key, "organization" | "institution") {
            continue;
        }
        if let Some(value) = fields.get(key).and_then(|value| value.as_str()) {
            if value.is_empty() {
                continue;
            }
            let label = match key {
                "organization" => "Organization",
                "institution" => "Institution",
                "publisher" => "Publisher",
                "location" => "Location",
                "score" => "Score",
                "proficiency" => "Proficiency",
                "role" => "Role",
                "email" => "Email",
                "phone" => "Phone",
                "note" => "Note",
                _ => "URL",
            };
            let value = if matches!(key, "location" | "url") {
                escape_markdown(value)
            } else {
                value.to_owned()
            };
            let _ = writeln!(block, "{label}: {value}");
        }
    }
    if let Some(period) = entry.kind.period() {
        if compact {
            let _ = writeln!(block, "{period}");
        } else {
            let _ = writeln!(block, "Period: {period}");
        }
    }
    if let Some(date) = entry.kind.date() {
        if compact {
            let _ = writeln!(block, "{date}");
        } else {
            let _ = writeln!(block, "Date: {date}");
        }
    }
    write_tag_field(&entry.tags, &mut block);
    if !block.is_empty() {
        output.push('\n');
        output.push_str(&block);
    }
    if let Some(body) = &entry.content {
        let _ = write!(output, "\n{}", body.to_markdown());
    } else if !entry.kind.highlights().is_empty() {
        let content = RichText(vec![cac_core::Inline::List {
            start: None,
            items: entry.kind.highlights().to_vec(),
        }]);
        let _ = write!(output, "\n{}", content.to_markdown());
    }
    if !output.ends_with('\n') {
        output.push('\n');
    }
}

/// Entry kinds whose default comes from the section title; the compact
/// spelling omits the redundant `Kind:` only inside those sections, which
/// the caller checks — under a heading entry the kind is always spelled.
fn sugared_kind_implied(_kind: &str) -> bool {
    false
}

/// The summary needs its directive only when a plain paragraph would be
/// misread — as a contact line, a field block, or a date.
fn summary_needs_directive(rendered: &str) -> bool {
    rendered.lines().any(|line| {
        let line = line.trim();
        if line.is_empty() {
            return false;
        }
        line.contains(" · ")
            || line.split_once(':').is_some_and(|(key, _)| {
                !key.is_empty()
                    && key
                        .chars()
                        .all(|c| c.is_alphanumeric() || matches!(c, ' ' | '-' | '_'))
            })
    })
}

/// A text/prose body needs its directive when it is not inline or when its
/// first line would read as a field.
fn body_needs_directive(rendered: &str) -> bool {
    rendered.lines().next().is_some_and(|line| {
        let line = line.trim();
        !line.is_empty()
            && line.split_once(':').is_some_and(|(key, _)| {
                !key.is_empty()
                    && key
                        .chars()
                        .all(|c| c.is_alphanumeric() || matches!(c, ' ' | '-' | '_'))
            })
    })
}

/// Open fields round-trip as ordinary `Key: value` lines; the parser
/// classifies unknown keys back into the open map.
fn write_open_fields(fields: &cac_core::FieldMap, output: &mut String) {
    for field in fields.iter() {
        let label = capitalize_key(&field.key);
        let _ = writeln!(output, "{label}: {}", field.value.as_text());
    }
}

fn capitalize_key(key: &str) -> String {
    let mut characters = key.chars();
    match characters.next() {
        Some(first) => first.to_uppercase().collect::<String>() + characters.as_str(),
        None => String::new(),
    }
}

/// Tags are fields in both spellings. A tag that itself contains a comma
/// only round-trips through the JSON comment form, so that form is used
/// when needed and the plain field otherwise. A comment needs blank lines
/// around it to stay its own block for formatters.
fn write_tag_field(tags: &TagSet, output: &mut String) {
    if tags.is_empty() {
        return;
    }
    if tags.iter().any(|tag| tag.contains(',')) {
        if !output.is_empty() && !output.ends_with("\n\n") {
            output.push('\n');
        }
        let json = serde_json::to_string(tags)
            .unwrap()
            .replace('<', "\\u003c")
            .replace('>', "\\u003e");
        let _ = writeln!(output, "<!-- tags: {json} -->\n");
        return;
    }
    let values = tags
        .iter()
        .map(|tag| escape_markdown(tag))
        .collect::<Vec<_>>()
        .join(", ");
    let _ = writeln!(output, "Tags: {values}");
}

pub fn to_markdown_checked(cv: &CvDocument) -> Result<String, ParseError> {
    crate::validate(cv)?;
    let output = to_markdown(cv);
    let reparsed = super::parse_markdown(&output)?;
    if &reparsed != cv {
        let expected = serde_json::to_value(cv).unwrap();
        let actual = serde_json::to_value(reparsed).unwrap();
        return Err(ParseError::Validation(format!(
            "Markdown conversion cannot preserve {}; use JSON, YAML, or TOML instead",
            difference(&expected, &actual, "cv")
        )));
    }
    Ok(output)
}

pub(crate) fn difference(
    expected: &serde_json::Value,
    actual: &serde_json::Value,
    path: &str,
) -> String {
    match (expected, actual) {
        (serde_json::Value::Object(left), serde_json::Value::Object(right)) => {
            for (key, value) in left {
                if Some(value) != right.get(key) {
                    return difference(
                        value,
                        &right.get(key).cloned().unwrap_or(serde_json::Value::Null),
                        &format!("{path}.{key}"),
                    );
                }
            }
        }
        (serde_json::Value::Array(left), serde_json::Value::Array(right)) => {
            for (index, value) in left.iter().enumerate() {
                if Some(value) != right.get(index) {
                    return difference(
                        value,
                        &right.get(index).cloned().unwrap_or(serde_json::Value::Null),
                        &format!("{path}[{index}]"),
                    );
                }
            }
        }
        _ => {}
    }
    path.to_owned()
}
