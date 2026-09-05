use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io::Write as _;
use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};

use crate::error::{Error, Result};
use crate::source::{print_result, read_cv};

use super::metadata::{self, ThemeFile, ThemeMetadata};
use super::project;

pub(super) struct TestedTheme {
    pub project: project::Project,
    pub theme_dir: PathBuf,
}

pub(super) fn run() -> Result<()> {
    test_current()?;
    Ok(())
}

pub(super) fn test_current() -> Result<TestedTheme> {
    let project = project::current()?;
    let theme_dir = project.root.join(".cac/themes").join(&project.name);
    for path in [
        project.root.join(".cac"),
        project.root.join(".cac/themes"),
        theme_dir.clone(),
    ] {
        if fs::symlink_metadata(&path)?.file_type().is_symlink() {
            return Err(Error::ThemeProject(format!(
                "symlinks are not supported: {}",
                path.display()
            )));
        }
    }
    let mut files = collect_files(&theme_dir)?;
    let manifest_path = theme_dir.join("theme.json");
    let mut manifest = metadata::read(&manifest_path).map_err(Error::ThemeProject)?;
    if manifest.name != project.name {
        return Err(Error::ThemeProject(format!(
            "manifest name `{}` does not match theme project `{}`",
            manifest.name, project.name
        )));
    }
    metadata::validate_editable(&manifest).map_err(Error::ThemeProject)?;
    if !theme_dir.join("theme.typ").is_file() {
        return Err(Error::ThemeProject(
            "theme entrypoint `theme.typ` is missing".into(),
        ));
    }
    let root = project
        .settings
        .root
        .as_deref()
        .ok_or_else(|| Error::ThemeProject("settings root is required".into()))?;
    let cv = read_cv(&project.root.join(root), None)?;
    let options = cac_render::RenderOptions {
        project_dir: Some(project.root.join(".cac")),
        user_dir: None,
        settings: project.settings.clone(),
    };
    let (rendered, preview, width, height) =
        cac_render::render_pdf_and_preview_with_options(&cv, &options)?;
    if rendered.theme_source != cac_render::ThemeSource::Project || rendered.theme != project.name {
        return Err(Error::ThemeProject(
            "rendering did not resolve the project-local development theme".into(),
        ));
    }
    let readme = metadata::readme(&manifest, width, height);
    let mut generated = BTreeMap::new();
    generated.insert("README.md".to_owned(), readme.into_bytes());
    generated.insert("preview.jpg".to_owned(), preview);
    files.retain(|path| !matches!(path.as_str(), "theme.json" | "README.md" | "preview.jpg"));
    for path in files {
        generated.insert(path.clone(), fs::read(theme_dir.join(path))?);
    }
    manifest.preview = Some("preview.jpg".into());
    manifest.files = generated
        .iter()
        .map(|(path, bytes)| ThemeFile {
            path: path.clone(),
            sha256: format!("{:x}", Sha256::digest(bytes)),
        })
        .collect();
    metadata::validate_packaged(&manifest).map_err(Error::ThemeProject)?;
    let offering = project.root.join("offering");
    if let Ok(metadata) = fs::symlink_metadata(&offering)
        && (!metadata.is_dir() || metadata.file_type().is_symlink())
    {
        return Err(Error::ThemeProject(
            "offering must be a regular directory".into(),
        ));
    }
    fs::create_dir_all(&offering)?;
    let pdf_path = offering.join(format!("{}.pdf", project.name));
    let artifacts = [
        (theme_dir.join("README.md"), generated["README.md"].clone()),
        (
            theme_dir.join("preview.jpg"),
            generated["preview.jpg"].clone(),
        ),
        (manifest_path, metadata::bytes(&manifest)?),
        (pdf_path.clone(), rendered.bytes),
    ];
    for (path, _) in &artifacts {
        if let Ok(metadata) = fs::symlink_metadata(path)
            && !metadata.is_file()
        {
            return Err(Error::ThemeProject(format!(
                "artifact is not a regular file: {}",
                path.display()
            )));
        }
    }
    let mut staged = Vec::new();
    for (path, bytes) in &artifacts {
        let mut file = tempfile::NamedTempFile::new_in(path.parent().unwrap())?;
        file.write_all(bytes)?;
        staged.push((file, path));
    }
    for (file, path) in staged {
        file.persist(path).map_err(|error| Error::Io(error.error))?;
    }
    print_result(
        "tested",
        pdf_path.strip_prefix(&project.root).unwrap_or(&pdf_path),
    );
    let preview_path = theme_dir.join("preview.jpg");
    print_result(
        "generated",
        preview_path
            .strip_prefix(&project.root)
            .unwrap_or(&theme_dir),
    );
    let readme_path = theme_dir.join("README.md");
    print_result(
        "generated",
        readme_path
            .strip_prefix(&project.root)
            .unwrap_or(&theme_dir),
    );
    Ok(TestedTheme { project, theme_dir })
}

