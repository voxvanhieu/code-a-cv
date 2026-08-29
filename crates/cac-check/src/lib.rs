use cac_core::{CvDocument, EntryKind};
use serde::Serialize;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Warning,
    Error,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Diagnostic {
    pub code: &'static str,
    pub severity: Severity,
    pub message: String,
    pub path: String,
}

pub fn check_content(cv: &CvDocument) -> Vec<Diagnostic> {
    let mut output = Vec::new();
    if cv.profile.email.is_none() && cv.profile.phone.is_none() {
        output.push(diagnostic(
            "CAC101",
            Severity::Error,
            "add an email address or phone number",
            "profile",
        ));
    }
    for (section_index, section) in cv.sections.iter().enumerate() {
        if section.entries.is_empty() {
            output.push(diagnostic(
                "CAC103",
                Severity::Warning,
                "remove the empty section or add an entry",
                &format!("sections[{section_index}]"),
            ));
        }
        for (entry_index, entry) in section.entries.iter().enumerate() {
            let path = format!("sections[{section_index}].entries[{entry_index}]");
            let highlights = match &entry.kind {
                EntryKind::Experience(value) => value.highlights.as_slice(),
                EntryKind::Education(value) => value.highlights.as_slice(),
                EntryKind::Project(value) => value.highlights.as_slice(),
                EntryKind::Publication(value) => value.highlights.as_slice(),
                EntryKind::SkillGroup(_) | EntryKind::Text(_) => &[],
            };
            if matches!(
                entry.kind,
                EntryKind::Experience(_) | EntryKind::Education(_) | EntryKind::Project(_)
            ) && highlights.is_empty()
            {
                output.push(diagnostic(
                    "CAC204",
                    Severity::Warning,
                    "add at least one concise highlight",
                    &path,
                ));
            }
            for (highlight_index, highlight) in highlights.iter().enumerate() {
                let plain = highlight.plain();
                let highlight_path = format!("{path}.highlights[{highlight_index}]");
                if !plain.chars().any(|character| character.is_ascii_digit())
                    && !plain.contains('%')
                    && !contains_duration(&plain)
                {
                    output.push(diagnostic(
                        "CAC201",
                        Severity::Warning,
                        "add a number, percentage, or duration when the result is measurable",
                        &highlight_path,
                    ));
                }
                let lower = plain.to_ascii_lowercase();
                if ["helped ", "worked on ", "responsible for "]
                    .iter()
                    .any(|prefix| lower.starts_with(prefix))
                {
                    output.push(diagnostic(
                        "CAC202",
                        Severity::Warning,
                        "open with a specific action verb",
                        &highlight_path,
                    ));
                }
                if first_person(&lower) {
                    output.push(diagnostic(
                        "CAC301",
                        Severity::Warning,
                        "remove first-person pronouns",
                        &highlight_path,
                    ));
                }
            }
        }
    }
    let granularities: Vec<u8> = cv
        .sections
        .iter()
        .flat_map(|section| &section.entries)
        .filter_map(|entry| entry.kind.period())
        .flat_map(|period| [&period.start, &period.end])
        .map(|point| point.granularity())
        .filter(|value| *value != 0)
        .collect();
    if let Some(first) = granularities.first()
        && granularities.iter().any(|value| value != first)
    {
        output.push(diagnostic(
            "CAC603",
            Severity::Warning,
            "use one date granularity throughout the CV",
            "sections",
        ));
    }
    output
}

fn contains_duration(value: &str) -> bool {
    let lower = value.to_ascii_lowercase();
    [" year", " month", " week", " day", " hour"]
        .iter()
        .any(|unit| lower.contains(unit))
}

fn first_person(value: &str) -> bool {
    value
        .split(|character: char| !character.is_alphanumeric() && character != '\'')
        .any(|word| matches!(word, "i" | "i'm" | "i've" | "my" | "mine"))
}

fn diagnostic(code: &'static str, severity: Severity, message: &str, path: &str) -> Diagnostic {
    Diagnostic {
        code,
        severity,
        message: message.into(),
        path: path.into(),
    }
}
