use std::fs;
use std::path::PathBuf;

use cac_io::{InputFormat, parse, to_markdown};
use clap::Args as ClapArgs;

use crate::error::{Error, Result};
use crate::source::{is_stdio, print_result, read_source, write_stdout};

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
    #[arg(long, help = "Replace an existing output file")]
    force: bool,
}

pub fn run(args: Args) -> Result<()> {
    if !is_stdio(&args.output) && args.output.exists() && !args.force {
        return Err(Error::Exists(args.output));
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
        print_result("created", &args.output);
        Ok(())
    }
}
