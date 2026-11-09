//! The legacy settings shim: every 0.3 style key maps to the new sheet.
//!
//! The mapping table lives in [`super::schema::LEGACY_SETTINGS_MAP`]. Each
//! legacy key in use adds a deprecation advisory naming its replacement, so
//! a project can migrate one key at a time for one minor release. The new
//! `[style]` groups win wherever a deprecated leaf and its replacement are
//! both present.

use super::schema::{LengthSpec, legacy_key};
use crate::settings::Settings;

/// The style sheet a legacy settings file produces, with its advisories.
pub struct ShimmedSettings {
    pub style: StyleOverrides,
    pub warnings: Vec<String>,
}

use super::schema::StyleOverrides;

/// Map every legacy settings group, plus the deprecated `[style]` leaves,
/// onto the new style sheet.
pub fn shim(settings: &Settings) -> ShimmedSettings {
    let mut style = StyleOverrides::default();
    let mut warnings: Vec<String> = Vec::new();
    let mut advise = |old: &str| {
        let Some(entry) = legacy_key(old) else { return };
        let note = match entry.note {
            "inverted" => {
                " (the boolean flips: keep_together is the inverse of allow_entry_page_break)"
                    .to_string()
            }
            "all-sides" => " (applies to all four sides)".to_string(),
            _ => String::new(),
        };
        warnings.push(format!(
            "`settings.{old}` is deprecated; use the style-sheet key `{}`{note}",
            entry.new
        ));
    };

    if let Some(page) = &settings.page {
        if let Some(paper) = &page.paper {
            style.page.get_or_insert_with(Default::default).paper = sheet_paper(paper);
            advise("page.paper");
        }
        if page.margin.is_some() {
            let spec = page.margin.clone().map(LengthSpec);
            let sheet_margin = style
                .page
                .get_or_insert_with(Default::default)
                .margin
                .get_or_insert_with(Default::default);
            sheet_margin.top = spec.clone();
            sheet_margin.right = spec.clone();
            sheet_margin.bottom = spec.clone();
            sheet_margin.left = spec;
        }
        if let Some(margins) = &page.margins {
            let sheet_margin = style
                .page
                .get_or_insert_with(Default::default)
                .margin
                .get_or_insert_with(Default::default);
            for (old, target, value) in [
                ("page.margins.top", &mut sheet_margin.top, &margins.top),
                (
                    "page.margins.right",
                    &mut sheet_margin.right,
                    &margins.right,
                ),
                (
                    "page.margins.bottom",
                    &mut sheet_margin.bottom,
                    &margins.bottom,
                ),
                ("page.margins.left", &mut sheet_margin.left, &margins.left),
            ] {
                if let Some(value) = value {
                    *target = Some(LengthSpec(value.clone()));
                    advise(old);
                }
            }
        }
    }
    if let Some(typography) = &settings.typography {
        if let Some(font) = &typography.font {
            style.fonts.get_or_insert_with(Default::default).body = Some(font.clone());
            advise("typography.font");
        }
        if let Some(font) = &typography.heading_font {
            style.fonts.get_or_insert_with(Default::default).heading = Some(font.clone());
            advise("typography.heading_font");
        }
        if let Some(size) = &typography.font_size {
            style.text.get_or_insert_with(Default::default).size = Some(LengthSpec(size.clone()));
            advise("typography.font_size");
        }
        if let Some(leading) = &typography.line_spacing {
            style.text.get_or_insert_with(Default::default).leading =
                Some(LengthSpec(leading.clone()));
            advise("typography.line_spacing");
        }
    }
    if let Some(spacing) = &settings.spacing {
        if let Some(item) = &spacing.list_item {
            style.spacing.get_or_insert_with(Default::default).item =
                Some(LengthSpec(item.clone()));
            advise("spacing.list_item");
        }
        if let Some(section) = &spacing.section {
            style.spacing.get_or_insert_with(Default::default).section =
                Some(LengthSpec(section.clone()));
            advise("spacing.section");
        }
        if let Some(gap) = &spacing.section_heading {
            // The 0.3 knob was the gap between a section heading and its
            // entries; the sheet spells it spacing.heading_gap.
            style
                .spacing
                .get_or_insert_with(Default::default)
                .heading_gap = Some(LengthSpec(gap.clone()));
            advise("spacing.section_heading");
        }
        if let Some(entry) = &spacing.entry {
            style.spacing.get_or_insert_with(Default::default).entry =
                Some(LengthSpec(entry.clone()));
            advise("spacing.entry");
        }
    }
    if let Some(pagination) = &settings.pagination
        && let Some(allow) = pagination.allow_entry_page_break
    {
        style
            .entry
            .get_or_insert_with(Default::default)
            .keep_together = Some(!allow);
        advise("pagination.allow_entry_page_break");
    }

    // The deprecated `[style]` leaves and their new-group equivalents.
    if let Some(sheet) = &settings.style {
        if let Some(accent) = &sheet.accent_color {
            style.colors.get_or_insert_with(Default::default).accent = Some(accent.clone());
            advise("style.accent_color");
        }
        if let Some(align) = sheet.body_alignment {
            style.text.get_or_insert_with(Default::default).align = Some(align);
            advise("style.body_alignment");
        }
        if let Some(align) = sheet.header_alignment {
            style.header.get_or_insert_with(Default::default).align = Some(align);
            advise("style.header_alignment");
        }
        if let Some(underline) = sheet.link_underline {
            style.links.get_or_insert_with(Default::default).underline = Some(underline);
            advise("style.link_underline");
        }
        if let Some(bullet) = &sheet.highlight_bullet {
            style.lists.get_or_insert_with(Default::default).bullet = Some(bullet.clone());
            advise("style.highlight_bullet");
        }
        // The new groups overlay the deprecated leaves — the modern
        // spelling always wins in the same layer.
        style = super::resolve::overlay(&style, &without_deprecated_leaves(sheet));
    }

    ShimmedSettings { style, warnings }
}

