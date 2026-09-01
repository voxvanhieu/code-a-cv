use cac_io::{InputFormat, STARTER_MARKDOWN, parse};
use std::fs;

use cac_core::DatePoint;
use cac_render::{
    RenderOptions, Settings, ThemeSource, format_date, render_html, render_pdf,
    render_pdf_with_options,
};
use tempfile::tempdir;

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
fn display_dates_match_classic_resume_style() {
    assert_eq!(format_date(&DatePoint::Year(2021)), "2021");
    assert_eq!(format_date(&DatePoint::YearMonth(2021, 8)), "Aug 2021");
    let full = serde_json::from_str("\"2021-08-17\"").unwrap();
    assert_eq!(format_date(&full), "Aug 17, 2021");
    assert_eq!(format_date(&DatePoint::Present), "Present");
}

#[test]
fn standalone_text_entries_render_as_one_list() {
    let source = "# Ada\n\nada@example.com\n\n## Professional Interests\n\n- Dependable systems\n- Long-distance cycling\n";
    let cv = parse(source, InputFormat::Markdown).unwrap();
    let html = render_html(&cv);

    assert!(html.contains(
        "<ul class=\"text-entries\"><li>Dependable systems</li><li>Long-distance cycling</li></ul>"
    ));
    assert!(!html.contains("<strong>Dependable systems</strong>"));

    let directory = tempdir().unwrap();
    write_theme(
        directory.path(),
        "inspect-text",
        r#"
#import "/.cac/base.typ" as base
#let checked-header(ctx) = {
  let entries = ctx.cv.sections.first().entries
  if entries.len() != 1 { panic("text entries were not grouped") }
  if entries.first().kind != "text" { panic("text entry kind was not preserved") }
  if entries.first().highlights.len() != 1 { panic("grouped text entry lost an item") }
}
#let theme = base.extend(components: (header: checked-header))
"#,
    );
    let rendered = render_pdf_with_options(
        &cv,
        &RenderOptions {
            project_dir: Some(directory.path().into()),
            settings: Settings {
                theme: Some("inspect-text".into()),
                ..Settings::default()
            },
            ..RenderOptions::default()
        },
    )
    .unwrap();
    assert!(rendered.bytes.starts_with(b"%PDF-"));
}

