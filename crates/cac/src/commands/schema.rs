use std::fs;
use std::path::{Path, PathBuf};

use crate::error::Result;
use crate::source::print_result;

pub const ABOUT: &str = "Refresh the settings JSON Schema and validate settings.json";
pub const AFTER_HELP: &str = "The schema is written to .cac/settings.schema.json, then the current settings.json is validated.";

pub const SETTINGS_SCHEMA_REFERENCE: &str = ".cac/settings.schema.json";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SyncStatus {
    Created,
    Updated,
    Unchanged,
}

pub fn run() -> Result<()> {
    let project = Path::new("");
    let (path, status) = sync(project)?;
    print_status(&path, status);
    let settings = Path::new("settings.json");
    cac_render::Settings::from_path(settings)?;
    print_result("valid", settings);
    Ok(())
}

pub fn sync(project: &Path) -> Result<(PathBuf, SyncStatus)> {
    let path = project.join(SETTINGS_SCHEMA_REFERENCE);
    let expected = schema_bytes()?;
    let status = match fs::read(&path) {
        Ok(actual) if actual == expected => SyncStatus::Unchanged,
        Ok(_) => SyncStatus::Updated,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => SyncStatus::Created,
        Err(error) => return Err(error.into()),
    };
    if status != SyncStatus::Unchanged {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&path, expected)?;
    }
    Ok((path, status))
}

pub fn sync_and_report(project: &Path) -> Result<()> {
    let (path, status) = sync(project)?;
    print_status(&path, status);
    Ok(())
}

fn schema_bytes() -> Result<Vec<u8>> {
    let mut bytes = serde_json::to_vec_pretty(&cac_render::settings_schema())?;
    bytes.push(b'\n');
    Ok(bytes)
}

fn print_status(path: &Path, status: SyncStatus) {
    match status {
        SyncStatus::Created => print_result("created", path),
        SyncStatus::Updated => print_result("updated", path),
        SyncStatus::Unchanged => {}
    }
}
