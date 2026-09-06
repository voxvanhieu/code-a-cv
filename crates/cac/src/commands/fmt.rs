use std::fs;
use std::path::PathBuf;

use cac_io::{InputFormat, parse};
use clap::Args as ClapArgs;

use crate::cli::CvFormat;
use crate::error::{Error, Result};
use crate::source::{is_stdio, print_result, read_source, write_stdout};

pub const ABOUT: &str = "Format a CV in its existing source format";
pub const AFTER_HELP: &str = "Examples:\n  cac fmt cv.md\n  cac fmt cv.json\n  cac fmt cv.md --dry-run\n  cat cv.md | cac fmt -";

#[derive(ClapArgs)]
pub struct Args {
    #[arg(help = "Read a CV source; defaults to settings.json root, then cv.md; use - for stdin")]
    input: Option<PathBuf>,
    #[arg(
        long,
        value_enum,
        help = "Set the input format; stdin defaults to markdown"
    )]
    input_format: Option<CvFormat>,
    #[arg(
        long,
        help = "Report whether formatting is needed without writing; fail if changes are needed"
    )]
    dry_run: bool,
}

pub fn run(args: Args) -> Result<()> {
    let input = match args.input {
        Some(input) => input,
        None => {
            let settings = PathBuf::from("settings.json");
            if settings.is_file() {
                cac_render::Settings::from_path(&settings)?
                    .root
                    .map(PathBuf::from)
                    .unwrap_or_else(|| PathBuf::from("cv.md"))
            } else {
                PathBuf::from("cv.md")
            }
        }
    };
    let format = if let Some(format) = args.input_format {
        format.into()
    } else if is_stdio(&input) {
        InputFormat::Markdown
    } else {
        input
            .extension()
            .and_then(|value| value.to_str())
            .and_then(InputFormat::from_extension)
            .ok_or_else(|| Error::Unsupported(input.clone()))?
    };
    let source = read_source(&input)?;
    let cv = parse(&source, format).map_err(|error| Error::SourceParse {
        path: input.clone(),
        error,
    })?;
    let formatted = match format {
        InputFormat::Markdown => format_markdown(&source),
        InputFormat::Json | InputFormat::JsonResume => {
            let value: serde_json::Value = serde_json::from_str(&source)?;
            format!("{}\n", serde_json::to_string_pretty(&value)?)
        }
        InputFormat::Yaml => {
            let value: serde_yaml_ng::Value = serde_yaml_ng::from_str(&source)?;
            serde_yaml_ng::to_string(&value)?
        }
        InputFormat::Toml => {
            let value: toml::Value = toml::from_str(&source).map_err(cac_io::ParseError::from)?;
            toml::to_string_pretty(&value)?
        }
    };
    if parse(&formatted, format)? != cv {
        return Err(cac_io::ParseError::Validation(
            "formatting would change CV data; source left unchanged".into(),
        )
        .into());
    }
    if args.dry_run {
        if source != formatted {
            return Err(cac_io::ParseError::Validation(
                "Source needs formatting; run cac fmt FILE".into(),
            )
            .into());
        }
        println!("PASS");
        Ok(())
    } else if !is_stdio(&input) {
        fs::write(&input, formatted).map_err(Error::from)?;
        print_result("formatted", &input);
        Ok(())
    } else {
        write_stdout(&formatted)
    }
}

// Keep syntax and metadata boundaries intact, including Markdown hard breaks.
fn format_markdown(source: &str) -> String {
    let mut output = String::new();
    for line in source.lines() {
        let trimmed = line.trim_end_matches([' ', '\t']);
        output.push_str(trimmed);
        if !trimmed.is_empty() && line.ends_with("  ") {
            output.push_str("  ");
        }
        output.push('\n');
    }
    while output.ends_with("\n\n") {
        output.pop();
    }
    output
}
