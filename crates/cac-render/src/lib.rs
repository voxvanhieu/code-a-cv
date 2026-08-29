use std::fmt::Write;

use cac_core::{CvDocument, DatePoint, Inline, RichText};
use serde::Serialize;
use thiserror::Error;
use typst::diag::{FileError, FileResult};
use typst::foundations::{Bytes, Datetime, Duration, Smart};
use typst::syntax::{FileId, RootedPath, Source, VirtualPath, VirtualRoot};
use typst::text::{Font, FontBook};
use typst::utils::LazyHash;
use typst::{Library, LibraryExt, World};
use typst_kit::fonts::{FontStore, embedded};
use typst_layout::PagedDocument;
use typst_pdf::{PdfOptions, PdfStandard, PdfStandards, Timestamp};

const MAIN_TYP: &str = r#"
#let cv = json("cv.json")
#let rich(nodes) = {
  for node in nodes {
    if node.kind == "text" { node.text }
    else if node.kind == "code" { raw(node.text) }
    else if node.kind == "emph" { emph(rich(node.body)) }
    else if node.kind == "strong" { strong(rich(node.body)) }
    else if node.kind == "link" { link(node.href, rich(node.body)) }
  }
}
#set document(title: cv.profile.name + " CV", author: cv.profile.name)
#set page(paper: "a4", margin: (x: 1.8cm, y: 1.5cm))
#set text(font: "Libertinus Serif", size: 10pt)
#set par(justify: true, leading: 0.55em)
#show heading.where(level: 2): it => block(above: 1.0em, below: 0.45em)[
  #set text(size: 12pt, weight: "bold")
  #it.body
  #line(length: 100%, stroke: 0.5pt)
]

#align(center)[
  #text(size: 20pt, weight: "bold")[#cv.profile.name]
  #v(0.25em)
  #cv.profile.contacts.join("  ·  ")
]

#if cv.profile.summary != none [
  #v(0.8em)
  #rich(cv.profile.summary)
]

#for section in cv.sections [
  == #section.title
  #for entry in section.entries [
    #grid(
      columns: (1fr, auto),
      gutter: 0.5em,
      [#strong(rich(entry.primary))#if entry.secondary != none [, #rich(entry.secondary)]],
      [#if entry.period != none [#entry.period]],
    )
    #if entry.highlights.len() > 0 [
      #set list(indent: 1em, body-indent: 0.45em, spacing: 0.15em)
      #for item in entry.highlights [
        - #rich(item)
      ]
    ]
    #v(0.35em)
  ]
]
"#;

#[derive(Debug, Error)]
pub enum RenderError {
    #[error("could not serialize the CV: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Typst compilation failed: {0}")]
    Compile(String),
    #[error("PDF export failed: {0}")]
    Pdf(String),
}

pub struct RenderedPdf {
    pub bytes: Vec<u8>,
    pub pages: usize,
}

pub fn render_pdf(cv: &CvDocument) -> Result<RenderedPdf, RenderError> {
    let json = serde_json::to_vec(&RenderView::from(cv))?;
    let world = CacWorld::new(json);
    let warned = typst::compile::<PagedDocument>(&world);
    let document = warned
        .output
        .map_err(|errors| RenderError::Compile(format_diagnostics(&errors)))?;
    let pages = document.pages().len();
    let standards = PdfStandards::new(&[PdfStandard::A_2b])
        .map_err(|error| RenderError::Pdf(format!("{error:?}")))?;
    let options = PdfOptions {
        ident: Smart::Custom(format!("cac:{}", cv.profile.name)),
        creator: Smart::Custom(Some(format!("cac {}", env!("CARGO_PKG_VERSION")))),
        timestamp: Some(Timestamp::new_utc(
            Datetime::from_ymd_hms(2000, 1, 1, 0, 0, 0).expect("valid pinned timestamp"),
        )),
        standards,
        ..PdfOptions::default()
    };
    let bytes = typst_pdf::pdf(&document, &options)
        .map_err(|errors| RenderError::Pdf(format_diagnostics(&errors)))?;
    Ok(RenderedPdf { bytes, pages })
}

