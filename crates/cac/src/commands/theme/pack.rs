use std::fs;
use std::io::Write;

use sha2::{Digest, Sha256};

use crate::error::{Error, Result};

use super::{metadata, test};

pub(super) fn run() -> Result<()> {
    let tested = test::test_current()?;
    let manifest =
        metadata::read(&tested.theme_dir.join("theme.json")).map_err(Error::ThemeProject)?;
    test::verify_files(&tested.theme_dir, &manifest)?;
    let destination = tested.project.root.join(format!("{}.zip", manifest.name));
    let mut temporary = tempfile::NamedTempFile::new_in(&tested.project.root)?;
    let mut zip = zip::ZipWriter::new(temporary.as_file_mut());
    let options = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated)
        .last_modified_time(zip::DateTime::default())
        .unix_permissions(0o644);
    let mut paths = manifest
        .files
        .iter()
        .map(|file| file.path.clone())
        .collect::<Vec<_>>();
    paths.push("theme.json".into());
    paths.sort();
    for path in paths {
        let bytes = fs::read(tested.theme_dir.join(&path))?;
        if let Some(file) = manifest.files.iter().find(|file| file.path == path)
            && format!("{:x}", Sha256::digest(&bytes)) != file.sha256
        {
            return Err(Error::ThemeChecksum {
                theme: manifest.name.clone(),
                path,
            });
        }
        zip.start_file(format!("{}/{path}", manifest.name), options)
            .map_err(|error| Error::ThemeProject(error.to_string()))?;
        zip.write_all(&bytes)?;
    }
    zip.finish()
        .map_err(|error| Error::ThemeProject(error.to_string()))?;
    temporary
        .persist(&destination)
        .map_err(|error| Error::Io(error.error))?;
    println!(
        "PACKED {}.zip\n\nTo publish:\n1. Fork https://github.com/voxvanhieu/code-a-cv\n2. Extract {}.zip into the fork's themes/ directory\n3. Add the theme name and description to themes/index.json\n4. Commit the files and open a pull request\n5. Use themes/{}/README.md as the pull request description",
        manifest.name, manifest.name, manifest.name
    );
    Ok(())
}
