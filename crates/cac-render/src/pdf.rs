use cac_core::{CvDocument, Inline, RichText};
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

const MAIN_TYP: &str = include_str!("classic.typ");

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
