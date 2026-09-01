use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};

use cac_core::{CvDocument, Entry, EntryKind, Inline, RichText};
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

use crate::{Settings, SettingsError, format_date, validate_theme_name};

const MAIN_TYP: &str = include_str!("typst/main.typ");
const BASE_TYP: &str = include_str!("typst/base.typ");
const CLASSIC_TYP: &str = include_str!("typst/themes/classic.typ");

pub const EMBEDDED_THEME_NAMES: &[&str] = &["classic"];

pub fn embedded_theme_source(name: &str) -> Option<&'static str> {
    match name {
        "classic" => Some(CLASSIC_TYP),
        _ => None,
    }
}

#[derive(Debug, Error)]
pub enum RenderError {
    #[error("could not serialize renderer data: {0}")]
    Json(#[from] serde_json::Error),
    #[error(transparent)]
    Settings(#[from] SettingsError),
    #[error("theme `{0}` was not found in the project, user, or embedded theme directories")]
    ThemeNotFound(String),
    #[error("could not read theme file `{path}`: {source}")]
    ThemeFile {
        path: String,
        source: std::io::Error,
    },
    #[error("could not load theme font `{path}`")]
    ThemeFont { path: String },
    #[error("Typst compilation failed: {0}")]
    Compile(String),
    #[error("PDF export failed: {0}")]
    Pdf(String),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ThemeSource {
    Project,
    User,
    Embedded,
}

impl fmt::Display for ThemeSource {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Project => "project",
            Self::User => "user",
            Self::Embedded => "embedded",
        })
    }
}

#[derive(Clone, Debug, Default)]
pub struct RenderOptions {
    /// The project's `.cac` directory
    pub project_dir: Option<PathBuf>,
    /// The user's `~/.cac` directory
    pub user_dir: Option<PathBuf>,
    pub settings: Settings,
}

#[derive(Debug)]
pub struct RenderedPdf {
    pub bytes: Vec<u8>,
    pub pages: usize,
    pub theme: String,
    pub theme_source: ThemeSource,
}

pub fn render_pdf(cv: &CvDocument) -> Result<RenderedPdf, RenderError> {
    render_pdf_with_options(cv, &RenderOptions::default())
}

pub fn render_pdf_with_options(
    cv: &CvDocument,
    options: &RenderOptions,
) -> Result<RenderedPdf, RenderError> {
    let settings = Settings::validate(options.settings.clone())?;
    let theme_name = settings.theme_name().to_owned();
    let resolved = resolve_theme(&theme_name, options)?;
    let mut render_settings = settings;
    render_settings.root = None;
    render_settings.theme = None;
    let world = CacWorld::new(
        serde_json::to_vec(&RenderView::from(cv))?,
        serde_json::to_vec(&render_settings)?,
        &resolved,
    )?;
    let warned = typst::compile::<PagedDocument>(&world);
    let document = warned
        .output
        .map_err(|errors| RenderError::Compile(format_diagnostics(&errors)))?;
    let pages = document.pages().len();
    let standards = PdfStandards::new(&[PdfStandard::A_2b])
        .map_err(|error| RenderError::Pdf(format!("{error:?}")))?;
    let pdf_options = PdfOptions {
        ident: Smart::Custom(format!("cac:{}", cv.profile.name)),
        creator: Smart::Custom(Some(format!("cac {}", env!("CARGO_PKG_VERSION")))),
        timestamp: Some(Timestamp::new_utc(
            Datetime::from_ymd_hms(2000, 1, 1, 0, 0, 0).expect("valid pinned timestamp"),
        )),
        standards,
        ..PdfOptions::default()
    };
    let bytes = typst_pdf::pdf(&document, &pdf_options)
        .map_err(|errors| RenderError::Pdf(format_diagnostics(&errors)))?;
    Ok(RenderedPdf {
        bytes,
        pages,
        theme: theme_name,
        theme_source: resolved.source,
    })
}

struct ResolvedTheme {
    source: ThemeSource,
    directory: Option<PathBuf>,
    embedded_source: Option<&'static str>,
}

fn resolve_theme(name: &str, options: &RenderOptions) -> Result<ResolvedTheme, RenderError> {
    validate_theme_name(name)?;
    for (root, source) in [
        (options.project_dir.as_deref(), ThemeSource::Project),
        (options.user_dir.as_deref(), ThemeSource::User),
    ] {
        if let Some(root) = root {
            let directory = root.join("themes").join(name);
            if directory.join("theme.typ").is_file() {
                return Ok(ResolvedTheme {
                    source,
                    directory: Some(directory),
                    embedded_source: None,
                });
            }
        }
    }
    if let Some(source) = embedded_theme_source(name) {
        Ok(ResolvedTheme {
            source: ThemeSource::Embedded,
            directory: None,
            embedded_source: Some(source),
        })
    } else {
        Err(RenderError::ThemeNotFound(name.into()))
    }
}

