use std::path::PathBuf;

use cac_check::{Severity, check_content};
use clap::Args as ClapArgs;

use crate::cli::CvFormat;
use crate::error::{Error, Result};
use crate::source::read_cv;

pub const ABOUT: &str = "Check CV content";
pub const AFTER_HELP: &str =
    "Examples:\n  cac check\n  cac check cv.json --strict\n  cat cv.md | cac check -";

#[derive(ClapArgs)]
pub struct Args {
    #[arg(
        value_name = "FILE",
        default_value = "cv.md",
        help = "Read a CV source file; use - for stdin"
    )]
    input: PathBuf,
    #[arg(
        long,
        value_name = "FORMAT",
        value_enum,
        help = "Set the input format; stdin defaults to markdown"
    )]
    input_format: Option<CvFormat>,
    #[arg(long, help = "Fail when any diagnostic is reported")]
    strict: bool,
    #[arg(long, help = "Show the parsed CV fields before content diagnostics")]
    explain: bool,
    #[arg(
        long,
        value_name = "PATH",
        help = "Validate open fields against a field pack (fields.toml); repeatable"
    )]
    pack: Vec<PathBuf>,
}

pub fn run(args: Args) -> Result<()> {
    let cv = read_cv(&args.input, args.input_format)?;
    if args.explain {
        println!("{}", serde_json::to_string_pretty(&cv)?);
    }
    let mut diagnostics = check_content(&cv);
    let mut packs = Vec::new();
    for path in &args.pack {
        packs.push(cac_io::packs::load_pack_file(path)?);
    }
    diagnostics.extend(cac_check::packs::check_packs(&cv, &packs));
    for diagnostic in &diagnostics {
        println!(
            "{} {:?} {}: {}",
            diagnostic.code, diagnostic.severity, diagnostic.path, diagnostic.message
        );
    }
    let errors = diagnostics
        .iter()
        .any(|diagnostic| diagnostic.severity == Severity::Error);
    let failed = errors || (args.strict && !diagnostics.is_empty());
    println!("{}", if failed { "FAIL" } else { "PASS" });
    if failed {
        Err(Error::CheckFailed)
    } else {
        Ok(())
    }
}