fn collect_files(root: &Path) -> Result<Vec<String>> {
    let mut output = Vec::new();
    visit(root, root, &mut output)?;
    output.sort();
    Ok(output)
}

fn visit(root: &Path, directory: &Path, output: &mut Vec<String>) -> Result<()> {
    let mut entries = fs::read_dir(directory)?.collect::<std::result::Result<Vec<_>, _>>()?;
    entries.sort_by_key(|entry| entry.file_name());
    for entry in entries {
        let file_type = entry.file_type()?;
        if file_type.is_symlink() {
            return Err(Error::ThemeProject(format!(
                "symlinks are not supported: {}",
                entry.path().display()
            )));
        }
        let relative = entry
            .path()
            .strip_prefix(root)
            .expect("entry is below theme root")
            .to_str()
            .ok_or_else(|| Error::ThemeProject("theme file paths must be UTF-8".into()))?
            .to_owned();
        #[cfg(windows)]
        let relative = relative.replace('\\', "/");
        if !metadata::safe_path(&relative) {
            return Err(Error::ThemeProject(format!(
                "unsafe theme path `{relative}`"
            )));
        }
        if file_type.is_dir() {
            visit(root, &entry.path(), output)?;
        } else if file_type.is_file() {
            output.push(relative);
        } else {
            return Err(Error::ThemeProject(format!(
                "unsupported theme entry: {}",
                entry.path().display()
            )));
        }
    }
    Ok(())
}

pub(super) fn verify_files(theme_dir: &Path, manifest: &ThemeMetadata) -> Result<()> {
    metadata::validate_packaged(manifest).map_err(Error::ThemeProject)?;
    let actual = collect_files(theme_dir)?
        .into_iter()
        .filter(|path| path != "theme.json")
        .collect::<BTreeSet<_>>();
    let declared = manifest
        .files
        .iter()
        .map(|file| file.path.clone())
        .collect();
    if actual != declared {
        return Err(Error::ThemeProject(
            "manifest file list does not match theme contents".into(),
        ));
    }
    for file in &manifest.files {
        let bytes = fs::read(theme_dir.join(&file.path))?;
        if format!("{:x}", Sha256::digest(bytes)) != file.sha256 {
            return Err(Error::ThemeChecksum {
                theme: manifest.name.clone(),
                path: file.path.clone(),
            });
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::super::metadata::ThemeMetadata;
    use super::*;

    #[test]
    fn verification_rejects_modified_missing_and_undeclared_files() {
        let directory = tempfile::tempdir().unwrap();
        let mut manifest = ThemeMetadata {
            name: "example".into(),
            description: "Example theme".into(),
            author: "Ada".into(),
            author_url: None,
            license: "MIT".into(),
            theme_api: cac_render::THEME_API_VERSION,
            preview: Some("preview.jpg".into()),
            files: Vec::new(),
        };
        for path in ["theme.typ", "README.md", "preview.jpg"] {
            fs::write(directory.path().join(path), b"original").unwrap();
            manifest.files.push(ThemeFile {
                path: path.into(),
                sha256: format!("{:x}", Sha256::digest(b"original")),
            });
        }
        verify_files(directory.path(), &manifest).unwrap();
        fs::write(directory.path().join("theme.typ"), b"modified").unwrap();
        assert!(matches!(
            verify_files(directory.path(), &manifest),
            Err(Error::ThemeChecksum { .. })
        ));
        fs::write(directory.path().join("theme.typ"), b"original").unwrap();
        fs::write(directory.path().join("extra.txt"), b"undeclared").unwrap();
        assert!(verify_files(directory.path(), &manifest).is_err());
        fs::remove_file(directory.path().join("extra.txt")).unwrap();
        fs::remove_file(directory.path().join("preview.jpg")).unwrap();
        assert!(verify_files(directory.path(), &manifest).is_err());
    }
}
