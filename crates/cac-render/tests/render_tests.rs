use cac_io::{InputFormat, STARTER_MARKDOWN, parse};
use cac_render::{render_html, render_pdf};

#[test]
fn html_escapes_text_and_preserves_rich_markup() {
    let source = "# A < B\n\na@example.com\n\n## Experience\n\n### C# Engineer, R&D\n2020–Present\n\n- Improved **100%** of `<jobs>`\n";
    let cv = parse(source, InputFormat::Markdown).unwrap();
    let html = render_html(&cv);

    assert!(html.contains("A &lt; B"));
    assert!(html.contains("R&amp;D"));
    assert!(html.contains("<strong>100%</strong>"));
    assert!(html.contains("&lt;jobs&gt;"));
    assert!(!html.contains("<jobs>"));
}

#[test]
fn pdf_is_valid_and_reproducible() {
    let cv = parse(STARTER_MARKDOWN, InputFormat::Markdown).unwrap();
    let first = render_pdf(&cv).unwrap();
    let second = render_pdf(&cv).unwrap();

    assert!(first.bytes.starts_with(b"%PDF-"));
    assert_eq!(first.pages, 1);
    assert_eq!(first.bytes, second.bytes);
}
