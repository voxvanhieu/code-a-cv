use std::collections::{BTreeMap, BTreeSet};

use cac_core::CvDocument;
use serde_json::Value;
use thiserror::Error;
use typst::layout::{Frame, FrameItem};
use typst::model::Destination;

use crate::{RenderOptions, RenderedPdf, pdf::render_pdf_document, view::render_view};

const FIXTURES: &[(&str, &str)] = &[
    ("minimal", include_str!("../fixtures/shared/minimal.json")),
    ("complete", include_str!("../fixtures/shared/complete.json")),
    (
        "translated",
        include_str!("../fixtures/shared/translated.json"),
    ),
    (
        "reordered-empty",
        include_str!("../fixtures/shared/reordered-empty.json"),
    ),
    ("mixed", include_str!("../fixtures/shared/mixed.json")),
    (
        "mixed-reversed",
        include_str!("../fixtures/shared/mixed-reversed.json"),
    ),
    ("missing", include_str!("../fixtures/shared/missing.json")),
    (
        "partial-dates",
        include_str!("../fixtures/shared/partial-dates.json"),
    ),
    (
        "long-labels",
        include_str!("../fixtures/shared/long-labels.json"),
    ),
    ("long", include_str!("../fixtures/shared/long.json")),
];

#[derive(Debug, Error)]
#[error("shared theme fixture `{fixture}` failed: {message}")]
pub struct SharedThemeError {
    pub fixture: String,
    pub message: String,
}

pub struct SharedThemeArtifact {
    pub name: String,
    pub pdf: RenderedPdf,
}

/// Render the bundled shared-theme corpus and check content, order, and links.
/// A passing corpus supplements, rather than replaces, visual and source review.
pub fn test_shared_theme(
    options: &RenderOptions,
) -> Result<Vec<SharedThemeArtifact>, SharedThemeError> {
    let mut artifacts = Vec::new();
    for &(name, source) in FIXTURES {
        let cv: CvDocument = serde_json::from_str(source).expect("valid bundled shared fixture");
        check_fixture(name, &cv, options, &mut artifacts)?;
    }
    let cv = serde_json::from_str(FIXTURES.last().expect("long fixture").1).expect("valid fixture");
    let mut continuation = options.clone();
    continuation.settings.pagination = Some(crate::PaginationSettings {
        allow_entry_page_break: Some(true),
    });
    check_fixture("long-breakable", &cv, &continuation, &mut artifacts)?;
    let cv = serde_json::from_str(FIXTURES[1].1).expect("valid complete fixture");
    let mut overrides = options.clone();
    let settings: crate::Settings = serde_json::from_value(serde_json::json!({
        "page": {"paper": "a4", "margins": {"top": "20mm", "bottom": "20mm", "left": "22mm", "right": "22mm"}},
        "typography": {"font": "Libertinus Serif", "heading_font": "DejaVu Sans", "font_size": "11pt", "line_spacing": "0.8em"},
        "style": {"accent_color": "#385070", "body_alignment": "left", "link_underline": false, "header_alignment": "right", "highlight_bullet": "–"},
        "spacing": {"list_item": "0.4em", "section": "1.4em", "section_heading": "0.8em", "entry": "1.1em"},
        "pagination": {"allow_entry_page_break": true}
    })).expect("valid override fixture");
    overrides.settings = crate::Settings {
        theme: options.settings.theme.clone(),
        ..settings
    };
    check_fixture("settings", &cv, &overrides, &mut artifacts)?;
    Ok(artifacts)
}

fn check_fixture(
    name: &str,
    cv: &CvDocument,
    options: &RenderOptions,
    artifacts: &mut Vec<SharedThemeArtifact>,
) -> Result<(), SharedThemeError> {
    let result = (|| -> Result<RenderedPdf, String> {
        let (pdf, document) =
            render_pdf_document(cv, options).map_err(|error| error.to_string())?;
        let mut text = String::new();
        let mut links = BTreeSet::new();
        for page in document.pages() {
            frame_content(&page.frame, &mut text, &mut links);
        }
        let view: Value = serde_json::from_slice(&render_view(cv).map_err(|e| e.to_string())?)
            .expect("serialized view");
        check_content(&view, &text, &links)?;
        Ok(pdf)
    })();
    let pdf = result.map_err(|message| SharedThemeError {
        fixture: name.into(),
        message,
    })?;
    artifacts.push(SharedThemeArtifact {
        name: name.into(),
        pdf,
    });
    Ok(())
}

