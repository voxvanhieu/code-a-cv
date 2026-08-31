use cac_core::{
    CvDocument, DatePoint, EducationEntry, Entry, EntryKind, ExperienceEntry, Origin, Period,
    Profile, RichText, Section, SectionKind, SkillGroupEntry, TagSet,
};
use serde::Deserialize;
use serde_json::json;
use url::Url;

use crate::{ParseError, parse_date_point};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct JsonResume {
    #[serde(default)]
    basics: JsonBasics,
    #[serde(default)]
    work: Vec<JsonWork>,
    #[serde(default)]
    education: Vec<JsonEducation>,
    #[serde(default)]
    skills: Vec<JsonSkill>,
}

#[derive(Default, Deserialize)]
struct JsonBasics {
    #[serde(default)]
    name: String,
    email: Option<String>,
    phone: Option<String>,
    location: Option<JsonLocation>,
    url: Option<Url>,
    summary: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct JsonLocation {
    city: Option<String>,
    region: Option<String>,
    country_code: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct JsonWork {
    name: String,
    position: String,
    start_date: Option<String>,
    end_date: Option<String>,
    #[serde(default)]
    highlights: Vec<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct JsonEducation {
    institution: String,
    #[serde(default)]
    study_type: String,
    #[serde(default)]
    area: String,
    start_date: Option<String>,
    end_date: Option<String>,
    #[serde(default)]
    courses: Vec<String>,
}

#[derive(Deserialize)]
struct JsonSkill {
    name: String,
    #[serde(default)]
    keywords: Vec<String>,
}

pub fn import_json_resume(source: &str) -> Result<CvDocument, ParseError> {
    let resume: JsonResume = serde_json::from_str(source).map_err(|error| ParseError::Json {
        path: "jsonresume".into(),
        message: error.to_string(),
    })?;
    let location = resume.basics.location.and_then(|value| {
        let parts: Vec<String> = [value.city, value.region, value.country_code]
            .into_iter()
            .flatten()
            .filter(|value| !value.is_empty())
            .collect();
        (!parts.is_empty()).then(|| parts.join(", "))
    });
    let profile = Profile {
        name: resume.basics.name,
        email: resume.basics.email,
        phone: resume.basics.phone,
        location,
        website: resume.basics.url,
        summary: resume.basics.summary.as_deref().map(RichText::parse),
    };
    let mut sections = Vec::new();
    if !resume.work.is_empty() {
        sections.push(Section {
            id: "experience".into(),
            title: "Experience".into(),
            kind: SectionKind::Experience,
            tags: TagSet::new(),
            entries: resume
                .work
                .into_iter()
                .map(|value| Entry {
                    kind: EntryKind::Experience(ExperienceEntry {
                        role: RichText::parse(&value.position),
                        organization: RichText::parse(&value.name),
                        location: None,
                        period: json_period(value.start_date, value.end_date),
                        highlights: value
                            .highlights
                            .iter()
                            .map(|value| RichText::parse(value))
                            .collect(),
                    }),
                    tags: TagSet::new(),
                    origin: Origin::default(),
                })
                .collect(),
        });
    }
    if !resume.education.is_empty() {
        sections.push(Section {
            id: "education".into(),
            title: "Education".into(),
            kind: SectionKind::Education,
            tags: TagSet::new(),
            entries: resume
                .education
                .into_iter()
                .map(|value| {
                    let qualification = [value.study_type, value.area]
                        .into_iter()
                        .filter(|value| !value.is_empty())
                        .collect::<Vec<_>>()
                        .join(" in ");
                    Entry {
                        kind: EntryKind::Education(EducationEntry {
                            qualification: RichText::parse(&qualification),
                            institution: RichText::parse(&value.institution),
                            period: json_period(value.start_date, value.end_date),
                            highlights: value
                                .courses
                                .iter()
                                .map(|value| RichText::parse(value))
                                .collect(),
                        }),
                        tags: TagSet::new(),
                        origin: Origin::default(),
                    }
                })
                .collect(),
        });
    }
    if !resume.skills.is_empty() {
        sections.push(Section {
            id: "skills".into(),
            title: "Skills".into(),
            kind: SectionKind::Skills,
            tags: TagSet::new(),
            entries: resume
                .skills
                .into_iter()
                .map(|value| Entry {
                    kind: EntryKind::SkillGroup(SkillGroupEntry {
                        name: RichText::parse(&value.name),
                        skills: value
                            .keywords
                            .iter()
                            .map(|value| RichText::parse(value))
                            .collect(),
                    }),
                    tags: TagSet::new(),
                    origin: Origin::default(),
                })
                .collect(),
        });
    }
    Ok(CvDocument { profile, sections })
}

fn json_period(start: Option<String>, end: Option<String>) -> Option<Period> {
    let start = start.as_deref().and_then(parse_date_point)?;
    let end = end
        .as_deref()
        .and_then(parse_date_point)
        .unwrap_or(DatePoint::Present);
    Period::new(start, end).ok()
}

pub fn export_json_resume(cv: &CvDocument) -> serde_json::Value {
    let work: Vec<_> = cv
        .sections
        .iter()
        .flat_map(|section| &section.entries)
        .filter_map(|entry| match &entry.kind {
            EntryKind::Experience(value) => Some(json!({
                "name": value.organization.plain(), "position": value.role.plain(),
                "startDate": value.period.as_ref().map(|period| period.start.to_string()),
                "endDate": value.period.as_ref().map(|period| period.end.to_string()),
                "highlights": value.highlights.iter().map(RichText::plain).collect::<Vec<_>>()
            })),
            _ => None,
        })
        .collect();
    let education: Vec<_> = cv
        .sections
        .iter()
        .flat_map(|section| &section.entries)
        .filter_map(|entry| match &entry.kind {
            EntryKind::Education(value) => Some(json!({
                "institution": value.institution.plain(), "studyType": value.qualification.plain(),
                "startDate": value.period.as_ref().map(|period| period.start.to_string()),
                "endDate": value.period.as_ref().map(|period| period.end.to_string()),
                "courses": value.highlights.iter().map(RichText::plain).collect::<Vec<_>>()
            })),
            _ => None,
        })
        .collect();
    let skills: Vec<_> = cv.sections.iter().flat_map(|section| &section.entries).filter_map(|entry| match &entry.kind {
        EntryKind::SkillGroup(value) => Some(json!({ "name": value.name.plain(), "keywords": value.skills.iter().map(RichText::plain).collect::<Vec<_>>() })),
        _ => None,
    }).collect();
    json!({
        "$schema": "https://raw.githubusercontent.com/jsonresume/resume-schema/master/schema.json",
        "basics": {
            "name": cv.profile.name, "email": cv.profile.email, "phone": cv.profile.phone,
            "url": cv.profile.website, "summary": cv.profile.summary.as_ref().map(RichText::plain),
            "location": { "city": cv.profile.location }
        },
        "work": work, "education": education, "skills": skills
    })
}
