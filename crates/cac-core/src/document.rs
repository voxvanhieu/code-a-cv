use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};
use url::Url;

use crate::fields::{EntryKindName, core_field};
use crate::{DatePoint, FieldMap, Period, RichText, SkillLevel};

pub type TagSet = BTreeSet<String>;

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CvDocument {
    pub profile: Profile,
    #[serde(default)]
    pub sections: Vec<Section>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Profile {
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub phone: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub location: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub website: Option<Url>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub summary: Option<RichText>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub contacts: Vec<Contact>,
    /// Project-relative path to a photo; the renderer resolves it inside the project jail.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub photo: Option<String>,
    /// A one-line headline under the name (JSON Resume `basics.label`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub headline: Option<String>,
    /// Structured postal address; themes choose which parts to show.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub address: Option<Address>,
    /// Open profile fields: author-written `Key: value` lines under the name
    /// the model does not type (a pack declares what they mean).
    #[serde(default, skip_serializing_if = "FieldMap::is_empty")]
    pub fields: FieldMap,
}

impl Profile {
    /// Whether the profile carries a value for `key`: the typed slot for a
    /// profile-scoped core field (a pack may require `photo` or `email` by
    /// its core name), the open fields for any other key.
    pub fn carries_core_field(&self, key: &str) -> bool {
        let present =
            |value: &Option<String>| value.as_ref().is_some_and(|value| !value.trim().is_empty());
        match core_field(key).map(|field| field.key) {
            Some("address") => self.address.is_some(),
            Some("email") => present(&self.email),
            Some("headline") => present(&self.headline),
            Some("location") => present(&self.location),
            Some("phone") => present(&self.phone),
            Some("photo") => present(&self.photo),
            Some("summary") => self.summary.as_ref().is_some_and(|v| !v.is_empty()),
            Some("website") => self.website.is_some(),
            _ => self.fields.get(key).is_some(),
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Address {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub street: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub city: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub region: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub postal_code: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub country: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Contact {
    pub label: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub href: Option<Url>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Section {
    pub id: String,
    pub title: String,
    #[serde(default)]
    pub kind: SectionKind,
    #[serde(default)]
    pub entries: Vec<Entry>,
    #[serde(default, skip_serializing_if = "TagSet::is_empty")]
    pub tags: TagSet,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum SectionKind {
    Education,
    Experience,
    Projects,
    Publications,
    Skills,
    #[default]
    Custom,
    Named(String),
}

impl SectionKind {
    pub fn is_valid(&self) -> bool {
        let value = self.as_str();
        !value.is_empty()
            && value
                .chars()
                .all(|c| c.is_alphanumeric() || matches!(c, '-' | '_' | '.'))
    }

    pub fn as_str(&self) -> &str {
        match self {
            Self::Education => "education",
            Self::Experience => "experience",
            Self::Projects => "projects",
            Self::Publications => "publications",
            Self::Skills => "skills",
            Self::Custom => "custom",
            Self::Named(value) => value,
        }
    }
}

impl Serialize for SectionKind {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for SectionKind {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = String::deserialize(deserializer)?;
        let kind = match value.as_str() {
            "education" => Self::Education,
            "experience" => Self::Experience,
            "projects" => Self::Projects,
            "publications" => Self::Publications,
            "skills" => Self::Skills,
            "custom" => Self::Custom,
            _ => Self::Named(value),
        };
        if !kind.is_valid() {
            return Err(serde::de::Error::custom(
                "section kind must be a nonempty identifier containing letters, numbers, hyphens, underscores, or dots",
            ));
        }
        Ok(kind)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Entry {
    #[serde(flatten)]
    pub kind: EntryKind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content: Option<RichText>,
    /// Open fields: author-written `Key: value` lines the model does not type.
    /// Themes document the keys they read; `cac check` validates against packs.
    #[serde(default, skip_serializing_if = "FieldMap::is_empty")]
    pub fields: FieldMap,
    #[serde(default, skip_serializing_if = "TagSet::is_empty")]
    pub tags: TagSet,
    #[serde(skip, default)]
    pub origin: Origin,
}

impl PartialEq for Entry {
    fn eq(&self, other: &Self) -> bool {
        self.kind == other.kind
            && self.content == other.content
            && self.fields == other.fields
            && self.tags == other.tags
    }
}

impl Eq for Entry {}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum EntryKind {
    Experience(ExperienceEntry),
    Education(EducationEntry),
    Project(ProjectEntry),
    Publication(PublicationEntry),
    SkillGroup(SkillGroupEntry),
    Custom(CustomEntry),
    Text(TextEntry),
    Prose(TextEntry),
    Language(LanguageEntry),
    Reference(ReferenceEntry),
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExperienceEntry {
    pub role: RichText,
    #[serde(default)]
    pub organization: RichText,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub location: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub period: Option<Period>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub date: Option<DatePoint>,
    #[serde(default)]
    pub highlights: Vec<RichText>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EducationEntry {
    pub qualification: RichText,
    #[serde(default)]
    pub institution: RichText,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub period: Option<Period>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub date: Option<DatePoint>,
    /// Free-text academic score, for example `3.9 GPA` (JSON Resume `education[].score`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub score: Option<String>,
    #[serde(default)]
    pub highlights: Vec<RichText>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProjectEntry {
    pub name: RichText,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<Url>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub period: Option<Period>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub date: Option<DatePoint>,
    #[serde(default)]
    pub highlights: Vec<RichText>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PublicationEntry {
    pub title: RichText,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub publisher: Option<RichText>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub date: Option<DatePoint>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<Url>,
    #[serde(default)]
    pub highlights: Vec<RichText>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SkillGroupEntry {
    pub name: RichText,
    #[serde(default)]
    pub skills: Vec<RichText>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CustomEntry {
    pub heading: RichText,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub period: Option<Period>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub date: Option<DatePoint>,
    #[serde(default)]
    pub highlights: Vec<RichText>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TextEntry {
    pub body: RichText,
}

/// A spoken language with an optional proficiency level (CEFR grade or a
/// named scale). The payload for the `language` entry kind.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LanguageEntry {
    pub name: RichText,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub proficiency: Option<SkillLevel>,
}

/// A reference with contact details (JSON Resume `references[]`). The payload
/// for the `reference` entry kind.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReferenceEntry {
    pub name: RichText,
    #[serde(default)]
    pub organization: Option<RichText>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub phone: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

impl EntryKind {
    pub fn heading(&self) -> (&RichText, Option<&RichText>) {
        match self {
            Self::Experience(value) => (&value.role, Some(&value.organization)),
            Self::Education(value) => (&value.qualification, Some(&value.institution)),
            Self::Project(value) => (&value.name, None),
            Self::Publication(value) => (&value.title, value.publisher.as_ref()),
            Self::SkillGroup(value) => (&value.name, None),
            Self::Custom(value) => (&value.heading, None),
            Self::Text(value) | Self::Prose(value) => (&value.body, None),
            Self::Language(value) => (&value.name, None),
            Self::Reference(value) => (&value.name, value.organization.as_ref()),
        }
    }

    pub fn period(&self) -> Option<&Period> {
        match self {
            Self::Experience(value) => value.period.as_ref(),
            Self::Education(value) => value.period.as_ref(),
            Self::Project(value) => value.period.as_ref(),
            Self::Custom(value) => value.period.as_ref(),
            Self::Publication(_) | Self::SkillGroup(_) | Self::Text(_) | Self::Prose(_) => None,
            Self::Language(_) | Self::Reference(_) => None,
        }
    }

    pub fn date(&self) -> Option<&DatePoint> {
        match self {
            Self::Experience(value) => value.date.as_ref(),
            Self::Education(value) => value.date.as_ref(),
            Self::Project(value) => value.date.as_ref(),
            Self::Custom(value) => value.date.as_ref(),
            Self::Publication(value) => value.date.as_ref(),
            Self::Language(_) | Self::Reference(_) => None,
            _ => None,
        }
    }

    pub fn highlights(&self) -> &[RichText] {
        match self {
            Self::Experience(value) => &value.highlights,
            Self::Education(value) => &value.highlights,
            Self::Project(value) => &value.highlights,
            Self::Publication(value) => &value.highlights,
            Self::SkillGroup(value) => &value.skills,
            Self::Custom(value) => &value.highlights,
            Self::Text(_) | Self::Prose(_) => &[],
            Self::Language(_) | Self::Reference(_) => &[],
        }
    }

    /// The canonical kind name the core field table uses for this entry.
    pub fn kind_name(&self) -> EntryKindName {
        match self {
            Self::Experience(_) => EntryKindName::Experience,
            Self::Education(_) => EntryKindName::Education,
            Self::Project(_) => EntryKindName::Project,
            Self::Publication(_) => EntryKindName::Publication,
            Self::SkillGroup(_) => EntryKindName::SkillGroup,
            Self::Custom(_) => EntryKindName::Custom,
            Self::Text(_) | Self::Prose(_) => EntryKindName::Text,
            Self::Language(_) => EntryKindName::Language,
            Self::Reference(_) => EntryKindName::Reference,
        }
    }
}

impl Entry {
    /// Whether the entry carries a value for the core field `key`: the
    /// entry's kind must be one of the field's declared entry scopes, and
    /// the typed slot (`role`, `period`, `tags`, `content`, …) must be
    /// present. Open fields (`self.fields`) are not consulted; callers
    /// combine the two.
    pub fn carries_core_field(&self, key: &str) -> bool {
        let Some(field) = core_field(key) else {
            return false;
        };
        let name = self.kind.kind_name();
        if !field.applies_to_entry_kind(name) {
            return false;
        }
        match field.key {
            "body" => match &self.kind {
                EntryKind::Text(value) | EntryKind::Prose(value) => !value.body.is_empty(),
                _ => false,
            },
            "content" => self.content.as_ref().is_some_and(|value| !value.is_empty()),
            "date" => self.kind.date().is_some(),
            "heading" => {
                matches!(&self.kind, EntryKind::Custom(value) if !value.heading.is_empty())
            }
            "highlights" => !self.kind.highlights().is_empty(),
            "institution" => {
                matches!(&self.kind, EntryKind::Education(value) if !value.institution.is_empty())
            }
            "location" => matches!(&self.kind, EntryKind::Experience(value) if value
                .location
                .as_ref()
                .is_some_and(|value| !value.trim().is_empty())),
            "name" => match &self.kind {
                EntryKind::Project(value) => !value.name.is_empty(),
                EntryKind::SkillGroup(value) => !value.name.is_empty(),
                EntryKind::Language(value) => !value.name.is_empty(),
                EntryKind::Reference(value) => !value.name.is_empty(),
                _ => false,
            },
            "organization" => {
                matches!(&self.kind, EntryKind::Experience(value) if !value.organization.is_empty())
            }
            "period" => self.kind.period().is_some(),
            "proficiency" => {
                matches!(&self.kind, EntryKind::Language(value) if value.proficiency.is_some())
            }
            "publisher" => matches!(&self.kind, EntryKind::Publication(value) if value
                .publisher
                .as_ref()
                .is_some_and(|value| !value.is_empty())),
            "qualification" => {
                matches!(&self.kind, EntryKind::Education(value) if !value.qualification.is_empty())
            }
            "role" => matches!(&self.kind, EntryKind::Experience(value) if !value.role.is_empty()),
            "score" => matches!(&self.kind, EntryKind::Education(value) if value
                .score
                .as_ref()
                .is_some_and(|value| !value.trim().is_empty())),
            "skills" => {
                matches!(&self.kind, EntryKind::SkillGroup(value) if !value.skills.is_empty())
            }
            "tags" => !self.tags.is_empty(),
            "title" => {
                matches!(&self.kind, EntryKind::Publication(value) if !value.title.is_empty())
            }
            "url" => match &self.kind {
                EntryKind::Project(value) => value.url.is_some(),
                EntryKind::Publication(value) => value.url.is_some(),
                _ => false,
            },
            // `id` and `kind` are section-scoped; `level` lives on skill
            // items — no entry can carry them.
            _ => false,
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Origin {
    pub path: String,
}

impl CvDocument {
    /// Project the document onto a tagged variant: sections whose tags
    /// intersect `tags` survive whole, untagged sections survive only
    /// through their tagged entries, and everything else drops. The
    /// profile always survives; a variant without the name block is not a
    /// CV. Returns `None` when nothing matches, so callers can report a
    /// diagnostic instead of writing an empty document.
    pub fn variant(&self, tags: &TagSet) -> Option<CvDocument> {
        let sections: Vec<Section> = self
            .sections
            .iter()
            .filter_map(|section| {
                if !section.tags.is_disjoint(tags) {
                    return Some(section.clone());
                }
                let entries: Vec<Entry> = section
                    .entries
                    .iter()
                    .filter(|entry| !entry.tags.is_disjoint(tags))
                    .cloned()
                    .collect();
                (!entries.is_empty()).then_some(Section {
                    entries,
                    ..section.clone()
                })
            })
            .collect();
        if sections.is_empty() {
            return None;
        }
        Some(CvDocument {
            profile: self.profile.clone(),
            sections,
        })
    }
}

#[cfg(test)]
#[path = "../tests/unit/variant.rs"]
mod variant_tests;
