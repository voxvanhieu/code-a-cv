//! The style contract module: types, defaults, and schema documents.
//!
//! The defaults here mirror `crates/cac-render/src/typst/base.typ` exactly;
//! the Typst side reads the resolved sheet in the next task, and the
//! determinism test doubles as the parity proof then.

pub mod schema;

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
    let sheet = default_overrides();
    let colors = ColorSpecs {
        text: sheet
            .colors
            .as_ref()
            .and_then(|colors| colors.text.as_ref())
            .map(|color| color.resolve(&ColorSpecs::default()))
            .transpose()?
            .unwrap_or_else(|| "#000000".into()),
        muted: sheet
            .colors
            .as_ref()
            .and_then(|colors| colors.muted.as_ref())
            .map(|color| color.resolve(&ColorSpecs::default()))
            .transpose()?
            .unwrap_or_else(|| "#aaaaaa".into()),
        accent: sheet
            .colors
            .as_ref()
            .and_then(|colors| colors.accent.as_ref())
            .map(|color| color.resolve(&ColorSpecs::default()))
            .transpose()?
            .unwrap_or_else(|| "#000000".into()),
        rule: "#000000".into(),
    };
    let text = sheet.text.as_ref().expect("text defaults");
    let length = |spec: &Option<LengthSpec>| spec.as_ref().map(|spec| spec.parse()).transpose();
    Ok(ResolvedStyle {
        page: ResolvedPage {
            paper: sheet
                .page
                .as_ref()
                .and_then(|page| page.paper)
                .unwrap_or(Paper::A4),
            margin: ResolvedMargins {
                top: sheet
                    .page
                    .as_ref()
                    .and_then(|page| page.margin.as_ref())
                    .and_then(|margin| margin.top.clone())
                    .map(|spec| spec.parse())
                    .transpose()?
                    .unwrap_or(Length::new(18.0, LengthUnit::Mm)),
                right: sheet
                    .page
                    .as_ref()
                    .and_then(|page| page.margin.as_ref())
                    .and_then(|margin| margin.right.clone())
                    .map(|spec| spec.parse())
                    .transpose()?
                    .unwrap_or(Length::new(18.0, LengthUnit::Mm)),
                bottom: sheet
                    .page
                    .as_ref()
                    .and_then(|page| page.margin.as_ref())
                    .and_then(|margin| margin.bottom.clone())
                    .map(|spec| spec.parse())
                    .transpose()?
                    .unwrap_or(Length::new(18.0, LengthUnit::Mm)),
                left: sheet
                    .page
                    .as_ref()
                    .and_then(|page| page.margin.as_ref())
                    .and_then(|margin| margin.left.clone())
                    .map(|spec| spec.parse())
                    .transpose()?
                    .unwrap_or(Length::new(18.0, LengthUnit::Mm)),
            },
        },
        fonts: ResolvedFonts {
            body: sheet
                .fonts
                .as_ref()
                .and_then(|fonts| fonts.body.clone())
                .unwrap_or_else(|| "Libertinus Serif".into()),
            heading: sheet
                .fonts
                .as_ref()
                .and_then(|fonts| fonts.heading.clone())
                .unwrap_or_else(|| "Libertinus Serif".into()),
            mono: sheet
                .fonts
                .as_ref()
                .and_then(|fonts| fonts.mono.clone())
                .unwrap_or_else(|| "DejaVu Sans Mono".into()),
        },
        colors: {
            // The rule color defaults to the accent token.
            let rule = sheet
                .colors
                .as_ref()
                .and_then(|sheet_colors| sheet_colors.rule.as_ref())
                .map(|color| color.resolve(&colors))
                .transpose()?
                .unwrap_or_else(|| colors.accent.clone());
            ResolvedColors {
                text: colors.text.clone(),
                muted: colors.muted.clone(),
                accent: colors.accent.clone(),
                rule,
            }
        },
        text: ResolvedText {
            size: length(&text.size)?.unwrap_or(Length::new(10.0, LengthUnit::Pt)),
            leading: length(&text.leading)?.unwrap_or(Length::new(0.65, LengthUnit::Em)),
            paragraph_gap: length(&text.paragraph_gap)?.unwrap_or(Length::new(1.2, LengthUnit::Em)),
            align: text.align.unwrap_or(TextAlign::Justified),
        },
        header: ResolvedHeader {
            align: sheet
                .header
                .as_ref()
                .and_then(|header| header.align)
                .unwrap_or(HeaderAlign::Left),
            rule: sheet
                .header
                .as_ref()
                .and_then(|header| header.rule)
                .unwrap_or(RuleStyle::None),
        },
        headings: ResolvedHeadings {
            name: resolve_heading(
                sheet
                    .headings
                    .as_ref()
                    .and_then(|headings| headings.name.clone()),
                &colors,
                HeadingSlot {
                    space_after: Some(LengthSpec("1.2em".into())),
                    ..Default::default()
                },
            )?,
            section: {
                let fallback = SectionHeadingSlot {
                    space_after: Some(LengthSpec("0.65em".into())),
                    ..Default::default()
                };
                let slot = sheet
                    .headings
                    .as_ref()
                    .and_then(|headings| headings.section.clone())
                    .unwrap_or(fallback.clone());
                let heading = resolve_heading(
                    Some(heading_into_slot(slot.clone())),
                    &colors,
                    heading_into_slot(fallback),
                )?;
                ResolvedSectionHeading {
                    size: heading.size,
                    weight: heading.weight,
                    color: heading.color,
                    transform: slot.transform.unwrap_or(HeadingTransform::None),
                    rule: slot.rule.unwrap_or(RuleStyle::None),
                    space_before: heading.space_before,
                    space_after: heading.space_after,
                }
            },
            entry: resolve_heading(
                sheet
                    .headings
                    .as_ref()
                    .and_then(|headings| headings.entry.clone()),
                &colors,
                HeadingSlot {
                    space_after: Some(LengthSpec("0.65em".into())),
                    ..Default::default()
                },
            )?,
        },
        spacing: {
            let spacing = sheet.spacing.as_ref().expect("spacing defaults");
            ResolvedSpacing {
                section: length(&spacing.section)?.unwrap_or(Length::new(1.2, LengthUnit::Em)),
                entry: length(&spacing.entry)?.unwrap_or(Length::new(1.2, LengthUnit::Em)),
                item: length(&spacing.item)?,
                heading_gap: Length::new(0.65, LengthUnit::Em),
            }
        },
        lists: ResolvedLists {
            bullet: sheet
                .lists
                .as_ref()
                .and_then(|lists| lists.bullet.clone())
                .unwrap_or_else(|| "•".into()),
        },
        links: ResolvedLinks {
            underline: sheet
                .links
                .as_ref()
                .and_then(|links| links.underline)
                .unwrap_or(true),
        },
        entry: {
            let entry = sheet.entry.as_ref().expect("entry defaults");
            ResolvedEntry {
                date_position: entry.date_position.unwrap_or(DatePosition::Right),
                subtitle_style: entry.subtitle_style.unwrap_or(SubtitleStyle::Italic),
                keep_together: entry.keep_together.unwrap_or(true),
            }
        },
        locale: sheet.locale.clone().unwrap_or_else(|| "en".into()),
    })
}

fn resolve_heading(
    slot: Option<HeadingSlot>,
    colors: &ColorSpecs,
    fallback: HeadingSlot,
) -> Result<ResolvedHeading, StyleContractError> {
    let slot = slot.unwrap_or(fallback);
    let parse = |spec: &Option<LengthSpec>| spec.as_ref().map(|spec| spec.parse()).transpose();
    Ok(ResolvedHeading {
        size: parse(&slot.size)?,
        weight: slot.weight.unwrap_or(Weight::Regular),
        color: slot
            .color
            .map(|color| color.resolve(colors))
            .transpose()?
            .unwrap_or_else(|| colors.text.clone()),
        space_before: parse(&slot.space_before)?.unwrap_or(Length::new(0.0, LengthUnit::Pt)),
        space_after: parse(&slot.space_after)?.unwrap_or(Length::new(0.0, LengthUnit::Pt)),
    })
}

fn heading_into_slot(section: SectionHeadingSlot) -> HeadingSlot {
    HeadingSlot {
        size: section.size,
        weight: section.weight,
        color: section.color,
        space_before: section.space_before,
        space_after: section.space_after,
    }
}

#[cfg(test)]
mod tests;
