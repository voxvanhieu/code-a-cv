use std::fs;
use std::io::{Read, Write};

use crate::error::{Error, Result};

use super::{metadata, test};

pub(super) fn run() -> Result<()> {
    let tested = test::test_current()?;
    let manifest =
        metadata::read(&tested.theme_dir.join("theme.json")).map_err(Error::ThemeProject)?;
    test::verify_files(&tested.theme_dir, &manifest)?;
    let destination = tested.project.root.join(format!("{}.zip", manifest.name));
    let temporary = tested
        .project
        .root
        .join(format!(".{}.zip.tmp", manifest.name));
    let file = fs::File::create(&temporary)?;
    let mut zip = zip::ZipWriter::new(file);
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
    let result = (|| -> Result<()> {
        for path in paths {
            zip.start_file(format!("{}/{path}", manifest.name), options)
                .map_err(|error| Error::ThemeProject(error.to_string()))?;
            let mut source = fs::File::open(tested.theme_dir.join(path))?;
            let mut bytes = Vec::new();
            source.read_to_end(&mut bytes)?;
            zip.write_all(&bytes)?;
        }
        zip.finish()
            .map_err(|error| Error::ThemeProject(error.to_string()))?;
        Ok(())
    })();
    if let Err(error) = result {
        let _ = fs::remove_file(&temporary);
        return Err(error);
    }
    fs::rename(&temporary, &destination)?;
    println!(
        "PACKED {}.zip\n\nTo publish:\n1. Fork https://github.com/voxvanhieu/code-a-cv\n2. Extract {}.zip into the fork's themes/ directory\n3. Add the theme name and description to themes/index.json\n4. Commit the files and open a pull request\n5. Use themes/{}/README.md as the pull request description",
        manifest.name, manifest.name, manifest.name
    );
    Ok(())
}
