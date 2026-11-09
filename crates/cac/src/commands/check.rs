use std::fs;
use std::path::{Path, PathBuf};

use cac_check::{Severity, check_content};
use cac_core::FieldPack;
use cac_io::ParseError;
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
        packs.push(load_pack(path)?);
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

/// Read a pack file and parse it. Filesystem access lives in the `cac`
/// crate; cac-io parses the text with the path used only as an error label.
fn load_pack(path: &Path) -> Result<FieldPack> {
    let label = path.display().to_string();
    let source = fs::read_to_string(path).map_err(|error| ParseError::Pack {
        path: label.clone(),
        message: format!("could not be read: {error}"),
    })?;
    cac_io::packs::parse_pack(&source, &label).map_err(Error::from)
}
