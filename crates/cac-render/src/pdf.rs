use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};

use cac_core::CvDocument;
use image::ImageEncoder as _;
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

use crate::{Settings, SettingsError, validate_theme_name};

pub const MAIN_TYP_SOURCE: &str = include_str!("typst/main.typ");
pub const BASE_TYP_SOURCE: &str = include_str!("typst/base.typ");
pub const STYLE_TYP_SOURCE: &str = include_str!("typst/style.typ");
pub const LAYOUT_TYP_SOURCE: &str = include_str!("typst/layout.typ");

const MAIN_TYP: &str = MAIN_TYP_SOURCE;
const BASE_TYP: &str = BASE_TYP_SOURCE;
const STYLE_TYP: &str = STYLE_TYP_SOURCE;
const LAYOUT_TYP: &str = LAYOUT_TYP_SOURCE;
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
    #[error("preview image export failed: {0}")]
    Image(String),
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
    /// 1-based pages whose content extends beyond the page edge.
    pub overflow_pages: Vec<usize>,
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
    render_pdf_document(cv, options).map(|(pdf, _)| pdf)
}

pub(crate) fn render_pdf_document(
    cv: &CvDocument,
    options: &RenderOptions,
) -> Result<(RenderedPdf, PagedDocument), RenderError> {
    let settings = Settings::validate(options.settings.clone())?;
    let theme_name = settings.theme_name().to_owned();
    let resolved = resolve_theme(&theme_name, options)?;
    let mut render_settings = settings;
    render_settings.root = None;
    render_settings.naming = None;
    render_settings.theme = None;
    render_settings.theme_project = None;
    let shimmed = crate::style::legacy::shim(&render_settings);
    let theme_uses_components = resolved
        .source_text
        .as_deref()
        .is_some_and(|source| source.contains("components:"));
    if theme_uses_components {
        eprintln!(
            "NOTE legacy component themes are deprecated; migrate to layout hooks or theme.toml"
        );
    }
    let family = (theme_name == "classic" && !theme_uses_components)
        .then(crate::style::legacy::classic_family);
    let (style_sheet, advisories) =
        crate::style::resolve::resolve_layered(family.as_ref(), &shimmed).map_err(|error| {
            RenderError::Settings(crate::settings::SettingsError::Style(error.to_string()))
        })?;
    for advisory in &advisories {
        eprintln!("NOTE {}", advisory.message);
    }
    let assets = collect_assets(
        cv.profile.photo.as_deref(),
        project_root(options).as_deref(),
    )?;
    let world = CacWorld::new(
        crate::view::render_view(cv)?,
        serde_json::to_vec(&render_settings)?,
        &render_settings,
        serde_json::to_vec_pretty(&style_sheet)?,
        // Sparse: only the keys settings set, with lengths already in the
        // (value, unit) form the bridge reads. The overlay re-asserts the
        // settings layer above legacy theme dictionaries; unset keys must
        // not clobber the family values beneath them.
        serde_json::to_vec(&renderer_style_json(&shimmed.style))?,
        &resolved,
        assets,
        options.project_dir.clone(),
    )?;
    let warned = typst::compile::<PagedDocument>(&world);
    let document = warned
        .output
        .map_err(|errors| RenderError::Compile(format_diagnostics(&errors)))?;
    let pages = document.pages().len();
    let overflow = crate::diagnostics::overflow_pages(document.pages());
    for page in &overflow {
        eprintln!("NOTE page {page} has content outside the page area");
    }
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
    Ok((
        RenderedPdf {
            bytes,
            pages,
            theme: theme_name,
            overflow_pages: overflow.clone(),
            theme_source: resolved.source,
        },
        document,
    ))
}

