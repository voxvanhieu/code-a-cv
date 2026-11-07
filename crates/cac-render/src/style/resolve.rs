//! The three-layer resolver: defaults, then the theme, then settings.
//!
//! Layers merge key-wise in Rust and materialize once into
//! [`ResolvedStyle`] — concrete lengths with unit tags and hex colors, so
//! neither renderer evaluates a string. Precedence is what it has always
//! been: the user's settings win, then the theme, then the built-ins.
//! Deprecated 0.3 spellings inside a layer are resolved by
//! `super::legacy::shim` before they reach here.

use super::schema::{
    ColorOverrides, ColorSpecs, EntryStyles, FontSpecs, HeaderStyles, HeadingSlot, HeadingStyles,
    Length, LengthSpec, LengthUnit, MarginSpecs, PageStyles, ResolvedColors, ResolvedEntry,
    ResolvedFonts, ResolvedHeader, ResolvedHeading, ResolvedHeadings, ResolvedLinks, ResolvedLists,
    ResolvedMargins, ResolvedPage, ResolvedSectionHeading, ResolvedSpacing, ResolvedStyle,
    ResolvedText, SectionHeadingSlot, SpacingStyles, StyleContractError, StyleOverrides, TextAlign,
    TextStyles,
};

/// Overlay `upper` onto `lower`: keys the upper layer sets win.
pub fn overlay(lower: &StyleOverrides, upper: &StyleOverrides) -> StyleOverrides {
    let merged = lower.clone();
    let mut merged = merged;
    if upper.page.is_some() {
        merged.page = overlay_page(lower.page.as_ref(), upper.page.as_ref());
    }
    if upper.fonts.is_some() {
        merged.fonts = overlay_fonts(lower.fonts.as_ref(), upper.fonts.as_ref());
    }
    if upper.colors.is_some() {
        merged.colors = overlay_colors(lower.colors.as_ref(), upper.colors.as_ref());
    }
    if upper.text.is_some() {
        merged.text = overlay_text(lower.text.as_ref(), upper.text.as_ref());
    }
    if upper.header.is_some() {
        merged.header = overlay_header(lower.header.as_ref(), upper.header.as_ref());
    }
    if upper.headings.is_some() {
        merged.headings = overlay_headings(lower.headings.as_ref(), upper.headings.as_ref());
    }
    if upper.spacing.is_some() {
        merged.spacing = overlay_spacing(lower.spacing.as_ref(), upper.spacing.as_ref());
    }
    if upper.lists.is_some() {
        merged.lists = overlay_lists(lower.lists.as_ref(), upper.lists.as_ref());
    }
    if upper.links.is_some() {
        merged.links = overlay_links(lower.links.as_ref(), upper.links.as_ref());
    }
    if upper.entry.is_some() {
        merged.entry = overlay_entry(lower.entry.as_ref(), upper.entry.as_ref());
    }
    merged.locale = upper.locale.clone().or_else(|| lower.locale.clone());
    merged
}

fn overlay_page(lower: Option<&PageStyles>, upper: Option<&PageStyles>) -> Option<PageStyles> {
    let merged = lower.cloned().unwrap_or_default();
    let upper = upper?;
    Some(PageStyles {
        paper: upper.paper.or(merged.paper),
        margin: match (upper.margin.clone(), merged.margin.clone()) {
            (Some(upper_margin), Some(lower_margin)) => Some(MarginSpecs {
                top: upper_margin.top.or(lower_margin.top),
                right: upper_margin.right.or(lower_margin.right),
                bottom: upper_margin.bottom.or(lower_margin.bottom),
                left: upper_margin.left.or(lower_margin.left),
            }),
            (upper_margin, lower_margin) => upper_margin.or(lower_margin),
        },
    })
}

fn overlay_fonts(lower: Option<&FontSpecs>, upper: Option<&FontSpecs>) -> Option<FontSpecs> {
    let merged = lower.cloned().unwrap_or_default();
    let upper = upper?;
    Some(FontSpecs {
        body: upper.body.clone().or(merged.body),
        heading: upper.heading.clone().or(merged.heading),
        mono: upper.mono.clone().or(merged.mono),
    })
}

