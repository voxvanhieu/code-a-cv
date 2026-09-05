use std::{fs, path::Path};

use cac_render::{RenderOptions, Settings, test_shared_theme};
use tempfile::tempdir;

#[test]
fn embedded_classic_preserves_shared_corpus_content_and_links() {
    let artifacts = test_shared_theme(&RenderOptions::default()).unwrap();
    assert!(
        artifacts
            .iter()
            .find(|a| a.name == "long")
            .unwrap()
            .pdf
            .pages
            > 1
    );
}

#[test]
fn registry_and_table_themes_preserve_shared_corpus() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let index: serde_json::Value =
        serde_json::from_slice(&fs::read(root.join("themes/index.json")).unwrap()).unwrap();
    for theme in index["themes"].as_array().unwrap() {
        let name = theme["name"].as_str().unwrap();
        test_shared_theme(&RenderOptions {
            project_dir: Some(root.clone()),
            settings: Settings {
                theme: Some(name.into()),
                ..Settings::default()
            },
            ..RenderOptions::default()
        })
        .unwrap_or_else(|error| panic!("{name}: {error}"));
    }
    test_shared_theme(&RenderOptions {
        project_dir: Some(root.join("docs/examples/customized-layout/.cac")),
        settings: Settings {
            theme: Some("table-layout".into()),
            ..Settings::default()
        },
        ..RenderOptions::default()
    })
    .unwrap();
}

#[test]
fn shared_gate_rejects_missing_fields_and_changed_order() {
    for component in [
        "header: ctx => [omitted profile]",
        "entry_details: (ctx, entry) => (ctx.components.heading)(ctx, 3, (ctx.components.rich)(ctx, entry.primary))",
        "section: (ctx, section) => base.section_flow(ctx, (..section, entries: section.entries.rev()))",
    ] {
        let directory = tempdir().unwrap();
        let target = directory.path().join("themes/incomplete");
        fs::create_dir_all(&target).unwrap();
        fs::write(target.join("theme.typ"), format!("#import \"/.cac/base.typ\" as base\n#let theme = base.extend(components: ({component},))")).unwrap();
        let error = test_shared_theme(&RenderOptions {
            project_dir: Some(directory.path().into()),
            settings: Settings {
                theme: Some("incomplete".into()),
                ..Settings::default()
            },
            ..RenderOptions::default()
        })
        .err()
        .expect("incomplete themes fail");
        assert!(
            error.message.contains("marker") || error.message.contains("order"),
            "{error}"
        );
    }
}

#[test]
fn future_render_view_kinds_use_complete_generic_fallback() {
    let directory = tempdir().unwrap();
    let target = directory.path().join("themes/future");
    fs::create_dir_all(&target).unwrap();
    fs::write(target.join("theme.typ"), r#"
#import "/.cac/base.typ" as base
#let section(ctx, section) = base.section_table(ctx, (
  ..section,
  entries: section.entries.map(entry => if entry.kind == "text" { entry } else { (..entry, kind: "future") }),
))
#let theme = base.extend(components: (section: section))
"#).unwrap();
    test_shared_theme(&RenderOptions {
        project_dir: Some(directory.path().into()),
        settings: Settings {
            theme: Some("future".into()),
            ..Settings::default()
        },
        ..RenderOptions::default()
    })
    .unwrap();
}

#[test]
fn table_selection_checks_every_row_and_preserves_child_style_overrides() {
    let directory = tempdir().unwrap();
    let target = directory.path().join("themes/checked");
    fs::create_dir_all(&target).unwrap();
    fs::write(target.join("theme.typ"), r#"
#import "/.cac/base.typ" as base
#let skill = (kind: "skill-group", primary: (), secondary: none, period: none, metadata: (), highlights: ())
#let custom = (..skill, kind: "custom", period: "2020")
#assert(base.table_kind((entries: ())) == none)
#assert(base.table_kind((entries: (skill, custom))) == none)
#assert(base.table_kind((entries: (custom, skill))) == none)
#assert(base.table_kind((entries: (skill,))) == "skill-group")
#assert(base.table_kind((entries: ((..skill, secondary: ()),))) == none)
#assert(base.table_kind((entries: ((..skill, kind: "future"),))) == none)
#let section(ctx, section) = {
  assert(ctx.styles.entry.space_after == 17pt)
  assert(ctx.styles.entry.allow_page_break)
  assert(ctx.styles.section.space_after_heading == 9pt)
  assert(ctx.styles.highlight.bullet == "+")
  assert(ctx.styles.header.alignment == right)
  assert(not ctx.styles.link.underline)
  if base.table_kind(section) == "skill-group" { base.section_labels(ctx, section) }
  else { base.section_table(ctx, section) }
}
#let styled(ctx, style, body) = {
  assert(ctx.tokens.fonts.body == "Libertinus Serif")
  base.styled-text(ctx, style, body)
}
#let theme = base.extend(components: (section: section, styled_text: styled))
"#).unwrap();
    let cv = serde_json::from_str(include_str!("../src/fixtures/shared/complete.json")).unwrap();
    let settings = serde_json::from_value(serde_json::json!({
        "theme": "checked",
        "typography": {"font": "Libertinus Serif"},
        "spacing": {"entry": "17pt", "section_heading": "9pt"},
        "pagination": {"allow_entry_page_break": true},
        "style": {"highlight_bullet": "+", "header_alignment": "right", "link_underline": false}
    }))
    .unwrap();
    cac_render::render_pdf_with_options(
        &cv,
        &RenderOptions {
            project_dir: Some(directory.path().into()),
            settings,
            ..RenderOptions::default()
        },
    )
    .unwrap();
}
