//! The field table: one data-driven description of every known CV field.
//!
//! The table replaces the hard-coded field matrices that used to live in the
//! parser's builder, the writer's key lists, and the JSON Schema. Each
//! [`FieldDef`] names a key, its human label, where it may appear, how its
//! value parses, and its JSON Resume mapping when one exists.
//!
//! Two families of fields live here:
//!
//! - **Typed fields** are carried by dedicated model structs (for example
//!   `ExperienceEntry::period`). The table describes them so consumers can
//!   share one source of truth.
//! - **Open fields** are author-written `Key: value` lines the model does not
//!   type. They are collected into [`FieldMap`] on an [`Entry`] or
//!   `Section`/`Profile` and stay valid data end to end.

use std::fmt::{self, Display, Formatter};

use serde::de::{self, Deserializer, MapAccess, Visitor};
use serde::{Deserialize, Serialize, Serializer};

use crate::{DatePoint, RichText};

/// Where a field may appear in a document.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FieldScope {
    /// Under the profile (the name block or front matter).
    Profile,
    /// Under a section heading.
    Section,
    /// Under an entry whose kind is one of the listed names.
    Entries(&'static [EntryKindName]),
    /// On an individual skill item inside a skill group.
    SkillItem,
}

/// The canonical name of an entry kind. `Prose` shares `Text`'s payload and
/// is spelled `Text` here; the alias lives only in [`crate::EntryKind`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EntryKindName {
    Experience,
    Education,
    Project,
    Publication,
    SkillGroup,
    Custom,
    Text,
    Language,
    Reference,
}

impl EntryKindName {
    /// The kebab-case spelling used in data and diagnostics.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Experience => "experience",
            Self::Education => "education",
            Self::Project => "project",
            Self::Publication => "publication",
            Self::SkillGroup => "skill-group",
            Self::Custom => "custom",
            Self::Text => "text",
            Self::Language => "language",
            Self::Reference => "reference",
        }
    }
}

impl Display for EntryKindName {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

/// How the text of a `Key: value` line parses into a typed value.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FieldParser {
    /// Plain text, kept verbatim.
    Text,
    /// Inline rich text (bold, links, code).
    Rich,
    /// A signed integer.
    Integer,
    /// `yes`/`no`, `true`/`false`.
    Boolean,
    /// A single date: year, year-month, or full date.
    Date,
    /// A start–end period.
    Period,
    /// An absolute URL.
    Url,
    /// A skill level: 0–5 or a named scale a theme maps.
    Level,
}

/// One row of the field table.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FieldDef {
    /// The canonical key, normalized to lowercase kebab-case.
    pub key: &'static str,
    /// English label shown by writers and diagnostics; packs may override labels per locale.
    pub label: &'static str,
    /// Every location the field may appear in.
    pub scopes: &'static [FieldScope],
    /// How the value parses.
    pub parser: FieldParser,
    /// The JSON Resume path this field maps to, when one exists.
    pub json_resume: Option<&'static str>,
}

impl FieldDef {
    /// Whether the field may appear at the given scope.
    pub fn applies_to(&self, scope: FieldScope) -> bool {
        match scope {
            // Entry scopes match if the kind lists overlap.
            FieldScope::Entries(kind) => self.scopes.iter().any(|candidate| match candidate {
                FieldScope::Entries(candidate_kind) => {
                    candidate_kind.iter().any(|name| kind.contains(name))
                }
                _ => false,
            }),
            other => self.scopes.contains(&other),
        }
    }

    /// Whether the field may appear on an entry of the given kind.
    pub fn applies_to_entry_kind(&self, kind: EntryKindName) -> bool {
        self.scopes
            .iter()
            .any(|scope| matches!(scope, FieldScope::Entries(kinds) if kinds.contains(&kind)))
    }

    /// Whether the field may appear on an entry of the given kebab-case
    /// kind name — the question a parser asks while the entry's kind is
    /// still its section's default.
    pub fn applies_to_entry_kind_name(&self, kind: &str) -> bool {
        self.scopes
            .iter()
            .any(|scope| matches!(scope, FieldScope::Entries(kinds) if kinds.iter().any(|name| name.as_str() == kind)))
    }
}

