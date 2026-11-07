pub mod dates;
pub mod types;

use crate::format_date;
use cac_core::{CvDocument, Entry, EntryKind, SectionKind};
use serde::Serialize;

use types::{InlineView, rich_view};

#[derive(Serialize)]
struct RenderView {
    profile: ProfileView,
    sections: Vec<SectionView>,
}
#[derive(Serialize)]
struct ProfileView {
    name: String,
    contacts: Vec<ContactView>,
    summary: Option<Vec<InlineView>>,
    // Named slots (C3/V1): the extension fields ride in the view so themes
    // can read them without a model change. Every key is present; absent
    // values serialize as null.
    photo: Option<String>,
    headline: Option<String>,
    address: Option<cac_core::Address>,
    fields: cac_core::FieldMap,
}
#[derive(Serialize)]
struct SectionView {
    id: String,
    kind: SectionKind,
    title: String,
    entries: Vec<EntryView>,
    tags: cac_core::TagSet,
}
#[derive(Serialize)]
struct EntryView {
    kind: &'static str,
    primary: Vec<InlineView>,
    secondary: Option<Vec<InlineView>>,
    period: Option<String>,
    // Named slots (C3/V1): structured dates, scalar metadata, and open
    // fields ride beside the legacy keys for one release. Every key is
    // present; absent values serialize as null.
    structured_period: Option<cac_core::Period>,
    date: Option<cac_core::DatePoint>,
    location: Option<String>,
    url: Option<String>,
    fields: cac_core::FieldMap,
    tags: cac_core::TagSet,
    highlights: Vec<Vec<InlineView>>,
    metadata: Vec<MetadataView>,
}

fn entry_kind_name(kind: &EntryKind) -> &'static str {
    match kind {
        EntryKind::Experience(_) => "experience",
        EntryKind::Education(_) => "education",
        EntryKind::Project(_) => "project",
        EntryKind::Publication(_) => "publication",
        EntryKind::SkillGroup(_) => "skill-group",
        EntryKind::Custom(_) => "custom",
        EntryKind::Text(_) => "text",
        EntryKind::Prose(_) => "prose",
    }
}

fn entry_view(entry: &Entry) -> EntryView {
    let (primary, secondary) = entry.kind.heading();
    EntryView {
        kind: entry_kind_name(&entry.kind),
        primary: rich_view(primary),
        secondary: secondary.filter(|value| !value.is_empty()).map(rich_view),
        period: entry
            .kind
            .period()
            .map(|period| {
                format!(
                    "{} – {}",
                    period.start.as_ref().map(format_date).unwrap_or_default(),
                    period.end.as_ref().map(format_date).unwrap_or_default()
                )
            })
            .or_else(|| entry.kind.date().map(format_date)),
        structured_period: entry.kind.period().cloned(),
        date: entry.kind.date().cloned(),
        location: match &entry.kind {
            EntryKind::Experience(value) => value.location.clone(),
            _ => None,
        },
        url: match &entry.kind {
            EntryKind::Project(value) => value.url.as_ref().map(ToString::to_string),
            EntryKind::Publication(value) => value.url.as_ref().map(ToString::to_string),
            _ => None,
        },
        fields: entry.fields.clone(),
        tags: entry.tags.clone(),
        metadata: {
            let mut metadata = metadata_view(&entry.kind);
            if let Some(body) = &entry.content {
                metadata.push(MetadataView {
                    role: "description",
                    body: rich_view(body),
                });
            }
            metadata
        },
        highlights: entry.kind.highlights().iter().map(rich_view).collect(),
    }
}

fn entry_views(entries: &[Entry]) -> Vec<EntryView> {
    let mut views: Vec<EntryView> = Vec::new();
    for entry in entries {
        if let EntryKind::Text(value) = &entry.kind
            && let Some(previous) = views.last_mut()
            && previous.kind == "text"
        {
            previous.highlights.push(rich_view(&value.body));
            continue;
        }
        views.push(entry_view(entry));
    }
    views
}

impl From<&CvDocument> for RenderView {
    fn from(cv: &CvDocument) -> Self {
        let mut contacts: Vec<_> = [
            (
                "email",
                cv.profile.email.clone(),
                cv.profile.email.as_ref().map(|v| format!("mailto:{v}")),
            ),
            (
                "phone",
                cv.profile.phone.clone(),
                cv.profile.phone.as_ref().map(|v| format!("tel:{v}")),
            ),
            ("location", cv.profile.location.clone(), None),
            (
                "website",
                cv.profile.website.as_ref().map(ToString::to_string),
                cv.profile.website.as_ref().map(ToString::to_string),
            ),
        ]
        .into_iter()
        .filter_map(|(kind, label, href)| label.map(|label| ContactView { kind, label, href }))
        .collect();
        contacts.extend(cv.profile.contacts.iter().map(|contact| {
            ContactView {
                kind: contact
                    .href
                    .as_ref()
                    .map_or("custom", |url| match url.scheme() {
                        "mailto" => "email",
                        "tel" => "phone",
                        _ => "website",
                    }),
                label: contact.label.clone(),
                href: contact.href.as_ref().map(ToString::to_string),
            }
        }));
        Self {
            profile: ProfileView {
                name: cv.profile.name.clone(),
                contacts,
                summary: cv.profile.summary.as_ref().map(rich_view),
                photo: cv.profile.photo.clone(),
                headline: cv.profile.headline.clone(),
                address: cv.profile.address.clone(),
                fields: cv.profile.fields.clone(),
            },
            sections: cv
                .sections
                .iter()
                .map(|section| SectionView {
                    id: section.id.clone(),
                    kind: section.kind.clone(),
                    title: section.title.clone(),
                    entries: entry_views(&section.entries),
                    tags: section.tags.clone(),
                })
                .collect(),
        }
    }
}

#[derive(Serialize)]
struct ContactView {
    kind: &'static str,
    label: String,
    href: Option<String>,
}
#[derive(Serialize)]
struct MetadataView {
    role: &'static str,
    body: Vec<InlineView>,
}
fn metadata_view(kind: &EntryKind) -> Vec<MetadataView> {
    let mut metadata = Vec::new();
    if let EntryKind::Experience(value) = kind
        && let Some(location) = &value.location
    {
        metadata.push(MetadataView {
            role: "location",
            body: vec![InlineView::Text {
                text: location.clone(),
            }],
        });
    }
    let url = match kind {
        EntryKind::Project(value) => value.url.as_ref(),
        EntryKind::Publication(value) => value.url.as_ref(),
        _ => None,
    };
    if let Some(url) = url {
        metadata.push(MetadataView {
            role: "url",
            body: vec![InlineView::Link {
                href: url.to_string(),
                body: vec![InlineView::Text {
                    text: url.to_string(),
                }],
            }],
        });
    }
    metadata
}

pub(crate) fn render_view(cv: &CvDocument) -> Result<Vec<u8>, serde_json::Error> {
    serde_json::to_vec(&RenderView::from(cv))
}

/// The rendered view as a serializable value, for snapshot tests.
#[doc(hidden)]
pub fn test_render_view(cv: &CvDocument) -> serde_json::Value {
    serde_json::to_value(RenderView::from(cv)).expect("the view serializes")
}

#[cfg(test)]
#[path = "../tests/unit/view.rs"]
mod tests;