pub fn render_pdf_and_preview_with_options(
    cv: &CvDocument,
    options: &RenderOptions,
) -> Result<(RenderedPdf, Vec<u8>, u32, u32), RenderError> {
    let settings = Settings::validate(options.settings.clone())?;
    let theme_name = settings.theme_name().to_owned();
    let resolved = resolve_theme(&theme_name, options)?;
    let mut render_settings = settings;
    render_settings.root = None;
    render_settings.naming = None;
    render_settings.theme = None;
    render_settings.theme_project = None;
    let shimmed = crate::style::legacy::shim(&render_settings);
    let theme_uses_components = resolved
        .source_text
        .as_deref()
        .is_some_and(|source| source.contains("components:"));
    if theme_uses_components {
        eprintln!(
            "NOTE legacy component themes are deprecated; migrate to layout hooks or theme.toml"
        );
    }
    let family = (theme_name == "classic" && !theme_uses_components)
        .then(crate::style::legacy::classic_family);
    let (style_sheet, advisories) =
        crate::style::resolve::resolve_layered(family.as_ref(), &shimmed).map_err(|error| {
            RenderError::Settings(crate::settings::SettingsError::Style(error.to_string()))
        })?;
    for advisory in &advisories {
        eprintln!("NOTE {}", advisory.message);
    }
    let assets = collect_assets(
        cv.profile.photo.as_deref(),
        project_root(options).as_deref(),
    )?;
    let world = CacWorld::new(
        crate::view::render_view(cv)?,
        serde_json::to_vec(&render_settings)?,
        &render_settings,
        serde_json::to_vec_pretty(&style_sheet)?,
        // Sparse: only the keys settings set, with lengths already in the
        // (value, unit) form the bridge reads. The overlay re-asserts the
        // settings layer above legacy theme dictionaries; unset keys must
        // not clobber the family values beneath them.
        serde_json::to_vec(&renderer_style_json(&shimmed.style))?,
        &resolved,
        assets,
        options.project_dir.clone(),
    )?;
    let document = typst::compile::<PagedDocument>(&world)
        .output
        .map_err(|errors| RenderError::Compile(format_diagnostics(&errors)))?;
    let first_page = document
        .pages()
        .first()
        .ok_or_else(|| RenderError::Image("document has no pages".into()))?;
    let preview_options = typst_render::RenderOptions {
        pixel_per_pt: 1.0.into(),
        render_bleed: false,
    };
    let pixmap = typst_render::render(first_page, &preview_options);
    let width = pixmap.width();
    let height = pixmap.height();
    let rgba = pixmap.take();
    let mut rgb = Vec::with_capacity((width * height * 3) as usize);
    for pixel in rgba.as_chunks::<4>().0 {
        let alpha = u16::from(pixel[3]);
        for channel in &pixel[..3] {
            rgb.push((u16::from(*channel) + 255 - alpha).min(255) as u8);
        }
    }
    let mut jpeg = Vec::new();
    image::codecs::jpeg::JpegEncoder::new_with_quality(&mut jpeg, 90)
        .write_image(&rgb, width, height, image::ExtendedColorType::Rgb8)
        .map_err(|error| RenderError::Image(error.to_string()))?;
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
    let pages = document.pages().len();
    let overflow = crate::diagnostics::overflow_pages(document.pages());
    for page in &overflow {
        eprintln!("NOTE page {page} has content outside the page area");
    }
    Ok((
        RenderedPdf {
            bytes,
            pages,
            theme: theme_name,
            overflow_pages: overflow.clone(),
            theme_source: resolved.source,
        },
        jpeg,
        width,
        height,
    ))
}

struct ResolvedTheme {
    source_text: Option<String>,
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
                    source_text: std::fs::read_to_string(directory.join("theme.typ")).ok(),
                    source,
                    directory: Some(directory),
                    embedded_source: None,
                });
            }
        }
    }
    if let Some(source) = embedded_theme_source(name) {
        Ok(ResolvedTheme {
            source_text: Some(source.to_string()),
            source: ThemeSource::Embedded,
            directory: None,
            embedded_source: Some(source),
        })
    } else {
        Err(RenderError::ThemeNotFound(name.into()))
    }
}

/// The view JSON the Typst bridge reads (`/.cac/cv.json`) — also what a
/// theme project's on-disk scaffold carries for `typst watch`.
pub fn view_json(cv: &CvDocument) -> String {
    String::from_utf8(crate::view::render_view(cv).unwrap_or_default()).unwrap_or_default()
}

/// The resolved style sheet JSON (`/.cac/style.json`) for a settings file
/// and theme — the same layering the PDF render path uses.
pub fn style_sheet_json(settings: &Settings, theme_name: &str) -> Result<String, SettingsError> {
    let sheet = crate::style::resolved_style_for(settings, theme_name)
        .map_err(|error| SettingsError::Style(error.to_string()))?;
    serde_json::to_vec_pretty(&sheet)
        .map(|bytes| String::from_utf8_lossy(&bytes).into_owned())
        .map_err(|error| SettingsError::Style(error.to_string()))
}

/// The sparse settings layer JSON (`/.cac/settings-style.json`), with
/// lengths already in the (value, unit) form the bridge reads.
pub fn settings_style_json(settings: &Settings) -> Result<String, SettingsError> {
    let shimmed = crate::style::legacy::shim(settings);
    serde_json::to_vec(&renderer_style_json(&shimmed.style))
        .map(|bytes| String::from_utf8_lossy(&bytes).into_owned())
        .map_err(|error| SettingsError::Style(error.to_string()))
}

