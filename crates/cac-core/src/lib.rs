use std::cmp::Ordering;
use std::collections::BTreeSet;
use std::fmt::{self, Display, Formatter};

use chrono::{Datelike, NaiveDate};
use pulldown_cmark::{Event, Options, Parser, Tag, TagEnd};
use schemars::JsonSchema;
use serde::de::{self, Visitor};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use url::Url;

pub type TagSet = BTreeSet<String>;
pub type ResolvedCv = CvDocument;

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct CvDocument {
    pub profile: Profile,
    #[serde(default)]
    pub sections: Vec<Section>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
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

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
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

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
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

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct Entry {
    #[serde(flatten)]
    pub kind: EntryKind,
    #[serde(default, skip_serializing_if = "TagSet::is_empty")]
    pub tags: TagSet,
    #[serde(skip, default)]
    #[schemars(skip)]
    pub origin: Origin,
}

impl PartialEq for Entry {
    fn eq(&self, other: &Self) -> bool {
        self.kind == other.kind && self.tags == other.tags
    }
}

impl Eq for Entry {}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum EntryKind {
    Experience(ExperienceEntry),
    Education(EducationEntry),
    Project(ProjectEntry),
    Publication(PublicationEntry),
    SkillGroup(SkillGroupEntry),
    Text(TextEntry),
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
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

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct EducationEntry {
    pub qualification: RichText,
    pub institution: RichText,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub period: Option<Period>,
    #[serde(default)]
    pub highlights: Vec<RichText>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
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

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
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

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SkillGroupEntry {
    pub name: RichText,
    #[serde(default)]
    pub skills: Vec<RichText>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
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
            Self::Text(value) => (&value.body, None),
        }
    }

    pub fn period(&self) -> Option<&Period> {
        match self {
            Self::Experience(value) => value.period.as_ref(),
            Self::Education(value) => value.period.as_ref(),
            Self::Project(value) => value.period.as_ref(),
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
            Self::Text(_) => &[],
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Origin {
    pub path: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Period {
    pub start: DatePoint,
    pub end: DatePoint,
}

impl Period {
    pub fn new(start: DatePoint, end: DatePoint) -> Result<Self, PeriodError> {
        if end != DatePoint::Present && start.sort_key() > end.sort_key() {
            return Err(PeriodError { start, end });
        }
        Ok(Self { start, end })
    }

    pub fn is_valid(&self) -> bool {
        self.end == DatePoint::Present || self.start.sort_key() <= self.end.sort_key()
    }
}

#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[error("period starts at {start} after it ends at {end}")]
pub struct PeriodError {
    pub start: DatePoint,
    pub end: DatePoint,
}

#[derive(Clone, Debug, PartialEq, Eq, JsonSchema)]
#[schemars(with = "String")]
pub enum DatePoint {
    Year(i32),
    YearMonth(i32, u8),
    Full(NaiveDate),
    Present,
}

impl DatePoint {
    pub fn year_month(year: i32, month: u8) -> Option<Self> {
        (1..=12)
            .contains(&month)
            .then_some(Self::YearMonth(year, month))
    }

    pub fn granularity(&self) -> u8 {
        match self {
            Self::Year(_) => 1,
            Self::YearMonth(_, _) => 2,
            Self::Full(_) => 3,
            Self::Present => 0,
        }
    }

    fn sort_key(&self) -> (i32, u32, u32) {
        match self {
            Self::Year(year) => (*year, 1, 1),
            Self::YearMonth(year, month) => (*year, u32::from(*month), 1),
            Self::Full(date) => (date.year(), date.month(), date.day()),
            Self::Present => (i32::MAX, 12, 31),
        }
    }
}

impl Display for DatePoint {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Year(value) => write!(formatter, "{value}"),
            Self::YearMonth(year, month) => write!(formatter, "{year:04}-{month:02}"),
            Self::Full(value) => write!(formatter, "{value}"),
            Self::Present => formatter.write_str("Present"),
        }
    }
}

impl Serialize for DatePoint {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for DatePoint {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct DatePointVisitor;
        impl Visitor<'_> for DatePointVisitor {
            type Value = DatePoint;
            fn expecting(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
                formatter.write_str("a year, YYYY-MM, YYYY-MM-DD, or `present`")
            }
            fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                i32::try_from(value).map(DatePoint::Year).map_err(E::custom)
            }
            fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                i32::try_from(value).map(DatePoint::Year).map_err(E::custom)
            }
            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                if value.eq_ignore_ascii_case("present") {
                    return Ok(DatePoint::Present);
                }
                if let Ok(year) = value.parse::<i32>() {
                    return Ok(DatePoint::Year(year));
                }
                if let Ok(date) = NaiveDate::parse_from_str(value, "%Y-%m-%d") {
                    return Ok(DatePoint::Full(date));
                }
                if let Some((year, month)) = value.split_once('-')
                    && let (Ok(year), Ok(month)) = (year.parse(), month.parse())
                    && let Some(point) = DatePoint::year_month(year, month)
                {
                    return Ok(point);
                }
                Err(E::custom(format!("invalid date point `{value}`")))
            }
        }
        deserializer.deserialize_any(DatePointVisitor)
    }
}