/// Entry scope shorthands for the table below.
const ALL_ENTRY_KINDS: &[EntryKindName] = &[
    EntryKindName::Experience,
    EntryKindName::Education,
    EntryKindName::Project,
    EntryKindName::Publication,
    EntryKindName::SkillGroup,
    EntryKindName::Custom,
    EntryKindName::Text,
    EntryKindName::Language,
    EntryKindName::Reference,
];
const DATED_ENTRY_KINDS: &[EntryKindName] = &[
    EntryKindName::Experience,
    EntryKindName::Education,
    EntryKindName::Project,
    EntryKindName::Publication,
    EntryKindName::Custom,
];
const DATED_NO_PUB_KINDS: &[EntryKindName] = &[
    EntryKindName::Experience,
    EntryKindName::Education,
    EntryKindName::Project,
    EntryKindName::Custom,
];

/// Every field the model understands today: typed fields the structs carry,
/// plus the universal extension fields. Sorted by key; lookups are binary.
pub static CORE_FIELDS: &[FieldDef] = &[
    FieldDef {
        key: "address",
        label: "Address",
        scopes: &[FieldScope::Profile],
        parser: FieldParser::Text,
        json_resume: Some("basics.location"),
    },
    FieldDef {
        key: "body",
        label: "Body",
        scopes: &[FieldScope::Entries(&[EntryKindName::Text])],
        parser: FieldParser::Rich,
        json_resume: None,
    },
    FieldDef {
        key: "content",
        label: "Content",
        scopes: &[FieldScope::Entries(&[EntryKindName::Text])],
        parser: FieldParser::Rich,
        json_resume: None,
    },
    FieldDef {
        key: "date",
        label: "Date",
        scopes: &[FieldScope::Entries(DATED_ENTRY_KINDS)],
        parser: FieldParser::Date,
        json_resume: None,
    },
    FieldDef {
        key: "email",
        label: "Email",
        scopes: &[
            FieldScope::Profile,
            FieldScope::Entries(&[EntryKindName::Reference]),
        ],
        parser: FieldParser::Text,
        json_resume: Some("basics.email"),
    },
    FieldDef {
        key: "heading",
        label: "Heading",
        scopes: &[FieldScope::Entries(&[EntryKindName::Custom])],
        parser: FieldParser::Rich,
        json_resume: None,
    },
    FieldDef {
        key: "headline",
        label: "Headline",
        scopes: &[FieldScope::Profile],
        parser: FieldParser::Text,
        json_resume: Some("basics.label"),
    },
    FieldDef {
        key: "highlights",
        label: "Highlights",
        scopes: &[FieldScope::Entries(DATED_ENTRY_KINDS)],
        parser: FieldParser::Rich,
        json_resume: Some("*.highlights"),
    },
    FieldDef {
        key: "id",
        label: "Id",
        scopes: &[FieldScope::Section],
        parser: FieldParser::Text,
        json_resume: None,
    },
    FieldDef {
        key: "institution",
        label: "Institution",
        scopes: &[FieldScope::Entries(&[EntryKindName::Education])],
        parser: FieldParser::Rich,
        json_resume: Some("education[].institution"),
    },
    FieldDef {
        key: "kind",
        label: "Kind",
        scopes: &[FieldScope::Section, FieldScope::Entries(ALL_ENTRY_KINDS)],
        parser: FieldParser::Text,
        json_resume: None,
    },
    FieldDef {
        key: "level",
        label: "Level",
        scopes: &[FieldScope::SkillItem],
        parser: FieldParser::Level,
        json_resume: Some("skills[].level"),
    },
    FieldDef {
        key: "location",
        label: "Location",
        scopes: &[
            FieldScope::Profile,
            FieldScope::Entries(&[EntryKindName::Experience]),
        ],
        parser: FieldParser::Text,
        json_resume: Some("basics.location"),
    },
    FieldDef {
        key: "name",
        label: "Name",
        scopes: &[FieldScope::Entries(&[
            EntryKindName::Project,
            EntryKindName::SkillGroup,
            EntryKindName::Language,
            EntryKindName::Reference,
        ])],
        parser: FieldParser::Rich,
        json_resume: None,
    },
    FieldDef {
        key: "note",
        label: "Note",
        scopes: &[FieldScope::Entries(&[EntryKindName::Reference])],
        parser: FieldParser::Text,
        json_resume: Some("references[].reference"),
    },
    FieldDef {
        key: "organization",
        label: "Organization",
        scopes: &[FieldScope::Entries(&[
            EntryKindName::Experience,
            EntryKindName::Reference,
        ])],
        parser: FieldParser::Rich,
        json_resume: Some("work[].name"),
    },
    FieldDef {
        key: "period",
        label: "Period",
        scopes: &[FieldScope::Entries(DATED_NO_PUB_KINDS)],
        parser: FieldParser::Period,
        json_resume: Some("work[].startDate/endDate"),
    },
    FieldDef {
        key: "phone",
        label: "Phone",
        scopes: &[
            FieldScope::Profile,
            FieldScope::Entries(&[EntryKindName::Reference]),
        ],
        parser: FieldParser::Text,
        json_resume: Some("basics.phone"),
    },
    FieldDef {
        key: "photo",
        label: "Photo",
        scopes: &[FieldScope::Profile],
        parser: FieldParser::Text,
        json_resume: Some("basics.image"),
    },
    FieldDef {
        key: "proficiency",
        label: "Proficiency",
        scopes: &[FieldScope::Entries(&[EntryKindName::Language])],
        parser: FieldParser::Level,
        json_resume: Some("languages[].level"),
    },
    FieldDef {
        key: "publisher",
        label: "Publisher",
        scopes: &[FieldScope::Entries(&[EntryKindName::Publication])],
        parser: FieldParser::Rich,
        json_resume: Some("publications[].publisher"),
    },
    FieldDef {
        key: "qualification",
        label: "Qualification",
        scopes: &[FieldScope::Entries(&[EntryKindName::Education])],
        parser: FieldParser::Rich,
        json_resume: Some("education[].studyType"),
    },
    FieldDef {
        key: "role",
        label: "Role",
        scopes: &[FieldScope::Entries(&[EntryKindName::Reference])],
        parser: FieldParser::Rich,
        json_resume: Some("work[].position"),
    },
    FieldDef {
        key: "score",
        label: "Score",
        scopes: &[FieldScope::Entries(&[EntryKindName::Education])],
        parser: FieldParser::Text,
        json_resume: Some("education[].score"),
    },
    FieldDef {
        key: "skills",
        label: "Skills",
        scopes: &[FieldScope::Entries(&[EntryKindName::SkillGroup])],
        parser: FieldParser::Rich,
        json_resume: Some("skills[].keywords"),
    },
    FieldDef {
        key: "summary",
        label: "Summary",
        scopes: &[FieldScope::Profile],
        parser: FieldParser::Rich,
        json_resume: Some("basics.summary"),
    },
    FieldDef {
        key: "tags",
        label: "Tags",
        scopes: &[FieldScope::Section, FieldScope::Entries(ALL_ENTRY_KINDS)],
        parser: FieldParser::Text,
        json_resume: None,
    },
    FieldDef {
        key: "title",
        label: "Title",
        scopes: &[FieldScope::Entries(&[EntryKindName::Publication])],
        parser: FieldParser::Rich,
        json_resume: Some("publications[].name"),
    },
    FieldDef {
        key: "url",
        label: "URL",
        scopes: &[FieldScope::Entries(&[
            EntryKindName::Project,
            EntryKindName::Publication,
        ])],
        parser: FieldParser::Url,
        json_resume: Some("*.url"),
    },
    FieldDef {
        key: "website",
        label: "Website",
        scopes: &[FieldScope::Profile],
        parser: FieldParser::Url,
        json_resume: Some("basics.url"),
    },
];

