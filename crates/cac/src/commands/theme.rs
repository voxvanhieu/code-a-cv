use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use clap::{Args as ClapArgs, Subcommand};
use serde::Deserialize;
use sha2::{Digest, Sha256};

use crate::error::{Error, Result};

const DEFAULT_REGISTRY: &str = "https://raw.githubusercontent.com/voxvanhieu/code-a-cv/main/themes";
const REGISTRY_ENV: &str = "CAC_THEME_REGISTRY";
const SYSTEM_THEME_NAMES: &[&str] = &["classic", "base", "main"];

mod init;
mod metadata;
mod pack;
mod project;
mod test;

pub const ABOUT: &str = "Find, install, and manage themes";
pub const AFTER_HELP: &str = "Examples:\n  cac theme init my-theme --author 'Ada Lovelace'\n  cac theme test\n  cac theme pack\n  cac theme list\n  cac theme install classic-blue --local";

#[derive(ClapArgs)]
pub struct Args {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Create a theme-development project
    Init(init::Args),
    /// Validate and generate the current theme project
    Test(test::Args),
    /// Test and package the current theme project
    Pack,
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
    /// Form themes are listed separately: they reject sections by design,
    /// so the universal-theme search does not surface them. Schema 2
    /// carries them; find_theme searches both lists.
    #[serde(default)]
    forms: Vec<ThemeSummary>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ThemeSummary {
    name: String,
    description: String,
    /// The minimum cac version that renders the theme (schema 2). Old
    /// clients never see the field: they keep reading the frozen schema-1
    /// index, which only lists themes they can render.
    #[serde(default)]
    min_cac: Option<String>,
}

/// Whether the running binary satisfies a `major.minor` requirement.
fn version_at_least(required: &str) -> bool {
    let parse = |text: &str| -> Vec<u64> {
        text.split('.')
            .map(|part| part.trim().parse::<u64>().unwrap_or(0))
            .collect()
    };
    let required = parse(required);
    let running = parse(env!("CARGO_PKG_VERSION"));
    for index in 0..required.len().max(running.len()) {
        let need = required.get(index).copied().unwrap_or(0);
        let have = running.get(index).copied().unwrap_or(0);
        if have > need {
            return true;
        }
        if have < need {
            return false;
        }
    }
    true
}

pub fn run(args: Args) -> Result<()> {
    match args.command {
        Command::Init(args) => init::run(args),
        Command::Test(args) => test::run(args),
        Command::Pack => pack::run(),
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
    let matches = |theme: &ThemeSummary| {
        query.is_empty()
            || theme.name.to_ascii_lowercase().contains(&query)
            || theme.description.to_ascii_lowercase().contains(&query)
    };
    for theme in registry.themes.iter().filter(|theme| matches(theme)) {
        println!("{}: {}", theme.name, theme.description);
    }
    // Form themes surface with their kind, so `cac theme search` finds
    // them and `install` works through the same lookup.
    for theme in registry.forms.iter().filter(|theme| matches(theme)) {
        println!("{} (form): {}", theme.name, theme.description);
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
    if let Some(author_url) = theme.author_url {
        println!("AUTHOR URL {author_url}");
    }
    println!("LICENSE {}", theme.license);
    if let Some(preview) = theme.preview {
        println!("PREVIEW {preview}");
    }
    Ok(())
}

fn install(args: InstallArgs) -> Result<()> {
    let name = &args.location.theme;
    cac_render::validate_theme_name(name)?;
    if SYSTEM_THEME_NAMES.contains(&name.as_str()) {
        return Err(Error::SystemThemeName(name.clone()));
    }
    project::reject_install()?;
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
    let summary = find_theme(&registry, name)?;
    if let Some(required) = &summary.min_cac
        && !version_at_least(required)
    {
        return Err(Error::ThemeRegistry(format!(
            "theme `{name}` requires cac {required} or newer; you are running {}",
            env!("CARGO_PKG_VERSION")
        )));
    }
    let theme = metadata(name)?;
    let base = registry_base();
    let files = theme
        .files
        .iter()
        .map(|file| {
            let bytes = fetch(&resource(&base, &format!("{name}/{}", file.path)))?;
            let actual = sha256_hex(&bytes);
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
    project::reject_selected_removal(&args.theme)?;
    let directory = selected_root(args.local)?.join(&args.theme);
    if !directory.join("theme.typ").is_file() {
        return Err(Error::ThemeNotInstalled(args.theme));
    }
    fs::remove_dir_all(&directory)?;
    println!("REMOVED {} ({})", args.theme, location_name(args.local));
    Ok(())
}

fn registry() -> Result<Registry> {
    // Schema 2 first, at its own path: released 0.3 clients only ever
    // read `index.json`, which stays frozen at the spellings they can
    // render, so the schema-2 file can carry new themes, forms, and
    // `min_cac` without breaking a single installed binary.
    if let Ok(bytes) = fetch(&resource(&registry_base(), "index.v2.json")) {
        return parse_registry(&bytes, "index.v2.json");
    }
    let bytes = fetch(&resource(&registry_base(), "index.json"))?;
    parse_registry(&bytes, "index.json")
}

fn parse_registry(bytes: &[u8], file: &str) -> Result<Registry> {
    let registry: Registry = serde_json::from_slice(bytes)
        .map_err(|source| Error::ThemeRegistry(format!("invalid {file}: {source}")))?;
    if registry.schema_version != 1 && registry.schema_version != 2 {
        return Err(Error::ThemeRegistry(format!(
            "unsupported registry schema version {}",
            registry.schema_version
        )));
    }
    let mut names = std::collections::BTreeSet::new();
    for theme in registry.themes.iter().chain(registry.forms.iter()) {
        cac_render::validate_theme_name(&theme.name)?;
        if theme.description.trim().is_empty() || !names.insert(theme.name.as_str()) {
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
        .chain(registry.forms.iter())
        .find(|theme| theme.name == name)
        .ok_or_else(|| Error::DownloadableTheme(name.into()))
}

fn metadata(name: &str) -> Result<metadata::ThemeMetadata> {
    let resource = resource(&registry_base(), &format!("{name}/theme.json"));
    let bytes = fetch(&resource)?;
    let theme: metadata::ThemeMetadata = serde_json::from_slice(&bytes)
        .map_err(|source| Error::ThemeRegistry(format!("invalid `{resource}`: {source}")))?;
    if theme.name != name {
        return Err(Error::ThemeRegistry(format!(
            "manifest name `{}` does not match `{name}`",
            theme.name
        )));
    }
    metadata::validate_packaged(&theme).map_err(Error::ThemeRegistry)?;
    Ok(theme)
}

fn registry_base() -> String {
    std::env::var(REGISTRY_ENV).unwrap_or_else(|_| DEFAULT_REGISTRY.into())
}

fn resource(base: &str, relative: &str) -> String {
    format!("{}/{relative}", base.trim_end_matches('/'))
}

fn fetch(resource: &str) -> Result<Vec<u8>> {
    if let Some(path) = file_resource_path(resource) {
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

/// The local path of a `file://` resource, or `None` for other schemes.
///
/// Windows file URLs keep a drive letter behind an empty authority
/// (`file:///C:/Users/...`), so the path arrives with a leading slash that
/// no Windows API accepts. Percent escapes are decoded the way browsers
/// write `file:` URLs.
fn file_resource_path(resource: &str) -> Option<PathBuf> {
    let path = resource.strip_prefix("file://")?;
    let path = match path.strip_prefix('/') {
        // `file:///C:/...` carries the drive letter behind the slash;
        // `file:///tmp/...` keeps the absolute root.
        Some(stripped) if is_windows_drive(stripped) => stripped,
        _ => path,
    };
    Some(PathBuf::from(percent_decode(path)))
}

fn is_windows_drive(path: &str) -> bool {
    let bytes = path.as_bytes();
    bytes.len() >= 2 && bytes[0].is_ascii_alphabetic() && bytes[1] == b':'
}

fn percent_decode(path: &str) -> String {
    let bytes = path.as_bytes();
    let mut decoded = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == b'%'
            && index + 2 < bytes.len()
            && bytes[index + 1].is_ascii_hexdigit()
            && bytes[index + 2].is_ascii_hexdigit()
        {
            let hex = std::str::from_utf8(&bytes[index + 1..index + 3])
                .expect("ASCII slice from checked bytes");
            if let Ok(byte) = u8::from_str_radix(hex, 16) {
                decoded.push(byte);
                index += 3;
                continue;
            }
        }
        decoded.push(bytes[index]);
        index += 1;
    }
    String::from_utf8_lossy(&decoded).into_owned()
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

fn sha256_hex(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

#[cfg(test)]
#[path = "../../tests/unit/theme_fetch.rs"]
mod theme_fetch_tests;
