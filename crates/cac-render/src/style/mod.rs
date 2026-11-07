//! The style contract module: types, defaults, and schema documents.
//!
//! The defaults here mirror `crates/cac-render/src/typst/base.typ` exactly;
//! the Typst side reads the resolved sheet in the next task, and the
//! determinism test doubles as the parity proof then.

pub mod legacy;
pub mod resolve;
pub mod schema;

pub use legacy::{ShimmedSettings, classic_family, shim};
pub use resolve::{materialize, overlay, resolve, resolve_layered};
pub use schema::{
    AtsMode, BackgroundSpecs, ColorOverrides, ColorSpec, ColorSpecs, DatePosition, EntryStyles,
    FontSpecs, HeaderAlign, HeaderStyles, HeadingSlot, HeadingStyles, HeadingTransform,
    LEGACY_SETTINGS_MAP, LayoutOptions, LayoutSpec, LegacyKeyMap, Length, LengthSpec, LengthUnit,
    LevelStyle, LinkStyles, ListStyles, MarginSpecs, OptionValue, PageStyles, Paper, PhotoShape,
    PhotoSlot, Requires, ResolvedColors, ResolvedEntry, ResolvedFonts, ResolvedHeader,
    ResolvedHeading, ResolvedHeadings, ResolvedLinks, ResolvedLists, ResolvedMargins, ResolvedPage,
    ResolvedSectionHeading, ResolvedSpacing, ResolvedStyle, ResolvedText, RuleStyle,
    SectionHeadingSlot, SpacingStyles, StyleContractError, StyleLayer, StyleOverrides,
    SubtitleStyle, TextAlign, TextStyles, ThemeKind, ThemeManifest, ThemeMeta, Weight, legacy_key,
    parse_length, style_schema_document, theme_schema_document,
};

/// The built-in style sheet. Values mirror `base.typ`'s default tokens,
/// styles, and page dictionaries so the resolved sheet can replace them
/// without moving a single point.
pub fn default_overrides() -> StyleOverrides {
    StyleOverrides {
        page: Some(PageStyles {
            paper: Some(Paper::A4),
            margin: Some(four_sides(LengthSpec("18mm".into()))),
        }),
        fonts: Some(FontSpecs {
            body: Some("Libertinus Serif".into()),
            heading: Some("Libertinus Serif".into()),
            mono: Some("DejaVu Sans Mono".into()),
        }),
        colors: Some(ColorOverrides {
            text: Some(ColorSpec("#000000".into())),
            // Typst's predefined `gray` is luma(170).
            muted: Some(ColorSpec("#aaaaaa".into())),
            accent: Some(ColorSpec("#000000".into())),
            rule: None,
        }),
        text: Some(TextStyles {
            size: Some(LengthSpec("10pt".into())),
            leading: Some(LengthSpec("0.65em".into())),
            paragraph_gap: Some(LengthSpec("1.2em".into())),
            align: Some(TextAlign::Justified),
        }),
        header: Some(HeaderStyles {
            align: Some(HeaderAlign::Left),
            rule: Some(RuleStyle::None),
        }),
        headings: Some(HeadingStyles {
            name: Some(HeadingSlot {
                size: None,
                weight: Some(Weight::Bold),
                color: Some(ColorSpec("text".into())),
                space_before: Some(LengthSpec("0pt".into())),
                space_after: Some(LengthSpec("1.2em".into())),
            }),
            section: Some(SectionHeadingSlot {
                size: None,
                weight: Some(Weight::Bold),
                color: Some(ColorSpec("text".into())),
                transform: Some(HeadingTransform::None),
                rule: Some(RuleStyle::None),
                space_before: Some(LengthSpec("0pt".into())),
                space_after: Some(LengthSpec("0.65em".into())),
            }),
            entry: Some(HeadingSlot {
                size: None,
                weight: Some(Weight::Bold),
                color: Some(ColorSpec("text".into())),
                space_before: Some(LengthSpec("0pt".into())),
                space_after: Some(LengthSpec("0.65em".into())),
            }),
        }),
        spacing: Some(SpacingStyles {
            section: Some(LengthSpec("1.2em".into())),
            heading_gap: Some(LengthSpec("0.65em".into())),
            entry: Some(LengthSpec("1.2em".into())),
            // The legacy `auto`: follow the text leading.
            item: None,
        }),
        lists: Some(ListStyles {
            bullet: Some("•".into()),
        }),
        links: Some(LinkStyles {
            underline: Some(true),
        }),
        entry: Some(EntryStyles {
            date_position: Some(DatePosition::Right),
            subtitle_style: Some(SubtitleStyle::Italic),
            // The legacy default refuses entry page breaks.
            keep_together: Some(true),
        }),
        locale: Some("en".into()),
        accent_color: None,
        body_alignment: None,
        header_alignment: None,
        link_underline: None,
        highlight_bullet: None,
    }
}

fn four_sides(length: LengthSpec) -> MarginSpecs {
    MarginSpecs {
        top: Some(length.clone()),
        right: Some(length.clone()),
        bottom: Some(length.clone()),
        left: Some(length),
    }
}

/// The built-in defaults, resolved. Falls back only if the built-in constants
/// above are malformed, which the test suite pins.
pub fn defaults() -> ResolvedStyle {
    resolve_defaults().expect("built-in defaults are well-formed")
}

fn resolve_defaults() -> Result<ResolvedStyle, StyleContractError> {
    crate::style::resolve::materialize(&default_overrides())
}

#[cfg(test)]
mod tests;
