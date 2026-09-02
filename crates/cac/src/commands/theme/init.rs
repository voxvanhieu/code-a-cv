use std::fs;
use std::io::{self, BufRead, Write};
use std::path::Path;

use clap::Args as ClapArgs;

use crate::commands::schema;
use crate::error::{Error, Result};
use crate::source::print_result;

use super::SYSTEM_THEME_NAMES;
use super::metadata::ThemeMetadata;

#[derive(ClapArgs)]
pub(super) struct Args {
    #[arg(
        value_name = "THEME_NAME",
        help = "Name the theme with lowercase letters, numbers, hyphens, or underscores"
    )]
    name: Option<String>,
    #[arg(long, value_name = "AUTHOR", help = "Set the theme author")]
    author: Option<String>,
    #[arg(
        long,
        value_name = "URL",
        help = "Set the optional HTTP or HTTPS author URL"
    )]
    author_url: Option<String>,
}

pub(super) fn run(args: Args) -> Result<()> {
    let stdin = io::stdin();
    let mut input = stdin.lock();
    let name = value(args.name, "Theme name", &mut input, validate_name)?;
    let author = value(args.author, "Author", &mut input, validate_author)?;
    let author_url = optional_url(args.author_url, &mut input)?;
    scaffold(&name, &author, author_url)
}

fn value<R: BufRead>(
    provided: Option<String>,
    prompt: &str,
    input: &mut R,
    validate: fn(&str) -> std::result::Result<(), String>,
) -> Result<String> {
    if let Some(value) = provided {
        validate(&value).map_err(Error::ThemeProject)?;
        return Ok(value.trim().into());
    }
    loop {
        eprint!("{prompt}: ");
        io::stderr().flush()?;
        let mut answer = String::new();
        if input.read_line(&mut answer)? == 0 {
            return Err(Error::ThemeProject(
                "input ended before initialization was complete".into(),
            ));
        }
        let answer = answer.trim().to_owned();
        match validate(&answer) {
            Ok(()) => return Ok(answer),
            Err(error) => eprintln!("Invalid {prompt}: {error}"),
        }
    }
}

fn optional_url<R: BufRead>(provided: Option<String>, input: &mut R) -> Result<Option<url::Url>> {
    if let Some(value) = provided {
        return parse_url(&value).map(Some).map_err(Error::ThemeProject);
    }
    loop {
        eprint!("Author URL (optional): ");
        io::stderr().flush()?;
        let mut answer = String::new();
        if input.read_line(&mut answer)? == 0 {
            return Err(Error::ThemeProject(
                "input ended before initialization was complete".into(),
            ));
        }
        let answer = answer.trim();
        if answer.is_empty() {
            return Ok(None);
        }
        match parse_url(answer) {
            Ok(url) => return Ok(Some(url)),
            Err(error) => eprintln!("Invalid Author URL: {error}"),
        }
    }
}

fn validate_name(name: &str) -> std::result::Result<(), String> {
    cac_render::validate_theme_name(name).map_err(|error| error.to_string())?;
    if SYSTEM_THEME_NAMES.contains(&name) {
        Err(format!("`{name}` is reserved"))
    } else {
        Ok(())
    }
}

fn validate_author(author: &str) -> std::result::Result<(), String> {
    if author.trim().is_empty() || author.contains(['\n', '\r']) {
        Err("expected a nonempty, single-line value".into())
    } else {
        Ok(())
    }
}

fn parse_url(value: &str) -> std::result::Result<url::Url, String> {
    let url = url::Url::parse(value).map_err(|_| "expected an absolute HTTP or HTTPS URL")?;
    if matches!(url.scheme(), "http" | "https") {
        Ok(url)
    } else {
        Err("expected an absolute HTTP or HTTPS URL".into())
    }
}

fn scaffold(name: &str, author: &str, author_url: Option<url::Url>) -> Result<()> {
    let theme_dir = Path::new(".cac/themes").join(name);
    for path in [
        Path::new("cv.md"),
        Path::new("settings.json"),
        Path::new(".cac/settings.schema.json"),
        theme_dir.as_path(),
    ] {
        if path.exists() {
            return Err(Error::Exists(path.into()));
        }
    }
    let settings = cac_render::Settings {
        schema: Some(schema::SETTINGS_SCHEMA_REFERENCE.into()),
        root: Some("cv.md".into()),
        theme: Some(name.into()),
        theme_project: Some(name.into()),
        ..Default::default()
    };
    let manifest = ThemeMetadata {
        name: name.into(),
        description: String::new(),
        author: author.into(),
        author_url,
        license: "MIT".into(),
        theme_api: cac_render::THEME_API_VERSION,
        preview: Some("preview.jpg".into()),
        files: Vec::new(),
    };
    fs::create_dir_all(theme_dir.join("assets"))?;
    fs::create_dir_all(theme_dir.join("fonts"))?;
    fs::write("cv.md", cac_io::STARTER_MARKDOWN)?;
    let mut settings_bytes = serde_json::to_vec_pretty(&settings)?;
    settings_bytes.push(b'\n');
    fs::write("settings.json", settings_bytes)?;
    schema::sync(Path::new(""))?;
    fs::write(
        theme_dir.join("theme.json"),
        super::metadata::bytes(&manifest)?,
    )?;
    fs::write(
        theme_dir.join("theme.typ"),
        "#import \"/.cac/base.typ\" as base\n\n#let theme = base.extend()\n",
    )?;
    for path in [
        Path::new("cv.md"),
        Path::new("settings.json"),
        Path::new(".cac/settings.schema.json"),
        theme_dir.join("theme.json").as_path(),
        theme_dir.join("theme.typ").as_path(),
    ] {
        print_result("created", path);
    }
    eprintln!(
        "Edit the description in {} before running `cac theme test`.",
        theme_dir.join("theme.json").display()
    );
    Ok(())
}