fn format_diagnostics<T: std::fmt::Debug>(diagnostics: &[T]) -> String {
    diagnostics
        .iter()
        .map(|value| format!("{value:?}"))
        .collect::<Vec<_>>()
        .join("; ")
}

struct CacWorld {
    library: LazyHash<Library>,
    fonts: FontStore,
    main: FileId,
    data: FileId,
    main_source: Source,
    json: Bytes,
}

impl CacWorld {
    fn new(json: Vec<u8>) -> Self {
        let main = FileId::new(RootedPath::new(
            VirtualRoot::Project,
            VirtualPath::new("main.typ").expect("valid embedded path"),
        ));
        let data = FileId::new(RootedPath::new(
            VirtualRoot::Project,
            VirtualPath::new("cv.json").expect("valid embedded path"),
        ));
        let mut fonts = FontStore::new();
        fonts.extend(embedded());
        Self {
            library: LazyHash::new(Library::default()),
            fonts,
            main,
            data,
            main_source: Source::new(main, MAIN_TYP.into()),
            json: Bytes::new(json),
        }
    }
}

impl World for CacWorld {
    fn library(&self) -> &LazyHash<Library> {
        &self.library
    }
    fn book(&self) -> &LazyHash<FontBook> {
        self.fonts.book()
    }
    fn main(&self) -> FileId {
        self.main
    }
    fn source(&self, id: FileId) -> FileResult<Source> {
        if id == self.main {
            Ok(self.main_source.clone())
        } else {
            Err(FileError::NotFound(id.vpath().get_without_slash().into()))
        }
    }
    fn file(&self, id: FileId) -> FileResult<Bytes> {
        if id == self.data {
            Ok(self.json.clone())
        } else {
            Err(FileError::NotFound(id.vpath().get_without_slash().into()))
        }
    }
    fn font(&self, index: usize) -> Option<Font> {
        self.fonts.font(index)
    }
    fn today(&self, _: Option<Duration>) -> Option<Datetime> {
        None
    }
}

#[derive(Serialize)]
struct RenderView {
    profile: ProfileView,
    sections: Vec<SectionView>,
}

#[derive(Serialize)]
struct ProfileView {
    name: String,
    contacts: Vec<String>,
    summary: Option<Vec<InlineView>>,
}

#[derive(Serialize)]
struct SectionView {
    title: String,
    entries: Vec<EntryView>,
}

#[derive(Serialize)]
struct EntryView {
    primary: Vec<InlineView>,
    secondary: Option<Vec<InlineView>>,
    period: Option<String>,
    highlights: Vec<Vec<InlineView>>,
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

impl From<&CvDocument> for RenderView {
    fn from(cv: &CvDocument) -> Self {
        let contacts = [
            cv.profile.email.clone(),
            cv.profile.phone.clone(),
            cv.profile.location.clone(),
            cv.profile.website.as_ref().map(ToString::to_string),
        ]
        .into_iter()
        .flatten()
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
                    title: section.title.clone(),
                    entries: section
                        .entries
                        .iter()
                        .map(|entry| {
                            let (primary, secondary) = entry.kind.heading();
                            EntryView {
                                primary: rich_view(primary),
                                secondary: secondary
                                    .filter(|value| !value.is_empty())
                                    .map(rich_view),
                                period: entry
                                    .kind
                                    .period()
                                    .map(|period| format!("{}–{}", period.start, period.end))
                                    .or_else(|| entry.kind.date().map(ToString::to_string)),
                                highlights: entry.kind.highlights().iter().map(rich_view).collect(),
                            }
                        })
                        .collect(),
                })
                .collect(),
        }
    }
}