/// The sheet with only its modern groups, for layering over the leaves.
fn without_deprecated_leaves(sheet: &StyleOverrides) -> StyleOverrides {
    let mut sheet = sheet.clone();
    sheet.accent_color = None;
    sheet.body_alignment = None;
    sheet.header_alignment = None;
    sheet.link_underline = None;
    sheet.highlight_bullet = None;
    sheet
}

/// Whether the deprecated spellings were used at all.
#[allow(dead_code)] // superseded by the shim; kept for the diagnostics road
pub fn uses_legacy_keys(settings: &Settings) -> bool {
    settings.page.is_some()
        || settings.typography.is_some()
        || settings.spacing.is_some()
        || settings.pagination.is_some()
        || settings.style.as_ref().is_some_and(|sheet| {
            sheet.accent_color.is_some()
                || sheet.body_alignment.is_some()
                || sheet.header_alignment.is_some()
                || sheet.link_underline.is_some()
                || sheet.highlight_bullet.is_some()
        })
}

/// The classic family's own values, which used to live in the embedded
/// classic.typ. They sit between the defaults and user settings: the sheet
/// resolver layers them below the theme and settings so a user's 9.5pt
/// still wins over the family's 10pt.
pub fn classic_family() -> StyleOverrides {
    serde_json::from_str(
        r##"{
            "page": { "paper": "us-letter", "margin": { "top": "12.7mm", "right": "12.7mm", "bottom": "12.7mm", "left": "12.7mm" } },
            "fonts": { "body": "New Computer Modern", "heading": "New Computer Modern" },
            "text": { "size": "10pt" },
            "header": { "align": "center" },
            "headings": {
                "name": { "size": "24pt" },
                "section": { "size": "12pt", "weight": "regular" },
                "entry": { "size": "10pt" }
            }
        }"##,
    )
    .expect("the classic family sheet is well-formed")
}

/// The 0.3 paper spelling onto the sheet enum.
pub(super) fn sheet_paper(paper: &str) -> Option<super::schema::Paper> {
    match paper {
        "a4" => Some(super::schema::Paper::A4),
        "us-letter" => Some(super::schema::Paper::UsLetter),
        _ => None,
    }
}
