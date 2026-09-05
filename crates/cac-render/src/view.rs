use crate::format_date;
use cac_core::{CvDocument, Entry, EntryKind, Inline, RichText, SectionKind};
use serde::Serialize;

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
}
#[derive(Serialize)]
struct SectionView {
    id: String,
    kind: SectionKind,
    title: String,
    entries: Vec<EntryView>,
}
#[derive(Serialize)]
struct EntryView {
    kind: &'static str,
    primary: Vec<InlineView>,
    secondary: Option<Vec<InlineView>>,
    period: Option<String>,
    highlights: Vec<Vec<InlineView>>,
    metadata: Vec<MetadataView>,
}
#[derive(Serialize)]
#[serde(tag = "kind", rename_all = "kebab-case")]
enum InlineView {
    Text { text: String },
    Emph { body: Vec<InlineView> },
    Strong { body: Vec<InlineView> },
    Code { text: String },
    Link { href: String, body: Vec<InlineView> },
}

fn inline_view(nodes: &[Inline]) -> Vec<InlineView> {
    nodes
        .iter()
        .map(|node| match node {
            Inline::Text(text) => InlineView::Text { text: text.clone() },
            Inline::Emph(body) => InlineView::Emph {
                body: inline_view(body),
            },
            Inline::Strong(body) => InlineView::Strong {
                body: inline_view(body),
            },
            Inline::Code(text) => InlineView::Code { text: text.clone() },
            Inline::Link { href, body } => InlineView::Link {
                href: href.to_string(),
                body: inline_view(body),
            },
        })
        .collect()
}

fn rich_view(value: &RichText) -> Vec<InlineView> {
    inline_view(&value.0)
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
                    format_date(&period.start),
                    format_date(&period.end)
                )
            })
            .or_else(|| entry.kind.date().map(format_date)),
        metadata: metadata_view(&entry.kind),
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
        let contacts = [
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
        Self {
            profile: ProfileView {
                name: cv.profile.name.clone(),
                contacts,
                summary: cv.profile.summary.as_ref().map(rich_view),
            },
            sections: cv
                .sections
                .iter()
                .map(|section| SectionView {
                    id: section.id.clone(),
                    kind: section.kind,
                    title: section.title.clone(),
                    entries: entry_views(&section.entries),
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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;

    #[test]
    fn complete_projection_covers_visible_fields() {
        let cv: CvDocument =
            serde_json::from_str(include_str!("fixtures/shared/complete.json")).unwrap();
        let complete = render_view(&cv).unwrap();
        let complete: Value = serde_json::from_slice(&complete).unwrap();
        assert_eq!(complete["profile"]["contacts"].as_array().unwrap().len(), 4);
        assert_eq!(
            complete["profile"]["contacts"][0]["href"],
            "mailto:CACEmailZZZ@example.com"
        );
        for (index, section) in cv.sections.iter().enumerate() {
            assert_eq!(complete["sections"][index]["id"], section.id);
            assert_eq!(
                complete["sections"][index]["kind"],
                serde_json::to_value(section.kind).unwrap()
            );
        }
        let sections = &complete["sections"];
        assert_eq!(sections[0]["entries"][0]["metadata"][0]["role"], "location");
        assert_eq!(
            sections[0]["entries"][0]["metadata"][0]["body"][0]["text"],
            "CACLocationZZZ"
        );
        for index in [2, 3] {
            let metadata = &sections[index]["entries"][0]["metadata"][0];
            assert_eq!(metadata["role"], "url");
            assert_eq!(metadata["body"][0]["kind"], "link");
            assert_eq!(
                metadata["body"][0]["href"],
                metadata["body"][0]["body"][0]["text"]
            );
        }
        assert_eq!(
            sections[3]["entries"][0]["secondary"][0]["text"],
            "CACPublisherZZZ"
        );
        assert_eq!(sections[3]["entries"][0]["period"], "2023");
        assert_eq!(sections[5]["entries"].as_array().unwrap().len(), 2);
        assert_eq!(
            sections[5]["entries"][1]["primary"][0]["text"],
            "CACTextOneZZZ"
        );
        assert_eq!(
            sections[5]["entries"][1]["highlights"][0][0]["text"],
            "CACTextTwoZZZ"
        );
    }
}
