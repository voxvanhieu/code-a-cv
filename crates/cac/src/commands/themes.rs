use std::collections::BTreeMap;
use std::fs;
use std::path::{Component, Path, PathBuf};

use clap::{Args as ClapArgs, Subcommand};
use serde::Deserialize;
use sha2::{Digest, Sha256};

use crate::error::{Error, Result};

const DEFAULT_REGISTRY: &str = "https://raw.githubusercontent.com/voxvanhieu/code-a-cv/main/themes";
const REGISTRY_ENV: &str = "CAC_THEME_REGISTRY";
const SYSTEM_THEME_NAMES: &[&str] = &["classic", "base", "main"];

pub const ABOUT: &str = "Find, install, and manage themes";
pub const AFTER_HELP: &str = "Examples:\n  cac themes list\n  cac themes search blue\n  cac themes info classic-blue\n  cac themes install classic-blue --local\n  cac themes remove classic-blue --local";

#[derive(ClapArgs)]
pub struct Args {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// List embedded and installed themes
    List,
    /// Search downloadable themes
    Search(SearchArgs),
    /// Show a downloadable theme's metadata
    Info(ThemeArgs),
    /// Install a downloadable theme
    Install(InstallArgs),
    /// Remove an installed theme
    Remove(LocationArgs),
}

#[derive(ClapArgs)]
struct SearchArgs {
    #[arg(value_name = "QUERY", help = "Filter by theme name or description")]
    query: Option<String>,
}

#[derive(ClapArgs)]
struct ThemeArgs {
    #[arg(value_name = "THEME")]
    theme: String,
}

#[derive(ClapArgs)]
struct InstallArgs {
    #[command(flatten)]
    location: LocationArgs,
    #[arg(long, help = "Replace an installed theme")]
    force: bool,
}

