use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};
use url::Url;

use crate::{DatePoint, Period, RichText};

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

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SectionKind {
    Education,
    Experience,
    Projects,
    Publications,
    Skills,
    #[default]
    Custom,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Entry {
    #[serde(flatten)]
    pub kind: EntryKind,
    #[serde(default, skip_serializing_if = "TagSet::is_empty")]
    pub tags: TagSet,
    #[serde(skip, default)]
    pub origin: Origin,
}

impl PartialEq for Entry {
    fn eq(&self, other: &Self) -> bool {
        self.kind == other.kind && self.tags == other.tags
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
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExperienceEntry {
    pub role: RichText,
    pub organization: RichText,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub location: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub period: Option<Period>,
    #[serde(default)]
    pub highlights: Vec<RichText>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EducationEntry {
    pub qualification: RichText,
    pub institution: RichText,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub period: Option<Period>,
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
    #[serde(default)]
    pub highlights: Vec<RichText>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TextEntry {
    pub body: RichText,
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
            Self::Text(value) => (&value.body, None),
        }
    }

    pub fn period(&self) -> Option<&Period> {
        match self {
            Self::Experience(value) => value.period.as_ref(),
            Self::Education(value) => value.period.as_ref(),
            Self::Project(value) => value.period.as_ref(),
            Self::Custom(value) => value.period.as_ref(),
            Self::Publication(_) | Self::SkillGroup(_) | Self::Text(_) => None,
        }
    }

    pub fn date(&self) -> Option<&DatePoint> {
        match self {
            Self::Publication(value) => value.date.as_ref(),
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
            Self::Text(_) => &[],
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Origin {
    pub path: String,
}
