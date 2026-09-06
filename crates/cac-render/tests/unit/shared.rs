use super::*;
use std::path::Path;
use typst::layout::{Abs, Point, Transform};

fn positions(frame: &Frame, transform: Transform, output: &mut Vec<(String, Point)>) {
    for (position, item) in frame.items() {
        let transform = transform.pre_concat(Transform::translate(position.x, position.y));
        match item {
            FrameItem::Group(group) => {
                positions(&group.frame, transform.pre_concat(group.transform), output)
            }
            FrameItem::Text(run) => {
                output.push((run.text.to_string(), Point::zero().transform(transform)))
            }
            _ => {}
        }
    }
}

#[test]
fn classic_dates_do_not_narrow_following_paragraphs() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    for theme in ["classic", "classic-left", "classic-blue"] {
        let options = RenderOptions {
            project_dir: Some(root.clone()),
            settings: crate::Settings {
                theme: Some(theme.into()),
                ..crate::Settings::default()
            },
            ..RenderOptions::default()
        };
        let paragraph = (0..80)
            .map(|index| format!("Width{index}"))
            .collect::<Vec<_>>()
            .join(" ");
        let mut source: Value =
            serde_json::from_str(include_str!("../../fixtures/shared/complete.json")).unwrap();
        source["sections"].as_array_mut().unwrap().truncate(1);
        source["sections"][0]["entries"][0]["highlights"] = serde_json::json!([paragraph]);
        let mut layouts = Vec::new();
        for dated in [true, false] {
            if !dated {
                source["sections"][0]["entries"][0]
                    .as_object_mut()
                    .unwrap()
                    .remove("period");
            }
            let cv = serde_json::from_value(source.clone()).unwrap();
            let (_, document) = render_pdf_document(&cv, &options).unwrap();
            let mut lines = Vec::new();
            positions(
                &document.pages()[0].frame,
                Transform::identity(),
                &mut lines,
            );
            let y = |marker: &str| {
                lines
                    .iter()
                    .find(|(text, _)| text.contains(marker))
                    .unwrap()
                    .1
                    .y
            };
            assert!(
                y("CACOrganizationZZZ") - y("CACRoleZZZ") >= Abs::pt(10.0),
                "{theme}: organization overlaps title"
            );
            let mut paragraph_lines = lines
                .into_iter()
                .filter(|(text, _)| text.contains("Width"))
                .collect::<Vec<_>>();
            assert!(paragraph_lines.len() > 1);
            let top = paragraph_lines[0].1.y;
            for (_, point) in &mut paragraph_lines {
                point.y -= top;
            }
            layouts.push(paragraph_lines);
        }
        assert_eq!(
            layouts[0], layouts[1],
            "{theme}: dates change paragraph wrapping"
        );
    }
}

#[test]
fn table_metadata_and_following_content_have_readable_line_gaps() {
    let cv = serde_json::from_str(include_str!("../../fixtures/shared/complete.json")).unwrap();
    let root =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/examples/customized-layout/.cac");
    let (_, document) = render_pdf_document(
        &cv,
        &RenderOptions {
            project_dir: Some(root),
            settings: crate::Settings {
                theme: Some("table-layout".into()),
                ..crate::Settings::default()
            },
            ..RenderOptions::default()
        },
    )
    .unwrap();
    let mut lines = Vec::new();
    for page in document.pages() {
        positions(&page.frame, Transform::identity(), &mut lines);
    }
    let y = |marker: &str| {
        lines
            .iter()
            .find(|(text, _)| text.contains(marker))
            .unwrap()
            .1
            .y
    };
    for (upper, lower) in [
        ("CACOrganizationZZZ", "CACLocationZZZ"),
        ("CACLocationZZZ", "CACExperienceHighlightZZZ"),
        ("CACInstitutionZZZ", "CACEducationHighlightZZZ"),
        ("CACProjectURLZZZ", "CACProjectHighlightZZZ"),
        ("CACPublisherZZZ", "CACPublicationURLZZZ"),
        ("CACPublicationURLZZZ", "CACPublicationHighlightZZZ"),
    ] {
        assert!(
            y(lower) - y(upper) >= Abs::pt(10.0),
            "{upper} overlaps {lower}"
        );
    }
}

#[test]
fn missing_dates_collapse_columns_and_partial_dates_keep_alignment() {
    for (source, dated) in [
        (include_str!("../../fixtures/shared/missing.json"), false),
        (
            include_str!("../../fixtures/shared/partial-dates.json"),
            true,
        ),
    ] {
        let cv = serde_json::from_str(source).unwrap();
        let (_, document) = render_pdf_document(
            &cv,
            &RenderOptions {
                project_dir: Some(
                    Path::new(env!("CARGO_MANIFEST_DIR"))
                        .join("../../docs/examples/customized-layout/.cac"),
                ),
                settings: crate::Settings {
                    theme: Some("table-layout".into()),
                    ..crate::Settings::default()
                },
                ..RenderOptions::default()
            },
        )
        .unwrap();
        let mut lines = Vec::new();
        for page in document.pages() {
            positions(&page.frame, Transform::identity(), &mut lines);
        }
        let x = |marker: &str| {
            lines
                .iter()
                .find(|(text, _)| text.contains(marker))
                .unwrap()
                .1
                .x
        };
        if dated {
            assert!(x("CACRoleZZZ") - x("CACSection0ZZZ") > Abs::pt(20.0));
            assert_eq!(x("CACRoleZZZ"), x("CACUndatedZZZ"));
        } else {
            assert_eq!(x("CACRoleZZZ"), x("CACSection0ZZZ"));
        }
    }
}

#[test]
fn rich_paragraphs_and_lists_do_not_overlap_neighboring_content() {
    let cv = serde_json::from_str(include_str!("../../fixtures/shared/flexible.json")).unwrap();
    let (_, document) = render_pdf_document(&cv, &RenderOptions::default()).unwrap();
    let mut lines = Vec::new();
    for page in document.pages() {
        positions(&page.frame, Transform::identity(), &mut lines);
    }
    let y = |marker: &str| {
        lines
            .iter()
            .find(|(text, _)| text.contains(marker))
            .unwrap()
            .1
            .y
    };
    for (upper, lower) in [
        ("CACWorkEmailZZZ", "CACSummaryFirstZZZ"),
        ("CACSummaryFirstZZZ", "CACSummarySecondZZZ"),
        ("CACSecondItemZZZ", "CACConclusionZZZ"),
    ] {
        assert!(
            y(lower) - y(upper) >= Abs::pt(10.0),
            "{upper} overlaps {lower}"
        );
    }
}
