//! The view contract (C3): named slots over structured data, consumed by
//! both renderers.
//!
//! The review's view replaces the `primary`/`secondary`/`metadata[role]`
//! vocabulary with named slots and carries dates structured — never
//! pre-formatted, never locale-bound — so one formatter in each renderer
//! turns them into display strings from the sheet's locale. The projection
//! that emits these types lands with the view task (V1); the types here are
//! the contract that task, the Typst layout hooks, and the HTML renderer
//! all code against.
//!
//! JSON shape for one entry, as the layout hooks see it:
//!
//! ```json
//! {
//!   "kind": "experience",
//!   "title": [], "subtitle": null,
//!   "period": {"start": "2023-01", "end": "Present"}, "date": null,
//!   "location": null, "url": null,
//!   "fields": {}, "highlights": [], "content": null, "tags": []
//! }
//! ```

use cac_core::{Address, DatePoint, FieldMap, Inline, Period, RichText, TagSet};
use serde::{Deserialize, Serialize};

/// The whole document as the renderers see it.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ViewDocument {
    pub profile: ViewProfile,
    pub sections: Vec<ViewSection>,
}

/// The profile block: name, contacts, summary, and the extension fields.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ViewProfile {
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub headline: Option<String>,
    /// Project-relative photo path; the renderer resolves it in the project jail.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub photo: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub address: Option<Address>,
    pub contacts: Vec<ViewContact>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub summary: Option<RichSlot>,
}

/// One contact line: a label, an optional target, and the derived kind.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ViewContact {
    pub kind: String,
    pub label: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub href: Option<String>,
}

/// A section with its identity, kind, and ordered entries.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ViewSection {
    pub id: String,
    pub kind: String,
    pub title: String,
    #[serde(default, skip_serializing_if = "TagSet::is_empty")]
    pub tags: TagSet,
    pub entries: Vec<ViewEntry>,
}

/// One entry in named slots. Content is a slot, not a metadata item;
/// anything the model does not type stays in `fields`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ViewEntry {
    pub kind: String,
    pub title: RichSlot,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub subtitle: Option<RichSlot>,
    /// Structured range; the renderer formats it with the sheet's locale.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub period: Option<Period>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub date: Option<DatePoint>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub location: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(default, skip_serializing_if = "FieldMap::is_empty")]
    pub fields: FieldMap,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub highlights: Vec<RichSlot>,
    /// The entry's ordered prose, a slot in its own right.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content: Option<RichSlot>,
    #[serde(default, skip_serializing_if = "TagSet::is_empty")]
    pub tags: TagSet,
}

/// Rich text as the renderers receive it: an ordered array of tagged nodes.
pub type RichSlot = Vec<InlineView>;

/// One inline node of rich text. Kinds match what `rich()` in base.typ
/// consumes; this shape is compatibility-bearing for existing themes.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum InlineView {
    Text {
        text: String,
    },
    Emph {
        body: Vec<InlineView>,
    },
    Strong {
        body: Vec<InlineView>,
    },
    Code {
        text: String,
    },
    Link {
        href: String,
        body: Vec<InlineView>,
    },
    Paragraph {
        body: Vec<InlineView>,
    },
    Break,
    List {
        start: Option<u64>,
        items: Vec<Vec<InlineView>>,
    },
}

/// Convert core rich text into the view's node array.
pub fn inline_view(nodes: &[Inline]) -> Vec<InlineView> {
    nodes
        .iter()
        .map(|node| match node {
            Inline::Paragraph(body) => InlineView::Paragraph {
                body: inline_view(body),
            },
            Inline::Break => InlineView::Break,
            Inline::List { start, items } => InlineView::List {
                start: *start,
                items: items.iter().map(rich_view).collect(),
            },
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

/// Convert one rich-text value into the view's node array.
pub fn rich_view(value: &RichText) -> RichSlot {
    inline_view(&value.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use cac_core::DatePoint;

    fn sample_entry() -> ViewEntry {
        ViewEntry {
            kind: "experience".into(),
            title: rich_view(&RichText::literal("Director")),
            subtitle: Some(rich_view(&RichText::literal("Analytical Engines"))),
            period: Some(
                Period::partial(
                    Some(DatePoint::YearMonth(2023, 1)),
                    Some(DatePoint::Present),
                )
                .unwrap(),
            ),
            date: None,
            location: Some("London".into()),
            url: None,
            fields: {
                let mut fields = FieldMap::new();
                fields.insert("gpa", cac_core::FieldValue::Text("3.9".into()));
                fields
            },
            highlights: vec![rich_view(&RichText::literal("Shipped the engine"))],
            content: None,
            tags: ["leadership".to_string()].into_iter().collect(),
        }
    }

    #[test]
    fn entry_serializes_the_named_slots() {
        let entry = sample_entry();
        let value = serde_json::to_value(&entry).unwrap();
        let object = value.as_object().unwrap();
        for slot in [
            "kind",
            "title",
            "subtitle",
            "period",
            "location",
            "fields",
            "highlights",
            "tags",
        ] {
            assert!(object.contains_key(slot), "view entry carries `{slot}`");
        }
        // Dates stay structured: canonical tokens, never display strings.
        assert_eq!(value["period"]["start"], "2023-01");
        assert_eq!(value["period"]["end"], "Present");
        assert_eq!(value["fields"]["gpa"], "3.9");
    }

    #[test]
    fn absent_slots_are_omitted() {
        let entry = sample_entry();
        let value = serde_json::to_value(&entry).unwrap();
        assert!(value.get("date").is_none());
        assert!(value.get("url").is_none());
        assert!(value.get("content").is_none());
    }

    #[test]
    fn entry_json_round_trips() {
        let entry = sample_entry();
        let json = serde_json::to_string(&entry).unwrap();
        let restored: ViewEntry = serde_json::from_str(&json).unwrap();
        assert_eq!(restored, entry);
    }

    #[test]
    fn rich_nodes_keep_the_tagged_shape() {
        let nodes = vec![InlineView::Strong {
            body: vec![InlineView::Text {
                text: "bold".into(),
            }],
        }];
        let value = serde_json::to_value(&nodes).unwrap();
        assert_eq!(value[0]["kind"], "strong");
        assert_eq!(value[0]["body"][0]["kind"], "text");
    }

    #[test]
    fn profile_contract_carries_the_extension_fields() {
        let profile = ViewProfile {
            name: "Ada Lovelace".into(),
            headline: Some("Analyst".into()),
            photo: Some("photo.jpg".into()),
            address: None,
            contacts: Vec::new(),
            summary: None,
        };
        let value = serde_json::to_value(&profile).unwrap();
        assert_eq!(value["headline"], "Analyst");
        assert_eq!(value["photo"], "photo.jpg");
        assert!(value.get("address").is_none());
    }
}