#[test]
fn settings_reject_unknown_properties_and_invalid_values() {
    let directory = tempdir().unwrap();
    let path = directory.path().join("settings.json");

    fs::write(&path, r#"{"colour":"red"}"#).unwrap();
    assert!(
        Settings::from_path(&path)
            .unwrap_err()
            .to_string()
            .contains("unknown field")
    );

    fs::write(&path, r#"{"theme":"../escape"}"#).unwrap();
    assert!(
        Settings::from_path(&path)
            .unwrap_err()
            .to_string()
            .contains("invalid theme name")
    );

    for root in ["", ".", "..", "../cv.md", "nested/cv.md", r"nested\cv.md"] {
        fs::write(
            &path,
            serde_json::to_vec(&serde_json::json!({ "root": root })).unwrap(),
        )
        .unwrap();
        assert!(
            Settings::from_path(&path)
                .unwrap_err()
                .to_string()
                .contains("invalid `root`")
        );
    }

    fs::write(&path, r#"{"font_size":"large"}"#).unwrap();
    assert!(
        Settings::from_path(&path)
            .unwrap_err()
            .to_string()
            .contains("positive Typst length")
    );

    let cv = parse(STARTER_MARKDOWN, InputFormat::Markdown).unwrap();
    let error = render_pdf_with_options(
        &cv,
        &RenderOptions {
            settings: Settings {
                font_size: Some("10pt); panic(\"injected\")".into()),
                ..Settings::default()
            },
            ..RenderOptions::default()
        },
    )
    .unwrap_err();
    assert!(error.to_string().contains("positive Typst length"));
}

#[test]
fn project_theme_wins_and_settings_override_theme_defaults() {
    let directory = tempdir().unwrap();
    let project = directory.path().join("project");
    let user = directory.path().join("user");
    write_theme(
        &project,
        "oxford",
        r#"
#import "/.cac/base.typ" as base
#let checked-header(ctx) = {
  if ctx.styles.entry.space_after != 2em { panic("settings did not override theme") }
  read("assets/label.txt")
}
#let theme = base.extend(
  styles: (entry: (space_after: 1em)),
  components: (header: checked-header),
)
"#,
    );
    fs::create_dir_all(project.join("themes/oxford/assets")).unwrap();
    fs::write(
        project.join("themes/oxford/assets/label.txt"),
        "Project theme",
    )
    .unwrap();
    write_theme(&user, "oxford", "#let broken =");
    let cv = parse(STARTER_MARKDOWN, InputFormat::Markdown).unwrap();
    let rendered = render_pdf_with_options(
        &cv,
        &RenderOptions {
            project_dir: Some(project),
            user_dir: Some(user),
            settings: Settings {
                theme: Some("oxford".into()),
                entry_spacing: Some("2em".into()),
                ..Settings::default()
            },
        },
    )
    .unwrap();

    assert_eq!(rendered.theme, "oxford");
    assert_eq!(rendered.theme_source, ThemeSource::Project);
    assert!(rendered.bytes.starts_with(b"%PDF-"));
}

#[test]
fn theme_api_version_is_checked() {
    let directory = tempdir().unwrap();
    write_theme(
        directory.path(),
        "future",
        "#let theme = (api_version: 99, tokens: (:), styles: (:), page: (:), components: (:))",
    );
    let cv = parse(STARTER_MARKDOWN, InputFormat::Markdown).unwrap();
    let error = render_pdf_with_options(
        &cv,
        &RenderOptions {
            project_dir: Some(directory.path().into()),
            settings: Settings {
                theme: Some("future".into()),
                ..Settings::default()
            },
            ..RenderOptions::default()
        },
    )
    .unwrap_err();

    let message = error.to_string();
    assert!(message.contains("unsupported theme API version 99"));
    assert!(message.contains("cac supports version 1"));
}

#[test]
fn user_theme_is_used_when_project_theme_is_absent() {
    let directory = tempdir().unwrap();
    let project = directory.path().join("project");
    let user = directory.path().join("user");
    write_theme(
        &user,
        "oxford",
        "#import \"/.cac/base.typ\" as base\n#let theme = base.extend()",
    );
    let cv = parse(STARTER_MARKDOWN, InputFormat::Markdown).unwrap();
    let rendered = render_pdf_with_options(
        &cv,
        &RenderOptions {
            project_dir: Some(project),
            user_dir: Some(user),
            settings: Settings {
                theme: Some("oxford".into()),
                ..Settings::default()
            },
        },
    )
    .unwrap();

    assert_eq!(rendered.theme_source, ThemeSource::User);
}

#[test]
fn missing_theme_and_missing_api_version_have_actionable_errors() {
    let directory = tempdir().unwrap();
    let cv = parse(STARTER_MARKDOWN, InputFormat::Markdown).unwrap();
    let missing = render_pdf_with_options(
        &cv,
        &RenderOptions {
            project_dir: Some(directory.path().into()),
            settings: Settings {
                theme: Some("missing".into()),
                ..Settings::default()
            },
            ..RenderOptions::default()
        },
    )
    .unwrap_err();
    assert!(missing.to_string().contains("was not found"));

    write_theme(
        directory.path(),
        "unversioned",
        "#let theme = (tokens: (:), styles: (:), page: (:), components: (:))",
    );
    let unversioned = render_pdf_with_options(
        &cv,
        &RenderOptions {
            project_dir: Some(directory.path().into()),
            settings: Settings {
                theme: Some("unversioned".into()),
                ..Settings::default()
            },
            ..RenderOptions::default()
        },
    )
    .unwrap_err();
    assert!(unversioned.to_string().contains("base.extend"));
}

#[cfg(unix)]
#[test]
fn theme_assets_cannot_escape_through_symlinks() {
    use std::os::unix::fs::symlink;

    let directory = tempdir().unwrap();
    write_theme(
        directory.path(),
        "escape",
        r#"
#import "/.cac/base.typ" as base
#let unsafe-header(ctx) = read("assets/outside.txt")
#let theme = base.extend(components: (header: unsafe-header))
"#,
    );
    fs::write(directory.path().join("outside.txt"), "outside").unwrap();
    fs::create_dir_all(directory.path().join("themes/escape/assets")).unwrap();
    symlink(
        directory.path().join("outside.txt"),
        directory.path().join("themes/escape/assets/outside.txt"),
    )
    .unwrap();
    let cv = parse(STARTER_MARKDOWN, InputFormat::Markdown).unwrap();
    let error = render_pdf_with_options(
        &cv,
        &RenderOptions {
            project_dir: Some(directory.path().into()),
            settings: Settings {
                theme: Some("escape".into()),
                ..Settings::default()
            },
            ..RenderOptions::default()
        },
    )
    .unwrap_err();

    assert!(error.to_string().contains("access denied"));
}

fn write_theme(root: &std::path::Path, name: &str, source: &str) {
    let directory = root.join("themes").join(name);
    fs::create_dir_all(&directory).unwrap();
    fs::write(directory.join("theme.typ"), source).unwrap();
}

#[test]
fn pdf_is_valid_and_reproducible() {
    let cv = parse(STARTER_MARKDOWN, InputFormat::Markdown).unwrap();
    let first = render_pdf(&cv).unwrap();
    let second = render_pdf(&cv).unwrap();

    assert!(first.bytes.starts_with(b"%PDF-"));
    assert_eq!(first.pages, 1);
    assert_eq!(first.theme, "classic");
    assert_eq!(first.theme_source, ThemeSource::Embedded);
    assert_eq!(first.bytes, second.bytes);
}

#[test]
fn classic_typography_accepts_equivalent_settings_overrides() {
    let cv = parse(STARTER_MARKDOWN, InputFormat::Markdown).unwrap();
    let inherited = render_pdf(&cv).unwrap();
    let explicit = render_pdf_with_options(
        &cv,
        &RenderOptions {
            settings: Settings {
                root: Some("cv.md".into()),
                paper: Some("us-letter".into()),
                page_margin: Some("12.7mm".into()),
                font: Some("New Computer Modern".into()),
                font_size: Some("10pt".into()),
                line_spacing: Some("0.65em".into()),
                section_spacing: Some("1.2em".into()),
                entry_spacing: Some("1.2em".into()),
                ..Settings::default()
            },
            ..RenderOptions::default()
        },
    )
    .unwrap();

    assert_eq!(inherited.bytes, explicit.bytes);
}

#[test]
fn classic_left_can_be_rendered_as_an_installed_theme() {
    let directory = tempdir().unwrap();
    write_theme(
        directory.path(),
        "classic-left",
        include_str!("../../../themes/classic-left/theme.typ"),
    );
    let cv = parse(STARTER_MARKDOWN, InputFormat::Markdown).unwrap();
    let rendered = render_pdf_with_options(
        &cv,
        &RenderOptions {
            project_dir: Some(directory.path().into()),
            settings: Settings {
                theme: Some("classic-left".into()),
                ..Settings::default()
            },
            ..RenderOptions::default()
        },
    )
    .unwrap();

    assert_eq!(rendered.theme, "classic-left");
    assert_eq!(rendered.theme_source, ThemeSource::Project);
    assert_eq!(rendered.pages, 1);
    assert!(rendered.bytes.starts_with(b"%PDF-"));
}

#[test]
fn heading_levels_resolve_independent_typography() {
    let directory = tempdir().unwrap();
    write_theme(
        directory.path(),
        "headings",
        r#"
#import "/.cac/base.typ" as base
#let checked-header(ctx) = {
  if ctx.styles.heading_1.line_spacing != 0.8em { panic("heading 1 spacing was not resolved") }
  if ctx.styles.heading_5.line_spacing != 1.4em { panic("heading 5 spacing was not resolved") }
  if ctx.styles.heading_1.paragraph_spacing != 0.7em { panic("paragraph spacing was not inherited") }
  if ctx.styles.heading_1.space_before != 0.2em { panic("heading space before was not overridden") }
  if ctx.styles.heading_1.space_after != 0.3em { panic("heading space after was not overridden") }
  if ctx.styles.heading_5.weight != "bold" { panic("shared heading style was not inherited") }
  if ctx.styles.section.space_after_heading != 0.4em { panic("section heading gap was not overridden") }
  (ctx.components.heading)(ctx, 1, [Heading one])
  (ctx.components.heading)(ctx, 5, [Heading five])
}

#let theme = base.extend(
  styles: (
    body: (paragraph_spacing: 0.7em),
    heading_1: (
      line_spacing: 0.8em,
      space_before: 0.2em,
      space_after: 0.3em,
    ),
    heading_5: (line_spacing: 1.4em),
    section: (space_after_heading: 0.4em),
  ),
  components: (header: checked-header),
)
"#,
    );
    let cv = parse(STARTER_MARKDOWN, InputFormat::Markdown).unwrap();
    let rendered = render_pdf_with_options(
        &cv,
        &RenderOptions {
            project_dir: Some(directory.path().into()),
            settings: Settings {
                theme: Some("headings".into()),
                ..Settings::default()
            },
            ..RenderOptions::default()
        },
    )
    .unwrap();

    assert!(rendered.bytes.starts_with(b"%PDF-"));
}