fn overlay_colors(
    lower: Option<&ColorOverrides>,
    upper: Option<&ColorOverrides>,
) -> Option<ColorOverrides> {
    let merged = lower.cloned().unwrap_or_default();
    let upper = upper?;
    Some(ColorOverrides {
        text: upper.text.clone().or(merged.text),
        muted: upper.muted.clone().or(merged.muted),
        accent: upper.accent.clone().or(merged.accent),
        rule: upper.rule.clone().or(merged.rule),
    })
}

fn overlay_text(lower: Option<&TextStyles>, upper: Option<&TextStyles>) -> Option<TextStyles> {
    let merged = lower.cloned().unwrap_or_default();
    let upper = upper?;
    Some(TextStyles {
        size: upper.size.clone().or(merged.size),
        leading: upper.leading.clone().or(merged.leading),
        paragraph_gap: upper.paragraph_gap.clone().or(merged.paragraph_gap),
        align: upper.align.or(merged.align),
    })
}

fn overlay_header(
    lower: Option<&HeaderStyles>,
    upper: Option<&HeaderStyles>,
) -> Option<HeaderStyles> {
    let merged = lower.cloned().unwrap_or_default();
    let upper = upper?;
    Some(HeaderStyles {
        align: upper.align.or(merged.align),
        rule: upper.rule.or(merged.rule),
    })
}

fn overlay_headings(
    lower: Option<&HeadingStyles>,
    upper: Option<&HeadingStyles>,
) -> Option<HeadingStyles> {
    let merged = lower.cloned().unwrap_or_default();
    let upper = upper?;
    Some(HeadingStyles {
        name: overlay_heading_slot(merged.name.as_ref(), upper.name.as_ref()),
        section: overlay_section_slot(merged.section.as_ref(), upper.section.as_ref()),
        entry: overlay_heading_slot(merged.entry.as_ref(), upper.entry.as_ref()),
    })
}

fn overlay_heading_slot(
    lower: Option<&HeadingSlot>,
    upper: Option<&HeadingSlot>,
) -> Option<HeadingSlot> {
    let merged = lower.cloned().unwrap_or_default();
    let upper = upper?;
    Some(HeadingSlot {
        size: upper.size.clone().or(merged.size),
        weight: upper.weight.or(merged.weight),
        color: upper.color.clone().or(merged.color),
        space_before: upper.space_before.clone().or(merged.space_before),
        space_after: upper.space_after.clone().or(merged.space_after),
    })
}

fn overlay_section_slot(
    lower: Option<&SectionHeadingSlot>,
    upper: Option<&SectionHeadingSlot>,
) -> Option<SectionHeadingSlot> {
    let merged = lower.cloned().unwrap_or_default();
    let upper = upper?;
    Some(SectionHeadingSlot {
        size: upper.size.clone().or(merged.size),
        weight: upper.weight.or(merged.weight),
        color: upper.color.clone().or(merged.color),
        transform: upper.transform.or(merged.transform),
        rule: upper.rule.or(merged.rule),
        space_before: upper.space_before.clone().or(merged.space_before),
        space_after: upper.space_after.clone().or(merged.space_after),
    })
}

fn overlay_spacing(
    lower: Option<&SpacingStyles>,
    upper: Option<&SpacingStyles>,
) -> Option<SpacingStyles> {
    let merged = lower.cloned().unwrap_or_default();
    let upper = upper?;
    Some(SpacingStyles {
        section: upper.section.clone().or(merged.section),
        heading_gap: upper.heading_gap.clone().or(merged.heading_gap),
        entry: upper.entry.clone().or(merged.entry),
        item: upper.item.clone().or(merged.item),
    })
}

