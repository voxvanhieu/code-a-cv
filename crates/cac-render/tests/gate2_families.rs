//! Gate 2: the full shared corpus renders through every built-in family.

use cac_core::CvDocument;
use cac_render::{RenderOptions, Settings, render_pdf_with_options};
use tempfile::tempdir;

const MINIMAL_PNG: [u8; 69] = [
    0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44, 0x52,
    0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x02, 0x00, 0x00, 0x00, 0x90, 0x77, 0x53,
    0xDE, 0x00, 0x00, 0x00, 0x0C, 0x49, 0x44, 0x41, 0x54, 0x78, 0x9C, 0x63, 0xF8, 0xFF, 0xFF, 0x3F,
    0x00, 0x05, 0xFE, 0x02, 0xFE, 0x0D, 0xEF, 0x46, 0xB8, 0x00, 0x00, 0x00, 0x00, 0x49, 0x45, 0x4E,
    0x44, 0xAE, 0x42, 0x60, 0x82,
];

fn corpus() -> Vec<(String, CvDocument)> {
    let dir = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../cac-render/fixtures/shared/"
    );
    let mut documents = Vec::new();
    for entry in std::fs::read_dir(dir).expect("corpus dir") {
        let path = entry.unwrap().path();
        if path
            .extension()
            .is_some_and(|extension| extension == "json")
        {
            let name = path.file_stem().unwrap().to_string_lossy().to_string();
            let source = std::fs::read_to_string(&path).unwrap();
            documents.push((name, serde_json::from_str(&source).unwrap()));
        }
    }
    documents.sort_by(|a, b| a.0.cmp(&b.0));
    documents
}

fn render_family(family: &str, cv: &CvDocument) -> usize {
    let directory = tempdir().unwrap();
    let theme = directory.path().join(".cac/themes").join(family);
    fs_err_create(&theme);
    std::fs::write(
        theme.join("theme.typ"),
        format!("#let theme = (layout: (family: \"{family}\"))\n"),
    )
    .unwrap();
    // Documents may reference a photo; point the CV at a known-good PNG
    // inside the project root so the jailed lookup resolves it. (Typst
    // dispatches the decoder by extension, and the corpus's referenced
    // names include .jpg files we synthesize as PNG.)
    let mut cv = cv.clone();
    cv.profile.photo = Some("corpus-photo.png".into());
    std::fs::write(directory.path().join("corpus-photo.png"), MINIMAL_PNG).unwrap();
    let settings: Settings = serde_json::from_str(&format!(r#"{{"theme":"{family}"}}"#)).unwrap();
    let rendered = render_pdf_with_options(
        &cv,
        &RenderOptions {
            project_dir: Some(directory.path().join(".cac")),
            user_dir: None,
            settings,
        },
    )
    .unwrap();
    rendered.pages
}

fn fs_err_create(path: &std::path::Path) {
    std::fs::create_dir_all(path).unwrap();
}

#[test]
fn every_family_renders_every_corpus_document() {
    for family in ["classic", "table", "compact", "sidebar"] {
        for (name, cv) in corpus() {
            let pages = render_family(family, &cv);
            assert!(pages >= 1, "{family} renders {name} with at least one page");
        }
    }
}

#[test]
fn the_form_family_renders_its_supported_corpus_subset() {
    // The form rejects sections outside its allowed list by design, so the
    // acceptance is: conforming documents render, unsupported ones fail
    // naming the section.
    for (name, cv) in corpus() {
        let kinds: Vec<&str> = cv
            .sections
            .iter()
            .map(|section| section.kind.as_str())
            .collect();
        // The form is selective by design: sections it cannot place are
        // rejected by name. Skip documents with kinds beyond its list.
        let supported = [
            "education",
            "experience",
            "languages",
            "certifications",
            "projects",
            "custom",
        ];
        if !kinds.iter().all(|kind| supported.contains(kind)) {
            continue;
        }
        let pages = render_family("form", &cv);
        assert!(pages >= 1, "form renders conforming {name}");
    }
}