/// Look up a core field definition by its normalized key.
pub fn core_field(key: &str) -> Option<&'static FieldDef> {
    let key = normalize_key(key);
    CORE_FIELDS
        .binary_search_by(|field| field.key.cmp(key.as_str()))
        .ok()
        .map(|index| &CORE_FIELDS[index])
}

/// The value of an open field.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FieldValue {
    Text(String),
    Integer(i64),
    Boolean(bool),
    Date(DatePoint),
}

impl FieldValue {
    /// The plain text of the value, as a writer would emit it.
    pub fn as_text(&self) -> String {
        match self {
            Self::Text(value) => value.clone(),
            Self::Integer(value) => value.to_string(),
            Self::Boolean(value) => {
                if *value {
                    "yes".into()
                } else {
                    "no".into()
                }
            }
            Self::Date(value) => value.to_string(),
        }
    }
}

impl Serialize for FieldValue {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Text(value) => serializer.serialize_str(value),
            Self::Integer(value) => serializer.serialize_i64(*value),
            Self::Boolean(value) => serializer.serialize_bool(*value),
            Self::Date(value) => value.serialize(serializer),
        }
    }
}

impl<'de> Deserialize<'de> for FieldValue {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct FieldValueVisitor;

        impl<'de> de::Visitor<'de> for FieldValueVisitor {
            type Value = FieldValue;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("text, an integer, a boolean, or a date")
            }

