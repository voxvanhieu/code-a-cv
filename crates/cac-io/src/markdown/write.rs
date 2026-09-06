use std::fmt::Write as _;

use cac_core::{CvDocument, EntryKind, RichText, TagSet, escape_markdown};

use crate::ParseError;

pub fn to_markdown(cv: &CvDocument) -> String {
    let mut output = format!("# {}\n", escape_markdown(&cv.profile.name));
    for (key, value) in [
        ("Email", cv.profile.email.as_deref()),
        ("Phone", cv.profile.phone.as_deref()),
        ("Location", cv.profile.location.as_deref()),
        (
            "Website",
            cv.profile.website.as_ref().map(|url| url.as_str()),
        ),
    ] {
        if let Some(value) = value {
            let _ = writeln!(output, "{key}: {}", escape_markdown(value));
        }
    }
    for contact in &cv.profile.contacts {
        let _ = writeln!(
            output,
            "Contact: {}",
            contact.href.as_ref().map_or_else(
                || escape_markdown(&contact.label),
                |href| format!(
                    "[{}](<{}>)",
                    escape_markdown(&contact.label),
                    href.as_str().replace('<', "%3C").replace('>', "%3E")
                ),
            )
        );
    }
    output.push('\n');
    if let Some(summary) = &cv.profile.summary {
        let _ = writeln!(output, "<!-- cac:summary -->\n{}\n", summary.to_markdown());
    }
    for section in &cv.sections {
        let _ = writeln!(
            output,
            "## {}\nKind: {}\nId: {}",
            escape_markdown(&section.title),
            section.kind.as_str(),
            escape_markdown(&section.id)
        );
        write_tags(&section.tags, &mut output);
        output.push('\n');
        for entry in &section.entries {
            if let EntryKind::Prose(value) = &entry.kind {
                let _ = writeln!(
                    output,
                    "<!-- cac:entry kind=prose -->\n{}",
                    value.body.to_markdown()
                );
                write_tags(&entry.tags, &mut output);
                output.push('\n');
                continue;
            }
            if let EntryKind::Text(value) = &entry.kind
                && !value.body.is_inline()
            {
                let _ = writeln!(
                    output,
                    "<!-- cac:entry kind=text -->\n{}",
                    value.body.to_markdown()
                );
                write_tags(&entry.tags, &mut output);
                output.push('\n');
                continue;
            }
            let fields = serde_json::to_value(&entry.kind).unwrap();
            let _ = writeln!(
                output,
                "### {}\nKind: {}",
                entry.kind.heading().0.to_markdown(),
                fields["type"].as_str().unwrap()
            );
            for key in [
                "organization",
                "institution",
                "publisher",
                "location",
                "url",
            ] {
                if let Some(value) = fields.get(key).and_then(|v| v.as_str()) {
                    if value.is_empty() {
                        continue;
                    }
                    let label = match key {
                        "organization" => "Organization",
                        "institution" => "Institution",
                        "publisher" => "Publisher",
                        "location" => "Location",
                        _ => "URL",
                    };
                    let value = if matches!(key, "location" | "url") {
                        escape_markdown(value)
                    } else {
                        value.to_owned()
                    };
                    let _ = writeln!(output, "{label}: {value}");
                }
            }
            if let Some(period) = entry.kind.period() {
                let _ = writeln!(output, "Period: {period}");
            }
            if let Some(date) = entry.kind.date() {
                let _ = writeln!(output, "Date: {date}");
            }
            write_tags(&entry.tags, &mut output);
            output.push('\n');
            if let Some(body) = &entry.content {
                let _ = writeln!(output, "{}\n", body.to_markdown());
            } else if !entry.kind.highlights().is_empty() {
                let content = RichText(vec![cac_core::Inline::List {
                    start: None,
                    items: entry.kind.highlights().to_vec(),
                }]);
                let _ = writeln!(output, "{}\n", content.to_markdown());
            }
        }
    }
    output
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

fn write_tags(tags: &TagSet, output: &mut String) {
    if !tags.is_empty() {
        let tags = serde_json::to_string(tags)
            .unwrap()
            .replace('<', "\\u003c")
            .replace('>', "\\u003e");
        let _ = writeln!(output, "<!-- tags: {tags} -->");
    }
}
