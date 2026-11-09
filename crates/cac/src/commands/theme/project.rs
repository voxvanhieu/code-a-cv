use std::path::{Path, PathBuf};

use crate::error::{Error, Result};

pub(super) struct Project {
    pub root: PathBuf,
    pub name: String,
    pub settings: cac_render::Settings,
}

pub(super) fn current() -> Result<Project> {
    let root = std::env::current_dir()?;
    let settings = cac_render::Settings::from_path(&root.join("settings.json"))?;
    let name = settings.theme.clone().ok_or_else(|| {
        Error::ThemeProject("the current directory is not a theme project".into())
    })?;
    let directory = root.join(".cac/themes").join(&name);
    if !directory.is_dir() {
        return Err(Error::ThemeProject(
            "the current directory is not a theme project; no matching theme under .cac/themes"
                .into(),
        ));
    }
    Ok(Project {
        root,
        name,
        settings,
    })
}

/// The one theme-project rule, shared by every guard: the current
/// directory carries `theme init`'s scaffold — `.cac/main.typ`, the
/// standalone Typst bridge. A consumer project that merely installed a
/// local theme never has it, so installs and removals stay available
/// there. The `themeProject` marker is retired (Y1); keying a guard on it
/// would let `theme remove` delete the theme under development in every
/// freshly initialized project.
fn is_theme_project() -> Result<Option<bool>> {
    if !Path::new("settings.json").is_file() {
        return Ok(None);
    }
    Ok(Some(Path::new(".cac/main.typ").is_file()))
}

pub(super) fn reject_install() -> Result<()> {
    if is_theme_project()? == Some(true) {
        return Err(Error::ThemeProject(
            "themes cannot be installed inside a theme project".into(),
        ));
    }
    Ok(())
}

pub(super) fn reject_selected_removal(name: &str) -> Result<()> {
    if !Path::new("settings.json").is_file() {
        return Ok(());
    }
    let settings = cac_render::Settings::from_path(Path::new("settings.json"))?;
    if settings.theme.as_deref() == Some(name) && Path::new(".cac/themes").join(name).is_dir() {
        return Err(Error::ThemeProject(format!(
            "theme `{name}` is being developed and cannot be removed"
        )));
    }
    Ok(())
}