            fn visit_str<E: de::Error>(self, value: &str) -> Result<Self::Value, E> {
                // A string is text — never a guessed date. Guessing turned
                // every integer-looking string into a year and broke
                // round-trip identity for fields like `postal-code`.
                Ok(FieldValue::Text(value.to_owned()))
            }

            fn visit_f64<E: de::Error>(self, value: f64) -> Result<Self::Value, E> {
                Ok(FieldValue::Text(value.to_string()))
            }

            fn visit_i64<E: de::Error>(self, value: i64) -> Result<Self::Value, E> {
                Ok(FieldValue::Integer(value))
            }

            fn visit_u64<E: de::Error>(self, value: u64) -> Result<Self::Value, E> {
                i64::try_from(value)
                    .map(FieldValue::Integer)
                    .map_err(|_| de::Error::custom("field numbers must fit in a signed integer"))
            }

            fn visit_bool<E: de::Error>(self, value: bool) -> Result<Self::Value, E> {
                Ok(FieldValue::Boolean(value))
            }
        }

        deserializer.deserialize_any(FieldValueVisitor)
    }
}

/// One open field: a key the model does not type, plus its value.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct OpenField {
    pub key: String,
    pub value: FieldValue,
}

/// An ordered map of open fields. Insertion order is preserved through
/// serialization, so native formats keep the author's field order.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct FieldMap {
    fields: Vec<OpenField>,
}

impl FieldMap {
    pub fn new() -> Self {
        Self::default()
    }

    /// Insert or replace a field, keeping the original position on replace.
    pub fn insert(&mut self, key: impl Into<String>, value: FieldValue) {
        let key = normalize_key(&key.into());
        if let Some(existing) = self.fields.iter_mut().find(|field| field.key == key) {
            existing.value = value;
        } else {
            self.fields.push(OpenField { key, value });
        }
    }

    pub fn get(&self, key: &str) -> Option<&FieldValue> {
        let key = &normalize_key(key);
        self.fields
            .iter()
            .find(|field| &field.key == key)
            .map(|field| &field.value)
    }

    pub fn iter(&self) -> impl Iterator<Item = &OpenField> {
        self.fields.iter()
    }

    pub fn len(&self) -> usize {
        self.fields.len()
    }

    pub fn is_empty(&self) -> bool {
        self.fields.is_empty()
    }
}

/// Normalize a field key the way the table does: trim, lowercase, fold
/// whitespace and underscores to hyphens.
pub fn normalize_key(key: &str) -> String {
    let mut normalized = String::with_capacity(key.len());
    let mut previous_was_separator = true;
    for character in key.trim().chars() {
        if character.is_whitespace() || matches!(character, '_' | '-') {
            // Whitespace, underscores, and hyphens are the same separator;
            // runs of them fold to one hyphen instead of stacking.
            if !previous_was_separator {
                normalized.push('-');
            }
            previous_was_separator = true;
        } else {
            normalized.extend(character.to_lowercase());
            previous_was_separator = false;
        }
    }
    while normalized.ends_with('-') {
        normalized.pop();
    }
    normalized
}

impl Serialize for FieldMap {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeMap;
        let mut map = serializer.serialize_map(Some(self.fields.len()))?;
        for field in &self.fields {
            map.serialize_entry(&field.key, &field.value)?;
        }
        map.end()
    }
}

impl<'de> Deserialize<'de> for FieldMap {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct FieldMapVisitor;

        impl<'de> Visitor<'de> for FieldMapVisitor {
            type Value = FieldMap;

            fn expecting(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
                formatter.write_str("a map of field keys to text, integers, booleans, or dates")
            }

