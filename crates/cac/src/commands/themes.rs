use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use clap::{Args as ClapArgs, Subcommand};

use crate::error::{Error, Result};

pub const ABOUT: &str = "List and manage themes";
pub const AFTER_HELP: &str = "Examples:\n  cac themes list\n  cac themes install classic --local\n  cac themes remove classic --local";

#[derive(ClapArgs)]
pub struct Args {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// List embedded and installed themes
    List,
    /// Install an embedded theme
    Install(LocationArgs),
    /// Remove an installed theme
    Remove(LocationArgs),
}

#[derive(ClapArgs)]
struct LocationArgs {
    #[arg(value_name = "THEME")]
    theme: String,
    #[arg(long, help = "Use the current project's .cac/themes directory")]
    local: bool,
}

pub fn run(args: Args) -> Result<()> {
    match args.command {
        Command::List => list(),
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

fn install(args: LocationArgs) -> Result<()> {
    cac_render::validate_theme_name(&args.theme)?;
    let source = cac_render::embedded_theme_source(&args.theme)
        .ok_or_else(|| Error::EmbeddedTheme(args.theme.clone()))?;
    let directory = selected_root(args.local)?.join(&args.theme);
    fs::create_dir_all(&directory)?;
    fs::write(directory.join("theme.typ"), source)?;
    println!("INSTALLED {} ({})", args.theme, location_name(args.local));
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
