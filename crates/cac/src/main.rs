use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use cac_check::{Severity, check_content};
use cac_io::{InputFormat, ParseError, export_json_resume, parse, to_markdown};
use clap::{Parser, Subcommand, ValueEnum};
use thiserror::Error;

#[derive(Parser)]
#[command(
    name = "cac",
    version,
    about = "Compile a CV from Markdown or structured data"
)]
struct Args {
    #[arg(long, global = true, help = "Write machine-readable command output")]
    json: bool,
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    #[command(about = "Create a starter cv.md")]
    Init {
        #[arg(long, value_name = "JSON_RESUME")]
        from: Option<PathBuf>,
        #[arg(short, long, default_value = "cv.md")]
        output: PathBuf,
        #[arg(long)]
        force: bool,
    },
    #[command(about = "Validate and render a CV")]
    Build {
        #[arg(default_value = "cv.md")]
        input: PathBuf,
        #[arg(
            short = 'f',
            long = "format",
            value_delimiter = ',',
            default_value = "pdf"
        )]
        formats: Vec<OutputFormat>,
        #[arg(short, long, default_value = "dist")]
        output: PathBuf,
    },
    #[command(about = "Check CV content")]
    Check {
        #[arg(default_value = "cv.md")]
        input: PathBuf,
        #[arg(long)]
        strict: bool,
    },
    #[command(about = "Convert a CV to another source format")]
    Convert {
        input: PathBuf,
        #[arg(long, value_enum)]
        to: ConversionFormat,
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    #[command(about = "List embedded themes")]
    Themes,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum OutputFormat {
    Pdf,
    Html,
    Json,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum ConversionFormat {
    Markdown,
    Yaml,
    Json,
    Toml,
    Jsonresume,
}

#[derive(Debug, Error)]
enum Error {
    #[error("{0}")]
    Io(#[from] std::io::Error),
    #[error("{0}")]
    Parse(#[from] ParseError),
    #[error("{0}")]
    Render(#[from] cac_render::RenderError),
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
}

fn main() -> ExitCode {
    let args = Args::parse();
    match run(args) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("error: {error}");
            ExitCode::FAILURE
        }
    }
}

fn run(args: Args) -> Result<(), Error> {
    match args.command {
        Command::Init {
            from,
            output,
            force,
        } => {
            if output.exists() && !force {
                return Err(Error::Exists(output));
            }
            let content = if let Some(path) = from {
                let source = fs::read_to_string(&path)?;
                to_markdown(&parse(&source, InputFormat::JsonResume)?)
            } else {
                cac_io::STARTER_MARKDOWN.into()
            };
            fs::write(&output, content)?;
            print_result(args.json, "created", &output);
        }
        Command::Build {
            input,
            formats,
            output,
        } => {
            let cv = read_cv(&input)?;
            fs::create_dir_all(&output)?;
            let stem = input
                .file_stem()
                .and_then(|value| value.to_str())
                .unwrap_or("cv");
            for format in formats {
                let path = match format {
                    OutputFormat::Pdf => {
                        let rendered = cac_render::render_pdf(&cv)?;
                        let path = output.join(format!("{stem}.pdf"));
                        fs::write(&path, rendered.bytes)?;
                        path
                    }
                    OutputFormat::Html => {
                        let path = output.join(format!("{stem}.html"));
                        fs::write(&path, cac_render::render_html(&cv))?;
                        path
                    }
                    OutputFormat::Json => {
                        let path = output.join(format!("{stem}.json"));
                        fs::write(&path, serde_json::to_vec_pretty(&cv)?)?;
                        path
                    }
                };
                print_result(args.json, "built", &path);
            }
        }
        Command::Check { input, strict } => {
            let cv = read_cv(&input)?;
            let diagnostics = check_content(&cv);
            if args.json {
                println!("{}", serde_json::to_string_pretty(&diagnostics)?);
            } else if diagnostics.is_empty() {
                println!("PASS");
            } else {
                for diagnostic in &diagnostics {
                    println!(
                        "{} {:?} {}: {}",
                        diagnostic.code, diagnostic.severity, diagnostic.path, diagnostic.message
                    );
                }
            }
            let errors = diagnostics
                .iter()
                .any(|diagnostic| diagnostic.severity == Severity::Error);
            if errors || (strict && !diagnostics.is_empty()) {
                return Err(Error::CheckFailed);
            }
        }
        Command::Convert { input, to, output } => {
            let cv = read_cv(&input)?;
            let content = match to {
                ConversionFormat::Markdown => to_markdown(&cv),
                ConversionFormat::Yaml => serde_yaml_ng::to_string(&cv)?,
                ConversionFormat::Json => serde_json::to_string_pretty(&cv)?,
                ConversionFormat::Toml => toml::to_string_pretty(&cv)?,
                ConversionFormat::Jsonresume => {
                    serde_json::to_string_pretty(&export_json_resume(&cv))?
                }
            };
            if let Some(path) = output {
                fs::write(&path, content)?;
                print_result(args.json, "created", &path);
            } else {
                println!("{content}");
            }
        }
        Command::Themes => {
            if args.json {
                println!("[\"classic\"]");
            } else {
                println!("classic (embedded)");
            }
        }
    }
    Ok(())
}

fn read_cv(path: &Path) -> Result<cac_core::CvDocument, Error> {
    let source = fs::read_to_string(path)?;
    let extension = path
        .extension()
        .and_then(|value| value.to_str())
        .ok_or_else(|| Error::Unsupported(path.into()))?;
    let format =
        InputFormat::from_extension(extension).ok_or_else(|| Error::Unsupported(path.into()))?;
    Ok(parse(&source, format)?)
}

fn print_result(json: bool, action: &str, path: &Path) {
    if json {
        println!(
            "{{\"action\":\"{action}\",\"path\":{}}}",
            serde_json::to_string(&path.display().to_string())
                .expect("a path string is valid JSON")
        );
    } else {
        println!("{} {}", action.to_ascii_uppercase(), path.display());
    }
}
