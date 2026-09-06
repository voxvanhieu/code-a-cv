use cac_core::{
    CvDocument, EducationEntry, Entry, EntryKind, ExperienceEntry, Origin, Period, Profile,
    RichText, Section, SectionKind, SkillGroupEntry, TagSet,
};
use serde::Deserialize;
use serde_json::json;
use url::Url;

use crate::{ParseError, parse_date_point};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct JsonResume {
    #[serde(rename = "$schema")]
    _schema: Option<String>,
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
#[serde(deny_unknown_fields)]
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
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct JsonLocation {
    city: Option<String>,
    region: Option<String>,
    country_code: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct JsonWork {
    name: String,
    position: String,
    start_date: Option<String>,
    end_date: Option<String>,
    #[serde(default)]
    highlights: Vec<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
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
#[serde(deny_unknown_fields)]
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
        summary: resume.basics.summary.as_deref().map(RichText::literal),
        contacts: Vec::new(),
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
                .map(|value| -> Result<Entry, ParseError> {
                    Ok(Entry {
                        kind: EntryKind::Experience(ExperienceEntry {
                            role: RichText::literal(&value.position),
                            organization: RichText::literal(&value.name),
                            location: None,
                            period: json_period(value.start_date, value.end_date)?,
                            date: None,
                            highlights: value
                                .highlights
                                .iter()
                                .map(|value| RichText::literal(value))
                                .collect(),
                        }),
                        tags: TagSet::new(),
                        content: None,
                        origin: Origin::default(),
                    })
                })
                .collect::<Result<Vec<_>, _>>()?,
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
                .map(|value| -> Result<Entry, ParseError> {
                    let qualification = [value.study_type, value.area]
                        .into_iter()
                        .filter(|value| !value.is_empty())
                        .collect::<Vec<_>>()
                        .join(" in ");
                    Ok(Entry {
                        kind: EntryKind::Education(EducationEntry {
                            qualification: RichText::literal(&qualification),
                            institution: RichText::literal(&value.institution),
                            period: json_period(value.start_date, value.end_date)?,
                            date: None,
                            highlights: value
                                .courses
                                .iter()
                                .map(|value| RichText::literal(value))
                                .collect(),
                        }),
                        tags: TagSet::new(),
                        content: None,
                        origin: Origin::default(),
                    })
                })
                .collect::<Result<Vec<_>, _>>()?,
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
                        name: RichText::literal(&value.name),
                        skills: value
                            .keywords
                            .iter()
                            .map(|value| RichText::literal(value))
                            .collect(),
                    }),
                    tags: TagSet::new(),
                    content: None,
                    origin: Origin::default(),
                })
                .collect(),
        });
    }
    Ok(CvDocument { profile, sections })
}

fn json_period(start: Option<String>, end: Option<String>) -> Result<Option<Period>, ParseError> {
    let point = |value: Option<String>, field: &str| {
        value
            .filter(|value| !value.trim().is_empty())
            .map(|value| {
                parse_date_point(&value).ok_or_else(|| {
                    ParseError::Validation(format!("invalid JSON Resume {field} `{value}`"))
                })
            })
            .transpose()
    };
    let start = point(start, "startDate")?;
    let end = point(end, "endDate")?;
    if start.is_none() && end.is_none() {
        return Ok(None);
    }
    Period::partial(start, end)
        .map(Some)
        .map_err(|error| ParseError::Validation(error.to_string()))
}

pub fn export_json_resume(cv: &CvDocument) -> serde_json::Value {
    let work: Vec<_> = cv
        .sections
        .iter()
        .flat_map(|section| &section.entries)
        .filter_map(|entry| match &entry.kind {
            EntryKind::Experience(value) => Some(json!({
                "name": value.organization.plain(), "position": value.role.plain(),
                "startDate": value.period.as_ref().map(|period| period.start.as_ref().map(ToString::to_string)),
                "endDate": value.period.as_ref().map(|period| period.end.as_ref().map(ToString::to_string)),
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
                "startDate": value.period.as_ref().map(|period| period.start.as_ref().map(ToString::to_string)),
                "endDate": value.period.as_ref().map(|period| period.end.as_ref().map(ToString::to_string)),
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

pub fn export_json_resume_checked(cv: &CvDocument) -> Result<serde_json::Value, ParseError> {
    crate::validate(cv)?;
    let output = export_json_resume(cv);
    let reparsed = crate::parse(&output.to_string(), crate::InputFormat::JsonResume)?;
    if &reparsed != cv {
        return Err(ParseError::Validation(format!(
            "JSON Resume conversion cannot preserve {}; use Markdown, JSON, YAML, or TOML instead",
            crate::markdown::difference(
                &serde_json::to_value(cv).unwrap(),
                &serde_json::to_value(reparsed).unwrap(),
                "cv"
            )
        )));
    }
    Ok(output)
}