fn frame_content(frame: &Frame, text: &mut String, links: &mut BTreeSet<String>) {
    for (_, item) in frame.items() {
        match item {
            FrameItem::Group(group) => frame_content(&group.frame, text, links),
            FrameItem::Text(run) => text.push_str(&run.text),
            FrameItem::Link(Destination::Url(url), _) => {
                links.insert(url.to_string());
            }
            _ => {}
        }
    }
}

fn normalize(text: &str) -> String {
    text.chars()
        .filter(|c| !c.is_whitespace() && *c != '\u{ad}')
        .collect()
}

fn visible_strings(
    value: &Value,
    markers: &mut BTreeMap<String, usize>,
    links: &mut BTreeSet<String>,
) {
    match value {
        Value::Object(object) => {
            for (key, value) in object {
                if key == "href" {
                    if let Some(url) = value.as_str() {
                        links.insert(url.into());
                    }
                } else if !matches!(key.as_str(), "id" | "kind" | "role") {
                    visible_strings(value, markers, links);
                }
            }
        }
        Value::Array(values) => {
            for value in values {
                visible_strings(value, markers, links);
            }
        }
        Value::String(text) => {
            for marker in text
                .split(|c: char| !c.is_ascii_alphanumeric())
                .filter(|s| s.starts_with("CAC"))
            {
                *markers.entry(marker.into()).or_default() += 1;
            }
        }
        _ => {}
    }
}

fn check_content(view: &Value, text: &str, actual_links: &BTreeSet<String>) -> Result<(), String> {
    let text = normalize(text);
    let mut markers = BTreeMap::new();
    let mut links = BTreeSet::new();
    visible_strings(view, &mut markers, &mut links);
    let mut repeatable = BTreeMap::new();
    visible_strings(
        &view["profile"]["name"],
        &mut repeatable,
        &mut BTreeSet::new(),
    );
    visible_strings(
        &view["profile"]["contacts"],
        &mut repeatable,
        &mut BTreeSet::new(),
    );
    for section in view["sections"].as_array().expect("sections") {
        visible_strings(&section["title"], &mut repeatable, &mut BTreeSet::new());
    }
    for (marker, expected) in markers {
        let actual = text.matches(&marker).count();
        if actual < expected || (actual != expected && !repeatable.contains_key(&marker)) {
            return Err(format!(
                "content marker `{marker}` appeared {actual} times; expected {expected}"
            ));
        }
    }
    for link in links {
        if !actual_links.contains(&link) {
            return Err(format!("missing link target `{link}`"));
        }
    }
    // Periods and phone numbers have no marker, but are visible contract fields.
    let mut values = BTreeMap::<String, usize>::new();
    let mut repeatable_values = BTreeSet::new();
    for section in view["sections"].as_array().expect("sections") {
        for entry in section["entries"].as_array().expect("entries") {
            if let Some(period) = entry["period"].as_str() {
                *values.entry(normalize(period)).or_default() += 1;
            }
        }
    }
    for contact in view["profile"]["contacts"].as_array().expect("contacts") {
        if contact["kind"] == "phone" {
            repeatable_values.insert(normalize(contact["label"].as_str().expect("label")));
            *values
                .entry(normalize(contact["label"].as_str().expect("label")))
                .or_default() += 1;
        }
    }
    for (value, expected) in values {
        let actual = text.matches(&value).count();
        if actual < expected || (actual != expected && !repeatable_values.contains(&value)) {
            return Err(format!("missing or duplicated visible value `{value}`"));
        }
    }
    let mut offset = 0;
    for section in view["sections"].as_array().expect("sections") {
        let title = normalize(section["title"].as_str().expect("title"));
        let marker = title
            .split_once(|c: char| !c.is_ascii_alphanumeric())
            .map_or(title.as_str(), |(first, _)| first);
        let position = text[offset..]
            .find(marker)
            .ok_or_else(|| format!("section order changed at `{title}`"))?;
        offset += position + marker.len();
        for entry in section["entries"].as_array().expect("entries") {
            let mut bodies = vec![&entry["primary"]];
            if entry["kind"] == "text" {
                bodies.extend(entry["highlights"].as_array().expect("highlights"));
            }
            for body in bodies {
                let mut entry_markers = BTreeMap::new();
                visible_strings(body, &mut entry_markers, &mut BTreeSet::new());
                if let Some(marker) = entry_markers.keys().next() {
                    let position = text[offset..]
                        .find(marker)
                        .ok_or_else(|| format!("entry order changed at `{marker}`"))?;
                    offset += position + marker.len();
                }
            }
        }
    }
    Ok(())
}

#[cfg(test)]
#[path = "../tests/unit/shared.rs"]
mod tests;