#[test]
fn base_spacing_properties_resolve_independently() {
    let directory = tempdir().unwrap();
    write_theme(
        directory.path(),
        "bare",
        r#"
#import "/.cac/base.typ" as base
#let checked-header(ctx) = {
  if ctx.styles.body.line_spacing != 0.9em { panic("body leading was not overridden") }
  if ctx.styles.body.paragraph_spacing != 1.2em { panic("paragraph spacing changed") }
  for level in range(1, 6) {
    let style = ctx.styles.at("heading_" + str(level))
    if style.font_size != ctx.styles.body.font_size { panic("heading font size differs from base") }
    if style.line_spacing != 0.9em { panic("heading leading differs from body") }
    if style.space_before != 0pt { panic("heading space before changed") }
    let expected_after = if level == 1 { 1.2em } else { 0.65em }
    if style.space_after != expected_after { panic("heading space after changed") }
  }
  if ctx.styles.list.item_spacing != auto { panic("list spacing is not automatic") }
  if ctx.styles.section.space_before != 1.2em { panic("section space before changed") }
  if ctx.styles.section.space_after_heading != 0.65em { panic("section heading gap changed") }
  if ctx.styles.entry.space_after != 1.2em { panic("entry space after changed") }
  [Bare theme]
}
#let theme = base.extend(components: (header: checked-header))
"#,
    );
    let cv = parse(STARTER_MARKDOWN, InputFormat::Markdown).unwrap();
    let rendered = render_pdf_with_options(
        &cv,
        &RenderOptions {
            project_dir: Some(directory.path().into()),
            settings: Settings {
                theme: Some("bare".into()),
                line_spacing: Some("0.9em".into()),
                ..Settings::default()
            },
            ..RenderOptions::default()
        },
    )
    .unwrap();

    assert!(rendered.bytes.starts_with(b"%PDF-"));
}
