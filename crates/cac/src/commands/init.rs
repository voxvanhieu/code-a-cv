use std::fs;
use std::path::PathBuf;

use cac_io::{InputFormat, parse, to_markdown};
use clap::Args as ClapArgs;

use crate::error::{Error, Result};
use crate::source::{is_stdio, print_result, read_source, write_stdout};

const SETTINGS_FILE: &str = "settings.json";

pub const ABOUT: &str = "Create a starter CV or import JSON Resume data";
pub const AFTER_HELP: &str =
    "Examples:\n  cac init\n  cac init --from resume.json -o cv.md\n  cac init -o -";

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
        help = "Write the Markdown CV to FILE; use - for stdout"
    )]
    output: PathBuf,
    #[arg(long, help = "Replace existing CV and settings files")]
    force: bool,
}

pub fn run(args: Args) -> Result<()> {
    let settings_path =
        (!is_stdio(&args.output)).then(|| args.output.with_file_name(SETTINGS_FILE));
    if args
        .output
        .file_name()
        .is_some_and(|name| name == SETTINGS_FILE)
    {
        return Err(Error::SettingsOutput(args.output));
    }
    if !args.force
        && let Some(settings_path) = &settings_path
    {
        for path in [&args.output, settings_path] {
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
    if is_stdio(&args.output) {
        write_stdout(&content)
    } else {
        fs::write(&args.output, content)?;
        let settings_path = settings_path.expect("file output has a settings path");
        let settings = format!("{{\n  \"theme\": \"{}\"\n}}\n", cac_render::DEFAULT_THEME);
        fs::write(&settings_path, settings)?;
        print_result("created", &args.output);
        print_result("created", &settings_path);
        Ok(())
    }
}
