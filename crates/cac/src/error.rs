use std::path::PathBuf;

use cac_io::ParseError;
use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("{0}")]
    Io(#[from] std::io::Error),
    #[error("{0}")]
    Parse(#[from] ParseError),
    #[error("{0}")]
    Render(#[from] cac_render::RenderError),
    #[error("{0}")]
    Settings(#[from] cac_render::SettingsError),
    #[error("could not serialize output: {0}")]
    Json(#[from] serde_json::Error),
    #[error("could not serialize YAML: {0}")]
    Yaml(#[from] serde_yaml_ng::Error),
    #[error("could not serialize TOML: {0}")]
    Toml(#[from] toml::ser::Error),
    #[error("{0} already exists; use --force to replace it")]
    Exists(PathBuf),
    #[error("{0} is reserved for theme settings; choose another CV output path")]
    SettingsOutput(PathBuf),
    #[error("cac init requires a file output; standard output is not supported")]
    InitStdout,
    #[error("CV output `{path}` does not have an extension compatible with the `{format}` format")]
    InitOutputFormat { path: PathBuf, format: &'static str },
    #[error("CV output `{0}` must have a UTF-8 file name for settings.json")]
    CvFileName(PathBuf),
    #[error("input path `{0}` has no supported extension")]
    Unsupported(PathBuf),
    #[error("CV checks failed")]
    CheckFailed,
    #[error("could not determine the user home directory")]
    HomeDirectory,
    #[error("theme `{0}` is not available in the downloadable theme registry")]
    DownloadableTheme(String),
    #[error("theme name `{0}` is reserved by cac and cannot be installed")]
    SystemThemeName(String),
    #[error("theme registry error: {0}")]
    ThemeRegistry(String),
    #[error("downloaded theme `{theme}` file `{path}` failed checksum verification")]
    ThemeChecksum { theme: String, path: String },
    #[error("theme `{0}` is not installed in the selected location")]
    ThemeNotInstalled(String),
    #[error("theme project error: {0}")]
    ThemeProject(String),
}
