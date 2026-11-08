use std::fs;
use std::path::{Path, PathBuf};

use clap::{Args as ClapArgs, ValueEnum};

use crate::cli::CvFormat;
use crate::commands::schema;
use crate::error::Result;
use crate::source::{input_stem, is_stdio, print_result, read_cv};

pub const ABOUT: &str = "Validate and render a CV as PDF or HTML";
pub const AFTER_HELP: &str =
    "Examples:\n  cac build\n  cac build cv.yaml --format pdf,html\n  cat cv.md | cac build -";

#[derive(ClapArgs)]
pub struct Args {
    #[arg(
        value_name = "FILE",
        help = "Read a CV source file; defaults to settings.json root, then cv.md; use - for stdin"
    )]
    input: Option<PathBuf>,
    #[arg(
        long,
        value_name = "FORMAT",
        value_enum,
        help = "Set the input format; stdin defaults to markdown"
    )]
    input_format: Option<CvFormat>,
    #[arg(
        short = 'f',
        long = "format",
        value_name = "FORMAT",
        value_delimiter = ',',
        default_value = "pdf",
        help = "Build one or more output formats"
    )]
    formats: Vec<OutputFormat>,
    #[arg(
        short,
        long,
        value_name = "DIR",
        default_value = "offering",
        help = "Write artifacts to DIR, replacing existing artifacts"
    )]
    output: PathBuf,
    #[arg(
        long,
        value_name = "FILE",
        help = "Read rendering settings from FILE; defaults to settings.json beside the CV"
    )]
    settings: Option<PathBuf>,
    #[arg(
        long,
        value_name = "TAGS",
        value_delimiter = ',',
        help = "Build a job-specific variant: keep sections and entries tagged with any of TAGS, drop the rest, and name the artifact after the variant"
    )]
    tags: Vec<String>,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum OutputFormat {
    Pdf,
    Html,
}

pub fn run(args: Args) -> Result<()> {
    let current_directory = std::env::current_dir()?;
    let settings_path = args.settings.or_else(|| {
        let directory = args
            .input
            .as_deref()
            .and_then(|input| (!is_stdio(input)).then(|| input.parent()))
            .flatten()
            .unwrap_or(&current_directory);
        let path = directory.join("settings.json");
        path.is_file().then_some(path)
    });
    let schema_project = if let Some(project) = settings_path.as_deref().and_then(Path::parent) {
        project.to_path_buf()
    } else if let Some(input) = args.input.as_deref() {
        project_directory(input)?
    } else {
        current_directory.clone()
    };
    schema::sync_and_report(&schema_project)?;
    let settings = settings_path
        .as_deref()
        .map(cac_render::Settings::from_path)
        .transpose()?
        .unwrap_or_default();
    let input = args.input.unwrap_or_else(|| {
        settings.root.as_deref().map_or_else(
            || PathBuf::from("cv.md"),
            |root| {
                settings_path
                    .as_deref()
                    .and_then(Path::parent)
                    .unwrap_or(&current_directory)
                    .join(root)
            },
        )
    });
    let cv = read_cv(&input, args.input_format)?;
    let mut stem = settings
        .naming
        .as_deref()
        .map_or_else(|| input_stem(&input).to_owned(), str::to_owned);
    let cv = if args.tags.is_empty() {
        cv
    } else {
        let tags: std::collections::BTreeSet<String> = args
            .tags
            .iter()
            .map(|tag| tag.trim().to_string())
            .filter(|tag| !tag.is_empty())
            .collect();
        let listed = tags.iter().cloned().collect::<Vec<_>>().join(", ");
        let kept = cv
            .variant(&tags)
            .ok_or(crate::error::Error::NoTagMatches(listed))?;
        let variant = args
            .tags
            .iter()
            .map(|tag| tag.trim())
            .filter(|tag| !tag.is_empty())
            .collect::<Vec<_>>()
            .join("-");
        println!("VARIANT {variant}");
        stem = format!("{stem}-{variant}");
        kept
    };
    let project = project_directory(&input)?;
    let resolved_style = cac_render::resolved_style_for(&settings, settings.theme_name())
        .map_err(|error| cac_render::SettingsError::Style(error.to_string()))?;
    let render_options = cac_render::RenderOptions {
        project_dir: Some(project.join(".cac")),
        user_dir: user_cac_directory(),
        settings,
    };
    fs::create_dir_all(&args.output)?;
    for format in args.formats {
        let path = match format {
            OutputFormat::Pdf => {
                let rendered = cac_render::render_pdf_with_options(&cv, &render_options)?;
                println!("THEME {} ({})", rendered.theme, rendered.theme_source);
                let path = args.output.join(format!("{stem}.pdf"));
                fs::write(&path, rendered.bytes)?;
                path
            }
            OutputFormat::Html => {
                let path = args.output.join(format!("{stem}.html"));
                fs::write(
                    &path,
                    cac_render::render_html_for_sheet(&cv, &resolved_style),
                )?;
                path
            }
        };
        print_result("built", &path);
    }
    Ok(())
}

fn project_directory(input: &Path) -> Result<PathBuf> {
    if is_stdio(input) {
        return Ok(std::env::current_dir()?);
    }
    // A bare "cv.md" has an empty parent; the "." keeps the jailed project
    // lookups absolute instead of cwd-relative.
    let parent = input
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty());
    Ok(parent
        .map(|parent| parent.to_path_buf())
        .unwrap_or_else(|| PathBuf::from(".")))
}

fn user_cac_directory() -> Option<PathBuf> {
    std::env::var_os(if cfg!(windows) { "USERPROFILE" } else { "HOME" })
        .map(PathBuf::from)
        .map(|path| path.join(".cac"))
}
