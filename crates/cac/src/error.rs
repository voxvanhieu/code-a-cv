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
    #[error("input path `{0}` has no supported extension")]
    Unsupported(PathBuf),
    #[error("CV checks failed")]
    CheckFailed,
    #[error("could not determine the user home directory")]
    HomeDirectory,
    #[error("theme `{0}` is not available as an embedded theme")]
    EmbeddedTheme(String),
    #[error("theme `{0}` is not installed in the selected location")]
    ThemeNotInstalled(String),
}
