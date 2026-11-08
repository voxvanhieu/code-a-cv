//! Field packs: declared meanings for open fields (M2).
//!
//! A pack is a table of open-field keys with labels, value kinds, and
//! requiredness — shipped with a theme or shared under a pack directory.
//! `cac check` validates open-field values against the pack and reports
//! undeclared keys as advisories. The core stays filesystem-free: loading
//! the TOML happens in cac-io.

use std::collections::BTreeMap;

use serde::Deserialize;

use crate::FieldValue;

/// The value kind a pack declares for an open field.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PackType {
    Text,
    Integer,
    Boolean,
    Length,
}

/// One declared open field.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub struct PackFieldDef {
    pub label: String,
    #[serde(rename = "type")]
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
pub fn validate_field_value(value: &FieldValue, kind: &PackType) -> Result<(), String> {
    match (value, kind) {
        (FieldValue::Text(text), PackType::Length) => {
            let text = text.trim();
            let units = ["pt", "mm", "cm", "in", "em", "rem", "%"];
            let well_formed = units.iter().any(|unit| {
                text.strip_suffix(unit)
                    .and_then(|number| number.trim().parse::<f64>().ok())
                    .is_some_and(|number| number.is_finite() && number >= 0.0)
                    && !text.trim_end_matches(unit).trim().is_empty()
                    || (unit == &"%" && text.ends_with('%'))
            });
            if well_formed {
                Ok(())
            } else {
                Err(format!(
                    "`{text}` is not a length; use a number with a unit such as `32mm`"
                ))
            }
        }
        (FieldValue::Integer(_), PackType::Integer) => Ok(()),
        (FieldValue::Text(text), PackType::Integer) => Err(format!(
            "`{text}` is not an integer; write the number without a unit"
        )),
        (FieldValue::Boolean(_), PackType::Boolean) => Ok(()),
        (FieldValue::Text(text), PackType::Boolean) => {
            Err(format!("`{text}` is not a boolean; use yes or no"))
        }
        (FieldValue::Text(_), PackType::Text) => Ok(()),
        (value, kind) => Err(format!(
            "a {kind:?} field cannot carry this value ({value:?})"
        )),
    }
}