fn format_diagnostics<T: fmt::Debug>(diagnostics: &[T]) -> String {
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
    main_source: Source,
    json: Bytes,
    settings: Bytes,
    theme: ResolvedTheme,
}

impl CacWorld {
    fn new(json: Vec<u8>, settings: Vec<u8>, theme: &ResolvedTheme) -> Result<Self, RenderError> {
        let main = file_id("main.typ");
        let mut fonts = FontStore::new();
        fonts.extend(embedded());
        if let Some(directory) = &theme.directory {
            load_theme_fonts(&directory.join("fonts"), &mut fonts)?;
        }
        Ok(Self {
            library: LazyHash::new(Library::default()),
            fonts,
            main,
            main_source: Source::new(main, MAIN_TYP.into()),
            json: Bytes::new(json),
            settings: Bytes::new(settings),
            theme: ResolvedTheme {
                source: theme.source,
                directory: theme.directory.clone(),
                embedded_source: theme.embedded_source,
            },
        })
    }

    fn theme_file(&self, relative: &Path) -> FileResult<Bytes> {
        let Some(root) = &self.theme.directory else {
            return Err(FileError::NotFound(relative.into()));
        };
        let root = root
            .canonicalize()
            .map_err(|error| FileError::Other(Some(error.to_string().into())))?;
        let canonical = root
            .join(relative)
            .canonicalize()
            .map_err(|error| FileError::Other(Some(error.to_string().into())))?;
        if !canonical.starts_with(&root) {
            return Err(FileError::AccessDenied);
        }
        fs::read(canonical)
            .map(Bytes::new)
            .map_err(|error| FileError::Other(Some(error.to_string().into())))
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
            return Ok(self.main_source.clone());
        }
        let path = id.vpath().get_without_slash();
        let contents = match path {
            ".cac/base.typ" => Bytes::new(BASE_TYP.as_bytes()),
            ".cac/theme.typ" if self.theme.directory.is_none() => Bytes::new(
                self.theme
                    .embedded_source
                    .expect("embedded theme source")
                    .as_bytes(),
            ),
            value if value.starts_with(".cac/") => self.theme_file(Path::new(&value[5..]))?,
            _ => return Err(FileError::NotFound(path.into())),
        };
        let text = std::str::from_utf8(&contents)
            .map_err(|_| FileError::InvalidUtf8)?
            .to_owned();
        Ok(Source::new(id, text))
    }

    fn file(&self, id: FileId) -> FileResult<Bytes> {
        match id.vpath().get_without_slash() {
            ".cac/cv.json" => Ok(self.json.clone()),
            ".cac/settings.json" => Ok(self.settings.clone()),
            value if value.starts_with(".cac/") => self.theme_file(Path::new(&value[5..])),
            value => Err(FileError::NotFound(value.into())),
        }
    }
    fn font(&self, index: usize) -> Option<Font> {
        self.fonts.font(index)
    }
    fn today(&self, _: Option<Duration>) -> Option<Datetime> {
        None
    }
}

fn file_id(path: &str) -> FileId {
    FileId::new(RootedPath::new(
        VirtualRoot::Project,
        VirtualPath::new(path).expect("valid embedded path"),
    ))
}

fn load_theme_fonts(directory: &Path, fonts: &mut FontStore) -> Result<(), RenderError> {
    if !directory.exists() {
        return Ok(());
    }
    let entries = fs::read_dir(directory).map_err(|source| RenderError::ThemeFile {
        path: directory.display().to_string(),
        source,
    })?;
    let mut paths = entries
        .map(|entry| {
            let entry = entry.map_err(|source| RenderError::ThemeFile {
                path: directory.display().to_string(),
                source,
            })?;
            Ok(entry.path())
        })
        .collect::<Result<Vec<_>, RenderError>>()?;
    paths.sort();
    for path in paths {
        if path.is_dir() {
            load_theme_fonts(&path, fonts)?;
        } else if path
            .extension()
            .and_then(|value| value.to_str())
            .is_some_and(|extension| {
                matches!(
                    extension.to_ascii_lowercase().as_str(),
                    "ttf" | "otf" | "ttc" | "otc"
                )
            })
        {
            let data = fs::read(&path).map_err(|source| RenderError::ThemeFile {
                path: path.display().to_string(),
                source,
            })?;
            let loaded = Font::iter(Bytes::new(data)).collect::<Vec<_>>();
            if loaded.is_empty() {
                return Err(RenderError::ThemeFont {
                    path: path.display().to_string(),
                });
            }
            fonts.extend(loaded.into_iter().map(|font| {
                let info = font.info().clone();
                (font, info)
            }));
        }
    }
    Ok(())
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
    kind: &'static str,
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
                    entries: entry_views(&section.entries),
                })
                .collect(),
        }
    }
}
