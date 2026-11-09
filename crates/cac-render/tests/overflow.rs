//! D1 slice 2: pages whose content extends beyond the page edge are named.

use cac_core::CvDocument;
use cac_render::{RenderOptions, Settings, render_pdf_with_options};
use tempfile::tempdir;

#[test]
fn an_overflowing_page_is_named_and_a_clean_document_is_not() {
    let directory = tempdir().unwrap();
    let theme = directory.path().join(".cac/themes").join("overflow-test");
    std::fs::create_dir_all(&theme).unwrap();
    std::fs::write(
        theme.join("theme.typ"),
        r#"
#import "/.cac/base.typ" as base
#let theme = (layout: (
  document: ctx => {
    let body = ctx.styles.body
    set document(title: "Overflow", author: ctx.cv.profile.name)
    set page(paper: "a4", margin: 18mm)
    set text(size: body.font_size)
    (ctx.components.header)(ctx)
    place(bottom + center, dy: 5cm)[#text(fill: red)[CACTailMarkerZZZ]]
  },
))
"#,
    )
    .unwrap();
    let cv: CvDocument =
        serde_json::from_str(r#"{"profile":{"name":"Ada"},"sections":[]}"#).unwrap();
    let settings: Settings = serde_json::from_str(r#"{"theme":"overflow-test"}"#).unwrap();
    let rendered = render_pdf_with_options(
        &cv,
        &RenderOptions {
            project_dir: Some(directory.path().join(".cac")),
            user_dir: None,
            settings,
        },
    )
    .unwrap();
    assert_eq!(
        rendered.overflow_pages,
        vec![1],
        "page 1 draws past the edge"
    );
}
