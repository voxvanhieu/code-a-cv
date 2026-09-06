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
    let name = settings.theme_project.clone().ok_or_else(|| {
        Error::ThemeProject("the current directory is not a theme project".into())
    })?;
    Ok(Project {
        root,
        name,
        settings,
    })
}

pub(super) fn reject_install() -> Result<()> {
    if Path::new("settings.json").is_file()
        && cac_render::Settings::from_path(Path::new("settings.json"))?
            .theme_project
            .is_some()
    {
        return Err(Error::ThemeProject(
            "themes cannot be installed inside a theme project".into(),
        ));
    }
    Ok(())
}

pub(super) fn reject_selected_removal(name: &str) -> Result<()> {
    if Path::new("settings.json").is_file()
        && cac_render::Settings::from_path(Path::new("settings.json"))?
            .theme_project
            .as_deref()
            == Some(name)
    {
        return Err(Error::ThemeProject(format!(
            "theme `{name}` is being developed and cannot be removed"
        )));
    }
    Ok(())
}