/// The sparse settings layer, renderer-ready: a `LengthSpec` serializes as
/// its authoring string (`"16mm"`), but the Typst bridge reads
/// `(value, unit)` pairs — so lengths convert here, once, in Rust, and no
/// string ever reaches a Typst `length()` helper.
fn renderer_style_json(style: &crate::style::schema::StyleOverrides) -> serde_json::Value {
    fn walk(value: serde_json::Value) -> serde_json::Value {
        match value {
            serde_json::Value::String(text) => {
                let spec: crate::style::schema::LengthSpec =
                    serde_json::from_value(serde_json::Value::String(text.clone()))
                        .expect("a string rebuilds a length spec");
                match spec.parse() {
                    Ok(length) => {
                        serde_json::to_value(length).unwrap_or(serde_json::Value::String(text))
                    }
                    Err(_) => serde_json::Value::String(text),
                }
            }
            serde_json::Value::Array(items) => {
                serde_json::Value::Array(items.into_iter().map(walk).collect())
            }
            serde_json::Value::Object(map) => serde_json::Value::Object(
                map.into_iter()
                    .filter(|(_, value)| !value.is_null())
                    .map(|(key, value)| (key, walk(value)))
                    .collect(),
            ),
            other => other,
        }
    }
    walk(serde_json::to_value(style).expect("style overrides serialize"))
}

/// The jail root for project files: the `.cac` directory's parent.
fn project_root(options: &RenderOptions) -> Option<PathBuf> {
    options
        .project_dir
        .as_deref()
        .and_then(|dir| dir.parent().map(Path::to_path_buf))
}

/// The pinned document date with its locale spellings, computed here so no
/// theme keeps a second era table: form themes print the 和暦 line from
/// this file instead of reading the clock.
fn document_date_json(settings: &Settings) -> String {
    let date = settings
        .document_date
        .as_deref()
        .and_then(|value| chrono::NaiveDate::parse_from_str(value, "%Y-%m-%d").ok());
    let payload = match date {
        Some(date) => {
            let point = cac_core::DatePoint::Full(date);
            serde_json::json!({
                "date": date.to_string(),
                "era": crate::view::dates::era_date_full(date),
                "japanese": crate::view::dates::format_date_locale("ja", &point),
            })
        }
        None => serde_json::json!({ "date": null, "era": null, "japanese": null }),
    };
    serde_json::to_string(&payload).unwrap_or_else(|_| "{}".into())
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
    /// The project `.cac` directory, when one exists; on-disk bridge files
    /// there win over the embedded library, so author edits are real.
    project_cac: Option<PathBuf>,
    json: Bytes,
    settings: Bytes,
    style: Bytes,
    settings_style: Bytes,
    document_date: Bytes,
    document_date_value: Option<Datetime>,
    theme: ResolvedTheme,
    /// The only project files Typst may read: the paths the renderer itself
    /// collected (the document's photo), keyed by the view's spelling.
    assets: std::collections::BTreeMap<String, Bytes>,
}

/// Read a project asset inside the jail: canonicalized root, no `..`, no
/// symlink escapes, absolute paths refused.
fn jailed_read(root: &Path, relative: &Path) -> Result<Vec<u8>, FileError> {
    if relative.is_absolute() {
        return Err(FileError::AccessDenied);
    }
    let canonical_root = root.canonicalize().unwrap_or_else(|_| root.to_path_buf());
    let canonical = canonical_root
        .join(relative)
        .canonicalize()
        .map_err(|error| FileError::Other(Some(error.to_string().into())))?;
    if !canonical.starts_with(&canonical_root) {
        return Err(FileError::AccessDenied);
    }
    fs::read(canonical).map_err(|error| FileError::Other(Some(error.to_string().into())))
}

/// Collect the project assets the render may read: only the document's
/// photo. Everything else stays unreachable from theme code, so a theme
/// cannot embed `.env`, `.git/config`, or settings contents in its output.
fn collect_assets(
    photo: Option<&str>,
    root: Option<&Path>,
) -> Result<std::collections::BTreeMap<String, Bytes>, RenderError> {
    let mut assets = std::collections::BTreeMap::new();
    let Some(photo) = photo else {
        return Ok(assets);
    };
    let Some(root) = root else {
        return Ok(assets);
    };
    // A photo the file cannot supply is a warning, not a hard error: the
    // corpus renders without one, and a theme that needs the image gets
    // Typst's own not-found error at compile time.
    let Ok(data) = jailed_read(root, Path::new(photo)) else {
        eprintln!(
            "NOTE photo `{photo}` was not found in the project directory; rendering without it"
        );
        return Ok(assets);
    };
    const MAX_ASSET_BYTES: usize = 32 * 1024 * 1024;
    if data.len() > MAX_ASSET_BYTES {
        eprintln!(
            "NOTE photo `{photo}` is larger than the {MAX_ASSET_BYTES}-byte asset cap; rendering without it"
        );
        return Ok(assets);
    }
    assets.insert(photo.to_owned(), Bytes::new(data));
    Ok(assets)
}