pub fn render_html(cv: &CvDocument) -> String {
    let mut output = String::from(
        "<!doctype html><html lang=\"en\"><head><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width,initial-scale=1\"><title>",
    );
    output.push_str(&html_escape::encode_text(&format!(
        "{} CV",
        cv.profile.name
    )));
    output.push_str("</title><style>:root{font-family:system-ui,sans-serif;color:#171717}body{max-width:800px;margin:2rem auto;padding:0 1.5rem;line-height:1.4}header{text-align:center}h1{margin-bottom:.25rem}h2{font-size:1.15rem;border-bottom:1px solid;margin-top:1.4rem}.contact{display:flex;gap:.75rem;justify-content:center;flex-wrap:wrap}.entry{margin:.8rem 0}.entry-head{display:flex;justify-content:space-between;gap:1rem}.period{white-space:nowrap}ul{margin:.35rem 0;padding-left:1.3rem}@media print{body{margin:0;max-width:none}@page{size:A4;margin:15mm}}</style></head><body itemscope itemtype=\"https://schema.org/Person\"><header><h1 itemprop=\"name\">");
    output.push_str(&html_escape::encode_text(&cv.profile.name));
    output.push_str("</h1><div class=\"contact\">");
    if let Some(email) = &cv.profile.email {
        let _ = write!(
            output,
            "<a itemprop=\"email\" href=\"mailto:{}\">{}</a>",
            html_escape::encode_double_quoted_attribute(email),
            html_escape::encode_text(email)
        );
    }
    if let Some(phone) = &cv.profile.phone {
        let _ = write!(
            output,
            "<span itemprop=\"telephone\">{}</span>",
            html_escape::encode_text(phone)
        );
    }
    if let Some(location) = &cv.profile.location {
        let _ = write!(
            output,
            "<span itemprop=\"address\">{}</span>",
            html_escape::encode_text(location)
        );
    }
    if let Some(website) = &cv.profile.website {
        let escaped = html_escape::encode_double_quoted_attribute(website.as_str());
        let _ = write!(
            output,
            "<a itemprop=\"url\" href=\"{escaped}\">{}</a>",
            html_escape::encode_text(website.as_str())
        );
    }
    output.push_str("</div></header>");
    if let Some(summary) = &cv.profile.summary {
        output.push_str("<p>");
        render_rich_html(summary, &mut output);
        output.push_str("</p>");
    }
    for section in &cv.sections {
        let _ = write!(
            output,
            "<section id=\"{}\"><h2>{}</h2>",
            html_escape::encode_double_quoted_attribute(&section.id),
            html_escape::encode_text(&section.title)
        );
        for entry in &section.entries {
            let (primary, secondary) = entry.kind.heading();
            output.push_str("<article class=\"entry\"><div class=\"entry-head\"><div><strong>");
            render_rich_html(primary, &mut output);
            if let Some(value) = secondary.filter(|value| !value.is_empty()) {
                output.push_str(", </strong>");
                render_rich_html(value, &mut output);
            } else {
                output.push_str("</strong>");
            }
            output.push_str("</div>");
            if let Some(period) = entry.kind.period() {
                let _ = write!(
                    output,
                    "<time class=\"period\">{}–{}</time>",
                    period.start, period.end
                );
            }
            output.push_str("</div>");
            if !entry.kind.highlights().is_empty() {
                output.push_str("<ul>");
                for highlight in entry.kind.highlights() {
                    output.push_str("<li>");
                    render_rich_html(highlight, &mut output);
                    output.push_str("</li>");
                }
                output.push_str("</ul>");
            }
            output.push_str("</article>");
        }
        output.push_str("</section>");
    }
    output.push_str("</body></html>");
    output
}

fn render_rich_html(value: &RichText, output: &mut String) {
    fn render(nodes: &[Inline], output: &mut String) {
        for node in nodes {
            match node {
                Inline::Text(value) => output.push_str(&html_escape::encode_text(value)),
                Inline::Code(value) => {
                    output.push_str("<code>");
                    output.push_str(&html_escape::encode_text(value));
                    output.push_str("</code>");
                }
                Inline::Emph(body) => {
                    output.push_str("<em>");
                    render(body, output);
                    output.push_str("</em>");
                }
                Inline::Strong(body) => {
                    output.push_str("<strong>");
                    render(body, output);
                    output.push_str("</strong>");
                }
                Inline::Link { href, body } => {
                    let _ = write!(
                        output,
                        "<a href=\"{}\">",
                        html_escape::encode_double_quoted_attribute(href.as_str())
                    );
                    render(body, output);
                    output.push_str("</a>");
                }
            }
        }
    }
    render(&value.0, output);
}

pub fn format_date(value: &DatePoint) -> String {
    value.to_string()
}