impl PartialOrd for DatePoint {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.sort_key().cmp(&other.sort_key()))
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, JsonSchema)]
#[schemars(with = "String")]
pub struct RichText(pub Vec<Inline>);

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Inline {
    Text(String),
    Emph(Vec<Inline>),
    Strong(Vec<Inline>),
    Code(String),
    Link { href: Url, body: Vec<Inline> },
}

impl RichText {
    pub fn parse(markdown: &str) -> Self {
        let mut roots = Vec::new();
        let mut stack: Vec<(InlineKind, Vec<Inline>)> = Vec::new();
        for event in Parser::new_ext(markdown, Options::empty()) {
            match event {
                Event::Start(Tag::Emphasis) => stack.push((InlineKind::Emph, Vec::new())),
                Event::Start(Tag::Strong) => stack.push((InlineKind::Strong, Vec::new())),
                Event::Start(Tag::Link { dest_url, .. }) => {
                    stack.push((InlineKind::Link(dest_url.into_string()), Vec::new()))
                }
                Event::End(TagEnd::Emphasis | TagEnd::Strong | TagEnd::Link) => {
                    if let Some((kind, body)) = stack.pop() {
                        let node = match kind {
                            InlineKind::Emph => Inline::Emph(body),
                            InlineKind::Strong => Inline::Strong(body),
                            InlineKind::Link(href) => Url::parse(&href)
                                .map(|href| Inline::Link {
                                    href,
                                    body: body.clone(),
                                })
                                .unwrap_or_else(|_| Inline::Text(plain_inlines(&body))),
                        };
                        push_inline(&mut roots, &mut stack, node);
                    }
                }
                Event::Text(value) => {
                    push_inline(&mut roots, &mut stack, Inline::Text(value.into_string()))
                }
                Event::Code(value) => {
                    push_inline(&mut roots, &mut stack, Inline::Code(value.into_string()))
                }
                Event::SoftBreak | Event::HardBreak => {
                    push_inline(&mut roots, &mut stack, Inline::Text(" ".into()))
                }
                _ => {}
            }
        }
        Self(roots)
    }

    pub fn plain(&self) -> String {
        plain_inlines(&self.0)
    }

    pub fn is_empty(&self) -> bool {
        self.plain().trim().is_empty()
    }

    pub fn to_markdown(&self) -> String {
        fn render(nodes: &[Inline], output: &mut String) {
            for node in nodes {
                match node {
                    Inline::Text(value) => output.push_str(value),
                    Inline::Emph(body) => {
                        output.push('*');
                        render(body, output);
                        output.push('*');
                    }
                    Inline::Strong(body) => {
                        output.push_str("**");
                        render(body, output);
                        output.push_str("**");
                    }
                    Inline::Code(value) => {
                        output.push('`');
                        output.push_str(value);
                        output.push('`');
                    }
                    Inline::Link { href, body } => {
                        output.push('[');
                        render(body, output);
                        output.push_str("](");
                        output.push_str(href.as_str());
                        output.push(')');
                    }
                }
            }
        }
        let mut output = String::new();
        render(&self.0, &mut output);
        output
    }
}

impl From<&str> for RichText {
    fn from(value: &str) -> Self {
        Self::parse(value)
    }
}

impl Display for RichText {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.plain())
    }
}

impl Serialize for RichText {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_markdown())
    }
}

impl<'de> Deserialize<'de> for RichText {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        String::deserialize(deserializer).map(|value| Self::parse(&value))
    }
}

enum InlineKind {
    Emph,
    Strong,
    Link(String),
}

fn push_inline(roots: &mut Vec<Inline>, stack: &mut [(InlineKind, Vec<Inline>)], node: Inline) {
    if let Some((_, body)) = stack.last_mut() {
        body.push(node);
    } else {
        roots.push(node);
    }
}

fn plain_inlines(nodes: &[Inline]) -> String {
    let mut output = String::new();
    for node in nodes {
        match node {
            Inline::Text(value) | Inline::Code(value) => output.push_str(value),
            Inline::Emph(body) | Inline::Strong(body) | Inline::Link { body, .. } => {
                output.push_str(&plain_inlines(body))
            }
        }
    }
    output
}
