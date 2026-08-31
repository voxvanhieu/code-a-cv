use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

use cac_core::CvDocument;
use cac_io::{InputFormat, parse};

use crate::cli::CvFormat;
use crate::error::{Error, Result};

pub fn read_cv(path: &Path, input_format: Option<CvFormat>) -> Result<CvDocument> {
    let source = read_source(path)?;
    let format = if let Some(format) = input_format {
        format.into()
    } else if is_stdio(path) {
        InputFormat::Markdown
    } else {
        let extension = path
            .extension()
            .and_then(|value| value.to_str())
            .ok_or_else(|| Error::Unsupported(path.into()))?;
        InputFormat::from_extension(extension).ok_or_else(|| Error::Unsupported(path.into()))?
    };
    Ok(parse(&source, format)?)
}

pub fn read_source(path: &Path) -> Result<String> {
    if is_stdio(path) {
        let mut source = String::new();
        io::stdin().read_to_string(&mut source)?;
        Ok(source)
    } else {
        Ok(fs::read_to_string(path)?)
    }
}

pub fn write_stdout(content: &str) -> Result<()> {
    let stdout = io::stdout();
    let mut output = stdout.lock();
    output.write_all(content.as_bytes())?;
    if !content.ends_with('\n') {
        output.write_all(b"\n")?;
    }
    Ok(())
}

pub fn is_stdio(path: &Path) -> bool {
    path == Path::new("-")
}

pub fn input_stem(path: &Path) -> &str {
    if is_stdio(path) {
        "cv"
    } else {
        path.file_stem()
            .and_then(|value| value.to_str())
            .unwrap_or("cv")
    }
}

pub fn print_result(action: &str, path: &Path) {
    println!("{} {}", action.to_ascii_uppercase(), path.display());
}