fn overlay_lists(
    lower: Option<&super::schema::ListStyles>,
    upper: Option<&super::schema::ListStyles>,
) -> Option<super::schema::ListStyles> {
    let merged = lower.cloned().unwrap_or_default();
    let upper = upper?;
    Some(super::schema::ListStyles {
        bullet: upper.bullet.clone().or(merged.bullet),
    })
}

fn overlay_links(
    lower: Option<&super::schema::LinkStyles>,
    upper: Option<&super::schema::LinkStyles>,
) -> Option<super::schema::LinkStyles> {
    let merged = lower.cloned().unwrap_or_default();
    let upper = upper?;
    Some(super::schema::LinkStyles {
        underline: upper.underline.or(merged.underline),
    })
}

fn overlay_entry(lower: Option<&EntryStyles>, upper: Option<&EntryStyles>) -> Option<EntryStyles> {
    let merged = lower.cloned().unwrap_or_default();
    let upper = upper?;
    Some(EntryStyles {
        date_position: upper.date_position.or(merged.date_position),
        subtitle_style: upper.subtitle_style.or(merged.subtitle_style),
        keep_together: upper.keep_together.or(merged.keep_together),
    })
}

/// Materialize a style sheet into the resolved form both renderers read.
pub fn materialize(sheet: &StyleOverrides) -> Result<ResolvedStyle, StyleContractError> {
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
    let rule = sheet
        .colors
        .as_ref()
        .and_then(|colors| colors.rule.as_ref())
        .map(|color| color.resolve(&colors))
        .transpose()?
        .unwrap_or_else(|| colors.accent.clone());
    let text = sheet.text.as_ref();
    let length = |spec: &Option<LengthSpec>| spec.as_ref().map(|spec| spec.parse()).transpose();
    let headings = sheet.headings.as_ref();
    Ok(ResolvedStyle {
        page: ResolvedPage {
            paper: sheet
                .page
                .as_ref()
                .and_then(|page| page.paper)
                .unwrap_or(super::schema::Paper::A4),
            margin: ResolvedMargins {
                top: page_margin(sheet, |margin| margin.top.clone())?,
                right: page_margin(sheet, |margin| margin.right.clone())?,
                bottom: page_margin(sheet, |margin| margin.bottom.clone())?,
                left: page_margin(sheet, |margin| margin.left.clone())?,
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
        colors: ResolvedColors {
            text: colors.text.clone(),
            muted: colors.muted.clone(),
            accent: colors.accent.clone(),
            rule,
        },
        text: ResolvedText {
            size: length(&text.and_then(|text| text.size.clone()))?
                .unwrap_or(Length::new(10.0, LengthUnit::Pt)),
            leading: length(&text.and_then(|text| text.leading.clone()))?
                .unwrap_or(Length::new(0.65, LengthUnit::Em)),
            paragraph_gap: length(&text.and_then(|text| text.paragraph_gap.clone()))?
                .unwrap_or(Length::new(1.2, LengthUnit::Em)),
            align: text
                .and_then(|text| text.align)
                .unwrap_or(TextAlign::Justified),
        },
        header: ResolvedHeader {
            align: sheet
                .header
                .as_ref()
                .and_then(|header| header.align)
                .unwrap_or(super::schema::HeaderAlign::Left),
            rule: sheet
                .header
                .as_ref()
                .and_then(|header| header.rule)
                .unwrap_or(super::schema::RuleStyle::None),
        },
        headings: ResolvedHeadings {
            name: materialize_heading(
                headings.and_then(|headings| headings.name.clone()),
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
                let slot = headings
                    .and_then(|headings| headings.section.clone())
                    .unwrap_or(fallback.clone());
                let heading = materialize_heading(
                    Some(super::schema::heading_into_slot(slot.clone())),
                    &colors,
                    super::schema::heading_into_slot(fallback),
                )?;
                ResolvedSectionHeading {
                    size: heading.size,
                    weight: heading.weight,
                    color: heading.color,
                    transform: slot
                        .transform
                        .unwrap_or(super::schema::HeadingTransform::None),
                    rule: slot.rule.unwrap_or(super::schema::RuleStyle::None),
                    space_before: heading.space_before,
                    space_after: heading.space_after,
                }
            },
            entry: materialize_heading(
                headings.and_then(|headings| headings.entry.clone()),
                &colors,
                HeadingSlot {
                    space_after: Some(LengthSpec("0.65em".into())),
                    ..Default::default()
                },
            )?,
        },
        spacing: {
            let spacing = sheet.spacing.clone().unwrap_or_default();
            ResolvedSpacing {
                section: length(&spacing.section)?.unwrap_or(Length::new(1.2, LengthUnit::Em)),
                entry: length(&spacing.entry)?.unwrap_or(Length::new(1.2, LengthUnit::Em)),
                item: spacing
                    .item
                    .as_ref()
                    .map(|item| item.parse_or_auto())
                    .transpose()?
                    .unwrap_or(None),
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
            let entry = sheet.entry.clone().unwrap_or_default();
            ResolvedEntry {
                date_position: entry
                    .date_position
                    .unwrap_or(super::schema::DatePosition::Right),
                subtitle_style: entry
                    .subtitle_style
                    .unwrap_or(super::schema::SubtitleStyle::Italic),
                keep_together: entry.keep_together.unwrap_or(true),
            }
        },
        locale: sheet.locale.clone().unwrap_or_else(|| "en".into()),
    })
}

fn page_margin(
    sheet: &StyleOverrides,
    pick: impl Fn(&MarginSpecs) -> Option<LengthSpec>,
) -> Result<Length, StyleContractError> {
    const DEFAULT_MM: f64 = 18.0;
    let spec = sheet
        .page
        .as_ref()
        .and_then(|page| page.margin.as_ref())
        .and_then(pick);
    match spec {
        Some(spec) => Ok(spec.parse()?),
        None => Ok(Length::new(DEFAULT_MM, LengthUnit::Mm)),
    }
}

fn materialize_heading(
    slot: Option<HeadingSlot>,
    colors: &ColorSpecs,
    fallback: HeadingSlot,
) -> Result<ResolvedHeading, StyleContractError> {
    let slot = slot.unwrap_or(fallback);
    let parse = |spec: &Option<LengthSpec>| spec.as_ref().map(|spec| spec.parse()).transpose();
    Ok(ResolvedHeading {
        size: parse(&slot.size)?,
        weight: slot.weight.unwrap_or(super::schema::Weight::Regular),
        color: slot
            .color
            .map(|color| color.resolve(colors))
            .transpose()?
            .unwrap_or_else(|| colors.text.clone()),
        space_before: parse(&slot.space_before)?.unwrap_or(Length::new(0.0, LengthUnit::Pt)),
        space_after: parse(&slot.space_after)?.unwrap_or(Length::new(0.0, LengthUnit::Pt)),
    })
}

/// Merge the three layers and materialize the sheet.
pub fn resolve(
    theme: &StyleOverrides,
    settings: &StyleOverrides,
) -> Result<ResolvedStyle, StyleContractError> {
    let merged = overlay(&overlay(&super::default_overrides(), theme), settings);
    materialize(&merged)
}

/// A resolver note that is not an error: deprecated spellings in use.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StyleAdvisory {
    pub message: String,
}

/// Resolve, collecting deprecation advisories from the shim.
pub fn resolve_layered(
    theme: Option<&StyleOverrides>,
    settings: &super::legacy::ShimmedSettings,
) -> Result<(ResolvedStyle, Vec<StyleAdvisory>), StyleContractError> {
    let empty = StyleOverrides::default();
    let theme = theme.unwrap_or(&empty);
    let merged = overlay(
        &overlay(&super::default_overrides(), theme),
        &settings.style,
    );
    let resolved = materialize(&merged)?;
    Ok((
        resolved,
        settings
            .warnings
            .iter()
            .cloned()
            .map(|message| StyleAdvisory { message })
            .collect(),
    ))
}