            fn visit_map<A: MapAccess<'de>>(self, mut access: A) -> Result<Self::Value, A::Error> {
                let mut map = FieldMap::new();
                while let Some((key, value)) = access.next_entry::<String, FieldValue>()? {
                    if key.trim().is_empty() {
                        return Err(de::Error::custom("field key must not be empty"));
                    }
                    if key.chars().any(|character| {
                        character.is_control()
                            || matches!(character, '<' | '>' | '&' | '"' | '\'' | '\\' | '`')
                    }) {
                        return Err(de::Error::custom(
                            "field key must not contain markup or control characters",
                        ));
                    }
                    map.insert(key, value);
                }
                Ok(map)
            }
        }

        deserializer.deserialize_map(FieldMapVisitor)
    }
}

/// A skill level: numeric on a 0–5 scale, or a named scale a theme maps
/// (for example `Expert` or a CEFR grade such as `C1`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SkillLevel {
    /// 0 to 5.
    Numeric(u8),
    /// A named level, kept verbatim; themes map it to their own scale.
    Named(String),
}

impl SkillLevel {
    /// Parse `4/5`, `4`, or a named level. A numeric spelling outside the
    /// 0–5 scale is an error, never a named level: `10/10` and `-1` are
    /// mistakes, not names.
    pub fn parse(text: &str) -> Option<Self> {
        let text = text.trim();
        if text.is_empty() {
            return None;
        }
        let number = match text.rsplit_once('/') {
            // Only `N/5` counts as a fraction; any other slash spelling
            // (`10/10`) is an invalid number, not a name.
            Some((base, "5")) if !base.is_empty() && base.chars().all(|c| c.is_ascii_digit()) => {
                base
            }
            Some(_) => return None,
            None => text,
        };
        // Any integer spelling — signed or out of range — is an invalid
        // level; digit strings too big for an integer are still numbers.
        if let Ok(value) = number.parse::<i64>() {
            return u8::try_from(value)
                .ok()
                .filter(|value| *value <= 5)
                .map(Self::Numeric);
        }
        if number.chars().all(|c| c.is_ascii_digit()) {
            return None;
        }
        Some(Self::Named(text.into()))
    }
}

impl Display for SkillLevel {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Numeric(value) => write!(formatter, "{value}/5"),
            Self::Named(value) => formatter.write_str(value),
        }
    }
}

impl Serialize for SkillLevel {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Numeric(value) => serializer.serialize_u8(*value),
            Self::Named(value) => serializer.serialize_str(value),
        }
    }
}

impl<'de> Deserialize<'de> for SkillLevel {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct SkillLevelVisitor;

        impl Visitor<'_> for SkillLevelVisitor {
            type Value = SkillLevel;

            fn expecting(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
                formatter.write_str("a level from 0 to 5 or a named level")
            }

            fn visit_u64<E: de::Error>(self, value: u64) -> Result<Self::Value, E> {
                u8::try_from(value)
                    .ok()
                    .filter(|value| *value <= 5)
                    .map(SkillLevel::Numeric)
                    .ok_or_else(|| de::Error::custom("numeric levels run from 0 to 5"))
            }

            fn visit_i64<E: de::Error>(self, value: i64) -> Result<Self::Value, E> {
                u8::try_from(value)
                    .ok()
                    .filter(|value| *value <= 5)
                    .map(SkillLevel::Numeric)
                    .ok_or_else(|| de::Error::custom("numeric levels run from 0 to 5"))
            }

            fn visit_str<E: de::Error>(self, value: &str) -> Result<Self::Value, E> {
                SkillLevel::parse(value).ok_or_else(|| de::Error::custom("empty level"))
            }
        }

        deserializer.deserialize_any(SkillLevelVisitor)
    }
}

/// A skill item with an optional level. Deserializes from a plain string when
/// no level is given, so existing data keeps parsing unchanged.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Skill {
    pub name: RichText,
    pub level: Option<SkillLevel>,
}

impl Serialize for Skill {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match &self.level {
            None => self.name.serialize(serializer),
            Some(level) => {
                use serde::ser::SerializeStruct;
                let mut state = serializer.serialize_struct("Skill", 2)?;
                state.serialize_field("name", &self.name)?;
                state.serialize_field("level", level)?;
                state.end()
            }
        }
    }
}

impl<'de> Deserialize<'de> for Skill {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct SkillVisitor;

