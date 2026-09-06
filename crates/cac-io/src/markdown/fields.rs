use super::markdown_error;
use crate::ParseError;
use cac_core::{DatePoint, Period, Section, TagSet};
use chrono::NaiveDate;
use std::collections::BTreeSet;

fn decode_section_id(value: &str) -> Option<String> {
    let mut bytes = Vec::new();
    let mut input = value.bytes();
    while let Some(byte) = input.next() {
        bytes.push(if byte == b'%' {
            let high = char::from(input.next()?).to_digit(16)?;
            let low = char::from(input.next()?).to_digit(16)?;
            (high * 16 + low) as u8
        } else {
            byte
        });
    }
    String::from_utf8(bytes)
        .ok()
        .filter(|id| !id.trim().is_empty())
}

pub(super) fn parse_section_annotation(
    line: &str,
    number: usize,
    section: &mut Section,
) -> Result<(), ParseError> {
    let Some(body) = line
        .strip_prefix("<!-- cac:section ")
        .and_then(|v| v.strip_suffix("-->"))
    else {
        return markdown_error(
            number,
            "invalid section annotation; expected <!-- cac:section id=... kind=... -->",
        );
    };
    let mut keys = BTreeSet::new();
    for field in body.split_whitespace() {
        let Some((key, value)) = field.split_once('=') else {
            return markdown_error(number, "expected key=value in section annotation");
        };
        if !keys.insert(key) {
            return markdown_error(number, "duplicate section annotation field");
        }
        match key {
            "id" => {
                section.id = decode_section_id(value).ok_or_else(|| ParseError::Markdown {
                    line: number,
                    message: "invalid or empty section id".into(),
                })?
            }
            "kind" => {
                section.kind = serde_json::from_value(serde_json::Value::String(value.into()))
                    .map_err(|_| ParseError::Markdown {
                        line: number,
                        message: format!("invalid section kind `{value}`"),
                    })?
            }
            _ => {
                return markdown_error(
                    number,
                    "unknown section annotation field; expected id or kind",
                );
            }
        }
    }
    if keys.is_empty() {
        return markdown_error(number, "section annotation must specify id or kind");
    }
    Ok(())
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

pub(super) fn parse_tags(line: &str) -> Option<TagSet> {
    let body = line.strip_prefix("<!--")?.strip_suffix("-->")?.trim();
    let values = body.strip_prefix("tags:")?.trim();
    if values.starts_with('[') {
        return serde_json::from_str(values).ok();
    }
    Some(
        values
            .split(',')
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_owned)
            .collect(),
    )
}

pub(super) fn parse_period(value: &str) -> Option<Result<Period, String>> {
    let pair = value.split_once('–').or_else(|| value.split_once(" - "))?;
    let point = |value: &str| {
        if value.is_empty() {
            Ok(None)
        } else {
            parse_date_point(value).map(Some).ok_or_else(|| format!("invalid date `{value}`; use YYYY, YYYY-MM, YYYY-MM-DD, or Present as the end; omit an unknown endpoint"))
        }
    };
    Some(point(pair.0.trim()).and_then(|start| {
        point(pair.1.trim())
            .and_then(|end| Period::partial(start, end).map_err(|error| error.to_string()))
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