#[derive(ClapArgs)]
struct LocationArgs {
    #[arg(value_name = "THEME")]
    theme: String,
    #[arg(long, help = "Use the current project's .cac/themes directory")]
    local: bool,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Registry {
    schema_version: u32,
    themes: Vec<ThemeSummary>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ThemeSummary {
    name: String,
    description: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ThemeMetadata {
    name: String,
    description: String,
    author: String,
    license: String,
    theme_api: u32,
    files: Vec<ThemeFile>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ThemeFile {
    path: String,
    sha256: String,
}

pub fn run(args: Args) -> Result<()> {
    match args.command {
        Command::List => list(),
        Command::Search(args) => search(args),
        Command::Info(args) => info(args),
        Command::Install(args) => install(args),
        Command::Remove(args) => remove(args),
    }
}

fn list() -> Result<()> {
    let mut themes = cac_render::EMBEDDED_THEME_NAMES
        .iter()
        .map(|name| ((*name).to_owned(), vec!["embedded"]))
        .collect();
    collect_themes(&user_root()?, "user", &mut themes)?;
    collect_themes(&local_root()?, "project", &mut themes)?;
    for (name, sources) in themes {
        println!("{name} ({})", sources.join(", "));
    }
    Ok(())
}

fn collect_themes(
    root: &Path,
    source: &'static str,
    themes: &mut BTreeMap<String, Vec<&'static str>>,
) -> Result<()> {
    if !root.exists() {
        return Ok(());
    }
    for entry in fs::read_dir(root)? {
        let entry = entry?;
        if entry.path().join("theme.typ").is_file()
            && let Some(name) = entry.file_name().to_str()
            && cac_render::validate_theme_name(name).is_ok()
        {
            themes.entry(name.into()).or_default().push(source);
        }
    }
    Ok(())
}

fn search(args: SearchArgs) -> Result<()> {
    let registry = registry()?;
    let query = args.query.unwrap_or_default().to_ascii_lowercase();
    for theme in registry.themes {
        if query.is_empty()
            || theme.name.to_ascii_lowercase().contains(&query)
            || theme.description.to_ascii_lowercase().contains(&query)
        {
            println!("{}: {}", theme.name, theme.description);
        }
    }
    Ok(())
}

fn info(args: ThemeArgs) -> Result<()> {
    cac_render::validate_theme_name(&args.theme)?;
    let registry = registry()?;
    find_theme(&registry, &args.theme)?;
    let theme = metadata(&args.theme)?;
    println!("NAME {}", theme.name);
    println!("DESCRIPTION {}", theme.description);
    println!("AUTHOR {}", theme.author);
    println!("LICENSE {}", theme.license);
    println!("THEME API {}", theme.theme_api);
    Ok(())
}

fn install(args: InstallArgs) -> Result<()> {
    let name = &args.location.theme;
    cac_render::validate_theme_name(name)?;
    if SYSTEM_THEME_NAMES.contains(&name.as_str()) {
        return Err(Error::SystemThemeName(name.clone()));
    }
    let root = selected_root(args.location.local)?;
    let directory = root.join(name);
    if directory.exists() && !args.force {
        return Err(Error::Exists(directory));
    }

    install_downloadable(name, &directory, args.force)?;
    println!(
        "INSTALLED {} ({})",
        name,
        location_name(args.location.local)
    );
    Ok(())
}

fn install_downloadable(name: &str, directory: &Path, force: bool) -> Result<()> {
    let registry = registry()?;
    find_theme(&registry, name)?;
    let theme = metadata(name)?;
    let base = registry_base();
    let files = theme
        .files
        .iter()
        .map(|file| {
            let bytes = fetch(&resource(&base, &format!("{name}/{}", file.path)))?;
            let actual = format!("{:x}", Sha256::digest(&bytes));
            if actual != file.sha256 {
                return Err(Error::ThemeChecksum {
                    theme: name.into(),
                    path: file.path.clone(),
                });
            }
            Ok((file, bytes))
        })
        .collect::<Result<Vec<_>>>()?;

    replace_directory(directory, force)?;
    for (file, bytes) in files {
        let path = directory.join(&file.path);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(path, bytes)?;
    }
    Ok(())
}

fn replace_directory(directory: &Path, force: bool) -> Result<()> {
    if directory.exists() {
        if !force {
            return Err(Error::Exists(directory.into()));
        }
        fs::remove_dir_all(directory)?;
    }
    Ok(())
}

fn remove(args: LocationArgs) -> Result<()> {
    cac_render::validate_theme_name(&args.theme)?;
    let directory = selected_root(args.local)?.join(&args.theme);
    if !directory.join("theme.typ").is_file() {
        return Err(Error::ThemeNotInstalled(args.theme));
    }
    fs::remove_dir_all(&directory)?;
    println!("REMOVED {} ({})", args.theme, location_name(args.local));
    Ok(())
}

fn registry() -> Result<Registry> {
    let bytes = fetch(&resource(&registry_base(), "index.json"))?;
    let registry: Registry = serde_json::from_slice(&bytes)
        .map_err(|source| Error::ThemeRegistry(format!("invalid index.json: {source}")))?;
    if registry.schema_version != 1 {
        return Err(Error::ThemeRegistry(format!(
            "unsupported registry schema version {}",
            registry.schema_version
        )));
    }
    let mut names = std::collections::BTreeSet::new();
    for theme in &registry.themes {
        cac_render::validate_theme_name(&theme.name)?;
        if theme.description.trim().is_empty() || !names.insert(&theme.name) {
            return Err(Error::ThemeRegistry(format!(
                "theme `{}` has invalid search metadata",
                theme.name
            )));
        }
    }
    Ok(registry)
}

fn find_theme<'a>(registry: &'a Registry, name: &str) -> Result<&'a ThemeSummary> {
    registry
        .themes
        .iter()
        .find(|theme| theme.name == name)
        .ok_or_else(|| Error::DownloadableTheme(name.into()))
}

fn metadata(name: &str) -> Result<ThemeMetadata> {
    let resource = resource(&registry_base(), &format!("{name}/theme.json"));
    let bytes = fetch(&resource)?;
    let theme: ThemeMetadata = serde_json::from_slice(&bytes)
        .map_err(|source| Error::ThemeRegistry(format!("invalid `{resource}`: {source}")))?;
    if theme.name != name {
        return Err(Error::ThemeRegistry(format!(
            "manifest name `{}` does not match `{name}`",
            theme.name
        )));
    }
    validate_metadata(&theme)?;
    Ok(theme)
}

fn validate_metadata(theme: &ThemeMetadata) -> Result<()> {
    cac_render::validate_theme_name(&theme.name)?;
    if theme.theme_api != 1 {
        return Err(Error::ThemeRegistry(format!(
            "theme `{}` uses unsupported theme API {}",
            theme.name, theme.theme_api
        )));
    }
    if theme.description.trim().is_empty()
        || theme.author.trim().is_empty()
        || theme.license.trim().is_empty()
        || theme.files.is_empty()
        || !theme.files.iter().any(|file| file.path == "theme.typ")
    {
        return Err(Error::ThemeRegistry(format!(
            "theme `{}` has incomplete metadata",
            theme.name
        )));
    }
    let mut paths = std::collections::BTreeSet::new();
    for file in &theme.files {
        let path = Path::new(&file.path);
        if path.is_absolute()
            || path
                .components()
                .any(|component| !matches!(component, Component::Normal(_)))
            || !paths.insert(&file.path)
            || file.sha256.len() != 64
            || !file
                .sha256
                .bytes()
                .all(|byte| byte.is_ascii_digit() || matches!(byte, b'a'..=b'f'))
        {
            return Err(Error::ThemeRegistry(format!(
                "theme `{}` has an invalid file entry",
                theme.name
            )));
        }
    }
    Ok(())
}

fn registry_base() -> String {
    std::env::var(REGISTRY_ENV).unwrap_or_else(|_| DEFAULT_REGISTRY.into())
}

fn resource(base: &str, relative: &str) -> String {
    format!("{}/{relative}", base.trim_end_matches('/'))
}

fn fetch(resource: &str) -> Result<Vec<u8>> {
    if let Some(path) = resource.strip_prefix("file://") {
        return Ok(fs::read(path)?);
    }
    let mut response = ureq::get(resource)
        .call()
        .map_err(|error| Error::ThemeRegistry(format!("could not fetch `{resource}`: {error}")))?;
    response
        .body_mut()
        .read_to_vec()
        .map_err(|error| Error::ThemeRegistry(format!("could not read `{resource}`: {error}")))
}

fn selected_root(local: bool) -> Result<PathBuf> {
    if local { local_root() } else { user_root() }
}

fn local_root() -> Result<PathBuf> {
    Ok(std::env::current_dir()?.join(".cac/themes"))
}

fn user_root() -> Result<PathBuf> {
    std::env::var_os(if cfg!(windows) { "USERPROFILE" } else { "HOME" })
        .map(PathBuf::from)
        .map(|path| path.join(".cac/themes"))
        .ok_or(Error::HomeDirectory)
}

fn location_name(local: bool) -> &'static str {
    if local { "project" } else { "user" }
}