        impl<'de> Visitor<'de> for SkillVisitor {
            type Value = Skill;

            fn expecting(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
                formatter.write_str("a skill name or a map with name and optional level")
            }

            fn visit_str<E: de::Error>(self, value: &str) -> Result<Self::Value, E> {
                Ok(Skill {
                    name: RichText::literal(value),
                    level: None,
                })
            }

            fn visit_map<A: MapAccess<'de>>(self, mut access: A) -> Result<Self::Value, A::Error> {
                let mut name = None;
                let mut level = None;
                while let Some(key) = access.next_key::<String>()? {
                    match key.as_str() {
                        "name" => name = Some(access.next_value::<RichText>()?),
                        "level" => level = Some(access.next_value::<SkillLevel>()?),
                        _ => {
                            access.next_value::<de::IgnoredAny>()?;
                            return Err(de::Error::unknown_field(&key, &["name", "level"]));
                        }
                    }
                }
                let name = name.ok_or_else(|| de::Error::missing_field("name"))?;
                Ok(Skill { name, level })
            }
        }

        deserializer.deserialize_any(SkillVisitor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn core_fields_are_sorted_and_unique() {
        assert!(
            CORE_FIELDS.windows(2).all(|pair| pair[0].key < pair[1].key),
            "CORE_FIELDS must stay strictly sorted for binary search; \
             sorting a copy first would hide a misordered table"
        );
    }

    #[test]
    fn core_field_finds_keys_in_any_case() {
        assert_eq!(core_field("photo").map(|field| field.label), Some("Photo"));
        assert_eq!(core_field("Photo").map(|field| field.key), Some("photo"));
        assert_eq!(core_field("unknown-field"), None);
    }

    #[test]
    fn normalize_key_folds_case_and_separators() {
        assert_eq!(normalize_key("Address Furigana"), "address-furigana");
        assert_eq!(normalize_key("postal_code"), "postal-code");
        assert_eq!(normalize_key("  NAME  "), "name");
        assert_eq!(normalize_key("commute_time"), "commute-time");
        assert_eq!(normalize_key("a - b"), "a-b");
    }

    #[test]
    fn field_value_keeps_strings_as_text() {
        // Integers that look like years and the word "present" are text;
        // guessing dates broke round-trip identity for open fields.
        let value: FieldValue = serde_json::from_str("\"8001\"").unwrap();
        assert_eq!(value, FieldValue::Text("8001".into()));
        let value: FieldValue = serde_json::from_str("\"present\"").unwrap();
        assert_eq!(value, FieldValue::Text("present".into()));
        let value: FieldValue = serde_json::from_str("3.9").unwrap();
        assert_eq!(value, FieldValue::Text("3.9".into()));
    }

    #[test]
    fn field_map_rejects_empty_keys() {
        let result = serde_json::from_str::<FieldMap>(r#"{"": "v"}"#);
        assert!(result.is_err());
    }

    #[test]
    fn field_map_rejects_markup_keys() {
        // Defense in depth for the renderers: a key is plain text data and
        // never carries markup into any output.
        let result = serde_json::from_str::<FieldMap>(r#"{"<b>bold</b>": "v"}"#);
        assert!(result.is_err());
        // Unicode keys keep working.
        let ok: FieldMap = serde_json::from_str(r#"{"学歴": "v"}"#).unwrap();
        assert_eq!(ok.len(), 1);
    }

    #[test]
    fn skill_level_rejects_out_of_range_numbers() {
        assert_eq!(SkillLevel::parse("6"), None);
        assert_eq!(SkillLevel::parse("-1"), None);
        assert_eq!(SkillLevel::parse("10/10"), None);
        assert_eq!(SkillLevel::parse("4/5"), Some(SkillLevel::Numeric(4)));
        assert_eq!(
            SkillLevel::parse("Expert"),
            Some(SkillLevel::Named("Expert".into()))
        );
        // TOML integers arrive as i64.
        let level: SkillLevel = serde_json::from_str("4").unwrap();
        assert_eq!(level, SkillLevel::Numeric(4));
        assert!(serde_json::from_str::<SkillLevel>("-1").is_err());
    }

    #[test]
    fn field_map_preserves_insertion_order() {
        let mut map = FieldMap::new();
        map.insert("GPA", FieldValue::Text("3.9".into()));
        map.insert("Furigana", FieldValue::Text("みょうじ なまえ".into()));
        map.insert("Dependents", FieldValue::Integer(2));
        let keys: Vec<&str> = map.iter().map(|field| field.key.as_str()).collect();
        assert_eq!(keys, ["gpa", "furigana", "dependents"]);
    }

    #[test]
    fn field_map_replace_keeps_position() {
        let mut map = FieldMap::new();
        map.insert("gpa", FieldValue::Text("3.9".into()));
        map.insert("note", FieldValue::Text("first".into()));
        map.insert("gpa", FieldValue::Text("4.0".into()));
        let values: Vec<String> = map.iter().map(|field| field.value.as_text()).collect();
        assert_eq!(values, ["4.0", "first"]);
        assert_eq!(map.len(), 2);
    }

    #[test]
    fn field_map_json_round_trip_is_ordered() {
        let mut map = FieldMap::new();
        map.insert("Zeta", FieldValue::Text("last".into()));
        map.insert("Alpha", FieldValue::Text("first".into()));
        map.insert("Count", FieldValue::Integer(3));
        map.insert("Willing", FieldValue::Boolean(true));
        // Serialize through the string path: `to_value` would route through
        // serde_json's sorted `Map`, which is a serde_json artifact, not ours.
        let json = serde_json::to_string(&map).unwrap();
        let positions: Vec<usize> = ["zeta", "alpha", "count", "willing"]
            .iter()
            .map(|key| json.find(key).expect("key must appear"))
            .collect();
        let mut sorted = positions.clone();
        sorted.sort();
        assert_eq!(
            positions, sorted,
            "insertion order must survive serialization"
        );
        let restored: FieldMap = serde_json::from_str(&json).unwrap();
        assert_eq!(restored, map);
    }

    #[test]
    fn field_value_rejects_non_scalars() {
        let result = serde_json::from_str::<FieldValue>(r#"{"nested": true}"#);
        assert!(result.is_err());
    }

    #[test]
    fn skill_level_parses_scales() {
        assert_eq!(SkillLevel::parse("4/5"), Some(SkillLevel::Numeric(4)));
        assert_eq!(SkillLevel::parse("5"), Some(SkillLevel::Numeric(5)));
        assert_eq!(
            SkillLevel::parse("Expert"),
            Some(SkillLevel::Named("Expert".into()))
        );
        assert_eq!(
            SkillLevel::parse("C1"),
            Some(SkillLevel::Named("C1".into()))
        );
        assert_eq!(SkillLevel::parse("6"), None);
        assert_eq!(SkillLevel::parse("  "), None);
    }

    #[test]
    fn skill_level_serde_round_trip() {
        let numeric: SkillLevel = serde_json::from_str("4").unwrap();
        assert_eq!(numeric, SkillLevel::Numeric(4));
        assert_eq!(serde_json::to_string(&numeric).unwrap(), "4");
        let named: SkillLevel = serde_json::from_str("\"Expert\"").unwrap();
        assert_eq!(named, SkillLevel::Named("Expert".into()));
        assert_eq!(serde_json::to_string(&named).unwrap(), "\"Expert\"");
        assert!(serde_json::from_str::<SkillLevel>("6").is_err());
    }

    #[test]
    fn skill_deserializes_from_plain_string() {
        let skill: Skill = serde_json::from_str("\"Rust\"").unwrap();
        assert_eq!(
            skill,
            Skill {
                name: RichText::literal("Rust"),
                level: None
            }
        );
        assert_eq!(serde_json::to_string(&skill).unwrap(), "\"Rust\"");
    }

    #[test]
    fn skill_with_level_round_trips_through_a_map() {
        let json = r#"{"name": "Rust", "level": 4}"#;
        let skill: Skill = serde_json::from_str(json).unwrap();
        assert_eq!(skill.level, Some(SkillLevel::Numeric(4)));
        let restored: Skill =
            serde_json::from_str(&serde_json::to_string(&skill).unwrap()).unwrap();
        assert_eq!(restored, skill);
        assert!(serde_json::from_str::<Skill>(r#"{"level": 4}"#).is_err());
        assert!(serde_json::from_str::<Skill>(r#"{"name": "Rust", "extra": 1}"#).is_err());
    }
}
