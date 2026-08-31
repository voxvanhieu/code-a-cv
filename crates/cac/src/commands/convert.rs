use std::fs;
use std::path::PathBuf;

use cac_io::{export_json_resume, to_markdown};
use clap::Args as ClapArgs;

use crate::cli::CvFormat;
use crate::error::Result;
use crate::source::{is_stdio, print_result, read_cv, write_stdout};

pub const ABOUT: &str = "Convert a CV to another source format";
pub const AFTER_HELP: &str = "Examples:\n  cac convert cv.md --to json\n  cac convert cv.yaml --to markdown -o cv.md\n  cat cv.json | cac convert - --input-format json --to toml";

#[derive(ClapArgs)]
pub struct Args {
    #[arg(value_name = "FILE", help = "Read a CV source file; use - for stdin")]
    input: PathBuf,
    #[arg(
        long,
        value_name = "FORMAT",
        value_enum,
        help = "Set the input format; stdin defaults to markdown"
    )]
    input_format: Option<CvFormat>,
    #[arg(
        long,
        value_name = "FORMAT",
        value_enum,
        help = "Set the output format"
    )]
    to: CvFormat,
    #[arg(
        short,
        long,
        value_name = "FILE",
        help = "Write to FILE, replacing it if present; defaults to stdout"
    )]
    output: Option<PathBuf>,
}

pub fn run(args: Args) -> Result<()> {
    let cv = read_cv(&args.input, args.input_format)?;
    let content = match args.to {
        CvFormat::Markdown => to_markdown(&cv),
        CvFormat::Yaml => serde_yaml_ng::to_string(&cv)?,
        CvFormat::Json => serde_json::to_string_pretty(&cv)?,
        CvFormat::Toml => toml::to_string_pretty(&cv)?,
        CvFormat::Jsonresume => serde_json::to_string_pretty(&export_json_resume(&cv))?,
    };
    if let Some(path) = args.output.filter(|path| !is_stdio(path)) {
        fs::write(&path, content)?;
        print_result("created", &path);
        Ok(())
    } else {
        write_stdout(&content)
    }
}