impl CacWorld {
    fn new(
        json: Vec<u8>,
        settings: Vec<u8>,
        settings_parsed: &Settings,
        style: Vec<u8>,
        settings_style: Vec<u8>,
        theme: &ResolvedTheme,
        assets: std::collections::BTreeMap<String, Bytes>,
        project_cac: Option<PathBuf>,
    ) -> Result<Self, RenderError> {
        let main = file_id("main.typ");
        let mut fonts = FontStore::new();
        fonts.extend(embedded());
        if let Some(directory) = &theme.directory {
            load_theme_fonts(&directory.join("fonts"), &mut fonts)?;
        }
        let document_date = document_date_from(settings_parsed);
        // A theme project's scaffold owns its bridge: an on-disk
        // main.typ renders instead of the embedded one, so the file the
        // author edits with `typst watch` is the file `cac build` runs.
        let main_source = project_cac
            .as_ref()
            .and_then(|root| fs::read_to_string(root.join("main.typ")).ok())
            .map_or_else(
                || Source::new(main, MAIN_TYP.into()),
                |text| Source::new(main, text),
            );
        Ok(Self {
            library: LazyHash::new(Library::default()),
            fonts,
            main,
            main_source,
            json: Bytes::new(json),
            settings: Bytes::new(settings),
            style: Bytes::new(style),
            settings_style: Bytes::new(settings_style),
            document_date: Bytes::new(document_date_json(settings_parsed).into_bytes()),
            document_date_value: document_date,
            project_cac,
            theme: ResolvedTheme {
                source_text: theme.source_text.clone(),
                source: theme.source,
                directory: theme.directory.clone(),
                embedded_source: theme.embedded_source,
            },
            assets,
        })
    }

    /// A bridge library file: the project scaffold's copy when one
    /// exists, the embedded library otherwise. Author edits under
    /// `.cac/` are real for `cac build`, not just for `typst watch`.
    fn bridge_file(&self, name: &str, embedded: &'static str) -> FileResult<Bytes> {
        if let Some(root) = &self.project_cac {
            let path = root.join(name);
            if path.is_file() {
                return fs::read(path)
                    .map(Bytes::new)
                    .map_err(|error| FileError::Other(Some(error.to_string().into())));
            }
        }
        Ok(Bytes::new(embedded.as_bytes()))
    }

    fn theme_file(&self, relative: &Path) -> FileResult<Bytes> {
        let Some(root) = &self.theme.directory else {
            return Err(FileError::NotFound(relative.into()));
        };
        jailed_read(root, relative).map(Bytes::new)
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
            ".cac/base.typ" => self.bridge_file("base.typ", BASE_TYP)?,
            ".cac/style.typ" => self.bridge_file("style.typ", STYLE_TYP)?,
            ".cac/layout.typ" => self.bridge_file("layout.typ", LAYOUT_TYP)?,
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
        let virtual_path = id.vpath().get_without_slash();
        match virtual_path {
            ".cac/cv.json" => Ok(self.json.clone()),
            ".cac/settings.json" => Ok(self.settings.clone()),
            ".cac/style.json" => Ok(self.style.clone()),
            ".cac/settings-style.json" => Ok(self.settings_style.clone()),
            ".cac/document-date.json" => Ok(self.document_date.clone()),
            value if value.starts_with(".cac/") => self.theme_file(Path::new(&value[5..])),
            // Project files: only the renderer-collected assets (the
            // document's photo) are reachable. A theme asking for anything
            // else — `.env`, `.git/config`, settings — is refused before
            // the filesystem is touched.
            other => self
                .assets
                .get(other)
                .cloned()
                .ok_or(FileError::AccessDenied),
        }
    }
    fn font(&self, index: usize) -> Option<Font> {
        self.fonts.font(index)
    }
    fn today(&self, _: Option<Duration>) -> Option<Datetime> {
        // Deterministic builds: the document date setting, never the clock.
        self.document_date_value
    }
}

/// The document date from settings, parsed once for the world's clock.
fn document_date_from(settings: &Settings) -> Option<Datetime> {
    let date = settings.document_date.as_deref()?;
    let parsed = chrono::NaiveDate::parse_from_str(date, "%Y-%m-%d").ok()?;
    use chrono::Datelike;
    Datetime::from_ymd_hms(
        parsed.year(),
        u8::try_from(parsed.month()).ok()?,
        u8::try_from(parsed.day()).ok()?,
        0,
        0,
        0,
    )
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
