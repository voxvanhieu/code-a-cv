use std::fs;
use std::path::{Path, PathBuf};

use cac_io::{InputFormat, parse, to_markdown};
use clap::{Args as ClapArgs, ValueEnum};

use crate::error::{Error, Result};
use crate::source::{is_stdio, print_result, read_source};

const SETTINGS_FILE: &str = "settings.json";

pub const ABOUT: &str = "Create a starter CV and settings or import JSON Resume data";
pub const AFTER_HELP: &str = "Examples:\n  cac init\n  cac init --format json\n  cac init --format yaml -o new-cv.yaml\n  cac init --from resume.json";

#[derive(Clone, Copy, ValueEnum)]
enum Format {
    Markdown,
    Yaml,
    Json,
    Toml,
}

impl Format {
    fn default_output(self) -> PathBuf {
        let extension = match self {
            Self::Markdown => "md",
            Self::Yaml => "yaml",
            Self::Json => "json",
            Self::Toml => "toml",
        };
        format!("cv.{extension}").into()
    }

    fn name(self) -> &'static str {
        match self {
            Self::Markdown => "markdown",
            Self::Yaml => "yaml",
            Self::Json => "json",
            Self::Toml => "toml",
        }
    }

    fn accepts_extension(self, output: &Path) -> bool {
        let Some(extension) = output.extension().and_then(|value| value.to_str()) else {
            return false;
        };
        let extension = extension.to_ascii_lowercase();
        match self {
            Self::Markdown => matches!(extension.as_str(), "md" | "markdown"),
            Self::Yaml => matches!(extension.as_str(), "yaml" | "yml"),
            Self::Json => extension == "json",
            Self::Toml => extension == "toml",
        }
    }
}

#[derive(ClapArgs)]
pub struct Args {
    #[arg(
        long,
        value_name = "FILE",
        help = "Import a JSON Resume file; use - for stdin"
    )]
    from: Option<PathBuf>,
    #[arg(
        long,
        value_name = "FORMAT",
        value_enum,
        default_value = "markdown",
        help = "Set the CV source format"
    )]
    format: Format,
    #[arg(
        short,
        long,
        value_name = "FILE",
        help = "Write the CV to FILE; default name depends on the format"
    )]
    output: Option<PathBuf>,
    #[arg(long, help = "Replace existing CV and settings files")]
    force: bool,
}

pub fn run(args: Args) -> Result<()> {
    let output = args.output.unwrap_or_else(|| args.format.default_output());
    if is_stdio(&output) {
        return Err(Error::InitStdout);
    }
    let settings_path = output.with_file_name(SETTINGS_FILE);
    if output.file_name().is_some_and(|name| name == SETTINGS_FILE) {
        return Err(Error::SettingsOutput(output));
    }
    if !args.format.accepts_extension(&output) {
        return Err(Error::InitOutputFormat {
            path: output,
            format: args.format.name(),
        });
    }
    if !args.force {
        for path in [&output, &settings_path] {
            if path.exists() {
                return Err(Error::Exists((*path).clone()));
            }
        }
    }
    let cv = if let Some(path) = args.from {
        let source = read_source(&path)?;
        parse(&source, InputFormat::JsonResume)?
    } else {
        parse(cac_io::STARTER_MARKDOWN, InputFormat::Markdown)?
    };
    let content = match args.format {
        Format::Markdown => to_markdown(&cv),
        Format::Yaml => serde_yaml_ng::to_string(&cv)?,
        Format::Json => serde_json::to_string_pretty(&cv)?,
        Format::Toml => toml::to_string_pretty(&cv)?,
    };
    let root = output
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| Error::CvFileName(output.clone()))?;
    let mut settings = serde_json::to_string_pretty(&cac_render::Settings {
        root: Some(root.into()),
        theme: Some(cac_render::DEFAULT_THEME.into()),
        ..cac_render::Settings::default()
    })?;
    settings.push('\n');
    fs::write(&output, content)?;
    fs::write(&settings_path, settings)?;
    print_result("created", &output);
    print_result("created", &settings_path);
    Ok(())
}
