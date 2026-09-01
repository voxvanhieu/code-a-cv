use std::fs;
use std::path::PathBuf;

use cac_io::{InputFormat, parse, to_markdown};
use clap::Args as ClapArgs;

use crate::error::{Error, Result};
use crate::source::{is_stdio, print_result, read_source};

const SETTINGS_FILE: &str = "settings.json";

pub const ABOUT: &str = "Create a starter CV and settings or import JSON Resume data";
pub const AFTER_HELP: &str = "Examples:\n  cac init\n  cac init --from resume.json -o cv.md";

#[derive(ClapArgs)]
pub struct Args {
    #[arg(
        long,
        value_name = "FILE",
        help = "Import a JSON Resume file; use - for stdin"
    )]
    from: Option<PathBuf>,
    #[arg(
        short,
        long,
        value_name = "FILE",
        default_value = "cv.md",
        help = "Write the Markdown CV to FILE"
    )]
    output: PathBuf,
    #[arg(long, help = "Replace existing CV and settings files")]
    force: bool,
}

pub fn run(args: Args) -> Result<()> {
    if is_stdio(&args.output) {
        return Err(Error::InitStdout);
    }
    let settings_path = args.output.with_file_name(SETTINGS_FILE);
    if args
        .output
        .file_name()
        .is_some_and(|name| name == SETTINGS_FILE)
    {
        return Err(Error::SettingsOutput(args.output));
    }
    if !args.force {
        for path in [&args.output, &settings_path] {
            if path.exists() {
                return Err(Error::Exists((*path).clone()));
            }
        }
    }
    let content = if let Some(path) = args.from {
        let source = read_source(&path)?;
        to_markdown(&parse(&source, InputFormat::JsonResume)?)
    } else {
        cac_io::STARTER_MARKDOWN.into()
    };
    let root = args
        .output
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| Error::CvFileName(args.output.clone()))?;
    let mut settings = serde_json::to_string_pretty(&cac_render::Settings {
        root: Some(root.into()),
        theme: Some(cac_render::DEFAULT_THEME.into()),
        ..cac_render::Settings::default()
    })?;
    settings.push('\n');
    fs::write(&args.output, content)?;
    fs::write(&settings_path, settings)?;
    print_result("created", &args.output);
    print_result("created", &settings_path);
    Ok(())
}
