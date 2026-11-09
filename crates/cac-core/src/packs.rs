//! Field packs: declared meanings for open fields (M2).
//!
//! A pack is a table of open-field keys with labels, value kinds, and
//! requiredness — shipped with a theme or shared under a pack directory.
//! `cac check` validates open-field values against the pack and reports
//! undeclared keys as advisories. The core stays filesystem-free: parsing
//! the TOML happens in cac-io.

use std::collections::BTreeMap;

use serde::Deserialize;

use crate::{DatePoint, FieldValue};

/// The value kind a pack declares for an open field.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PackType {
    #[default]
    Text,
    Integer,
    Boolean,
    Length,
    Date,
}

/// One declared open field. Unknown keys in a field table are parse errors.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PackFieldDef {
    pub label: String,
    #[serde(rename = "type", default)]
    pub kind: PackType,
    #[serde(default)]
    pub required: bool,
}

/// A named field pack.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub struct FieldPack {
    pub name: String,
    #[serde(default)]
    pub profile: BTreeMap<String, PackFieldDef>,
    #[serde(default)]
    pub entry: BTreeMap<String, PackFieldDef>,
}

impl FieldPack {
    /// The declared definition for an open field key, profile or entry.
    pub fn field(&self, scope: PackScope, key: &str) -> Option<&PackFieldDef> {
        let normalized = crate::fields::normalize_key(key);
        match scope {
            PackScope::Profile => self.profile.get(&normalized),
            PackScope::Entry => self.entry.get(&normalized),
        }
    }
}

/// Where an open field appeared.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PackScope {
    Profile,
    Entry,
}

/// Validate one open-field value against its declared kind.
///
/// Markdown and front matter carry every open field as text, so text is
/// coerced through the declared kind before it is judged: `Dependents: 2`
/// passes an `integer` field, `Remote: yes` a `boolean` field. Values that
/// already carry a type validate directly.
pub fn validate_field_value(value: &FieldValue, kind: &PackType) -> Result<(), String> {
    match (value, kind) {
        (FieldValue::Text(text), PackType::Length) => {
            let text = text.trim();
            if parses_as_length(text) {
                Ok(())
            } else {
                Err(format!(
                    "`{text}` is not a length; use a number with a unit such as `32mm`"
                ))
            }
        }
        (FieldValue::Text(text), PackType::Integer) => {
            if text.trim().parse::<i64>().is_ok() {
                Ok(())
            } else {
                Err(format!(
                    "`{text}` is not an integer; write the number without a unit"
                ))
            }
        }
        (FieldValue::Text(text), PackType::Boolean) => {
            if boolean_text(text).is_some() {
                Ok(())
            } else {
                Err(format!("`{text}` is not a boolean; use yes or no"))
            }
        }
        (FieldValue::Text(text), PackType::Date) => {
            if DatePoint::parse(text.trim()).is_some() {
                Ok(())
            } else {
                Err(format!(
                    "`{text}` is not a date; use a year, YYYY-MM, or YYYY-MM-DD"
                ))
            }
        }
        (FieldValue::Integer(_), PackType::Integer)
        | (FieldValue::Boolean(_), PackType::Boolean)
        | (FieldValue::Date(_), PackType::Date)
        | (FieldValue::Text(_), PackType::Text) => Ok(()),
        (value, kind) => Err(format!(
            "a {kind:?} field cannot carry this value ({value:?})"
        )),
    }
}

/// Accept `32mm`, `12.5%`, `0pt` — a finite nonnegative number with a unit.
fn parses_as_length(text: &str) -> bool {
    ["pt", "mm", "cm", "in", "em", "rem", "%"]
        .iter()
        .any(|unit| {
            text.strip_suffix(unit)
                .and_then(|number| number.trim().parse::<f64>().ok())
                .is_some_and(|number| number.is_finite() && number >= 0.0)
                && !text
                    .strip_suffix(unit)
                    .unwrap_or_default()
                    .trim()
                    .is_empty()
        })
}

/// The booleans an author can write in text: `yes`/`no`, `true`/`false`.
fn boolean_text(text: &str) -> Option<bool> {
    match text.trim().to_ascii_lowercase().as_str() {
        "yes" | "true" => Some(true),
        "no" | "false" => Some(false),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn validate(text: &str, kind: PackType) -> Result<(), String> {
        validate_field_value(&FieldValue::Text(text.into()), &kind)
    }

    #[test]
    fn text_coerces_to_the_declared_integer() {
        assert_eq!(validate("2", PackType::Integer), Ok(()));
        assert_eq!(validate(" 2 ", PackType::Integer), Ok(()));
        assert_eq!(validate("-1", PackType::Integer), Ok(()));
        assert!(validate("two", PackType::Integer).is_err());
        assert!(validate("2nd", PackType::Integer).is_err());
        assert_eq!(
            validate_field_value(&FieldValue::Integer(2), &PackType::Integer),
            Ok(())
        );
        assert!(
            validate_field_value(&FieldValue::Boolean(true), &PackType::Integer).is_err(),
            "typed values still validate directly against their own kind"
        );
    }

    #[test]
    fn text_coerces_to_the_declared_boolean() {
        for yes in ["yes", "true", " TRUE ", "No", "false"] {
            assert_eq!(validate(yes, PackType::Boolean), Ok(()), "{yes}");
        }
        assert!(validate("maybe", PackType::Boolean).is_err());
        assert!(validate("", PackType::Boolean).is_err());
        assert_eq!(
            validate_field_value(&FieldValue::Boolean(true), &PackType::Boolean),
            Ok(())
        );
    }

    #[test]
    fn text_coerces_to_the_declared_date() {
        for date in ["2026", "2026-04", "2026-04-01", "Present"] {
            assert_eq!(validate(date, PackType::Date), Ok(()), "{date}");
        }
        assert!(validate("soon", PackType::Date).is_err());
        assert!(validate("2026-13", PackType::Date).is_err());
        let date: DatePoint = serde_json::from_str("\"2026-04\"").unwrap();
        assert_eq!(
            validate_field_value(&FieldValue::Date(date), &PackType::Date),
            Ok(())
        );
    }

    #[test]
    fn text_lengths_need_a_number_and_a_unit() {
        for length in ["32mm", "12.5%", "0pt", " 3 cm "] {
            assert_eq!(validate(length, PackType::Length), Ok(()), "{length}");
        }
        for not_length in ["32", "mm", "half%", "1cm2"] {
            assert!(
                validate(not_length, PackType::Length).is_err(),
                "{not_length}"
            );
        }
    }

    #[test]
    fn text_kind_accepts_any_text() {
        assert_eq!(validate("anything at all", PackType::Text), Ok(()));
    }
}
