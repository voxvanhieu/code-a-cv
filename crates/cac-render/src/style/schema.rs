//! The style contract: one vocabulary shared by defaults, `theme.toml`,
//! and `settings.json`.
//!
//! Three layers speak these types:
//!
//! - **Authoring layer** — [`StyleOverrides`] (every key optional) is the
//!   `[style]` table inside both `theme.toml` and `settings.json`.
//!   [`ThemeManifest`] is the whole `theme.toml`.
//! - **Resolved layer** — [`ResolvedStyle`] is the fully merged sheet with
//!   concrete values that both renderers read. Colors arrive as hex; lengths
//!   arrive as numbers with a unit tag, so Typst never evaluates a string.
//! - **Legacy layer** — [`LEGACY_SETTINGS_MAP`] maps every old settings key
//!   to its new key for the deprecation shim.
//!
//! Length strings keep the authoring form (`"16mm"`, `"0.65em"`); the
//! resolved sheet carries [`Length`]. Colors accept hex (`"#1f4e79"`) or a
//! token name (`"accent"`) in the authoring layer; resolution looks token
//! names up in the sheet's color group. A hex color whose channels are all
//! equal resolves to the same grayscale value the renderers emit today.

use std::collections::BTreeMap;
use std::fmt::{self, Display, Formatter};

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Authoring primitives
// ---------------------------------------------------------------------------

/// A length as the author writes it: a non-negative number with an explicit
/// unit. Zero is allowed where spacing allows it (heading space, gaps).
///
/// Deserializes and serializes as the authoring string (`"16mm"`) — the
/// same spelling, so a settings file the tool writes back parses again.
/// The renderers read the parsed [`Length`], not this type.
#[derive(Clone, Debug, PartialEq, Eq, JsonSchema, Deserialize)]
pub struct LengthSpec(pub String);

impl Serialize for LengthSpec {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.0)
    }
}

impl LengthSpec {
    /// Parse `"16mm"`, `"0.65em"`, `"12%"`, … into a resolved length.
    pub fn parse(&self) -> Result<Length, StyleContractError> {
        parse_length(&self.0).ok_or_else(|| StyleContractError::Length(self.0.clone()))
    }

    /// Parse a length that may also be `auto`, resolving to `None`.
    pub fn parse_or_auto(&self) -> Result<Option<Length>, StyleContractError> {
        if self.0.trim().eq_ignore_ascii_case("auto") {
            return Ok(None);
        }
        self.parse().map(Some)
    }
}

impl Display for LengthSpec {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

/// Reinterpret a section heading slot as the plain heading slot it carries.
pub(super) fn heading_into_slot(section: SectionHeadingSlot) -> HeadingSlot {
    HeadingSlot {
        size: section.size,
        weight: section.weight,
        color: section.color,
        space_before: section.space_before,
        space_after: section.space_after,
    }
}

/// A color as the author writes it: hex (`"#1f4e79"`) or a token name
/// (`"text"`, `"muted"`, `"accent"`, `"rule"`).
#[derive(Clone, Debug, PartialEq, Eq, JsonSchema, Serialize, Deserialize)]
pub struct ColorSpec(pub String);

impl ColorSpec {
    /// The token name, when the spec references one.
    pub fn as_name(&self) -> Option<&str> {
        (!self.0.starts_with('#')).then_some(self.0.as_str())
    }

    /// The concrete hex value, when the spec is one.
    pub fn as_hex(&self) -> Option<&str> {
        is_hex_color(&self.0).then_some(self.0.as_str())
    }

    /// Resolve against the sheet's color tokens.
    pub fn resolve(&self, colors: &ColorSpecs) -> Result<String, StyleContractError> {
        match self.as_hex() {
            Some(hex) => Ok(hex.to_string()),
            None => colors.lookup(self.as_name().unwrap_or(&self.0)),
        }
    }
}

/// Where a style key may be set from.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StyleLayer {
    /// Built into the binary.
    Defaults,
    /// A theme's `theme.toml` `[style]` table.
    Theme,
    /// The user's `settings.json` `[style]` table.
    Settings,
}

impl Display for StyleLayer {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Defaults => "defaults",
            Self::Theme => "theme",
            Self::Settings => "settings",
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum StyleContractError {
    #[error(
        "invalid length `{0}`; expected a positive number with a unit (pt, mm, cm, in, em, rem, %)"
    )]
    Length(String),
    #[error(
        "invalid color `{0}`; expected a hex value such as `#1f4e79` or a token name (text, muted, accent, rule)"
    )]
    Color(String),
    #[error("unknown color token `{0}`; known tokens are text, muted, accent, rule")]
    UnknownColorToken(String),
    #[error("`theme.extends` may not reference itself")]
    ExtendsCycle,
}

// ---------------------------------------------------------------------------
// Authoring layer: the [style] table
// ---------------------------------------------------------------------------

/// One style sheet: every key optional, merged over the next-lower layer.
/// This is the `[style]` table inside `theme.toml` and `settings.json`.
#[derive(Clone, Debug, Default, PartialEq, Eq, JsonSchema, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct StyleOverrides {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub page: Option<PageStyles>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fonts: Option<FontSpecs>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub colors: Option<ColorOverrides>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text: Option<TextStyles>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub header: Option<HeaderStyles>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub headings: Option<HeadingStyles>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub spacing: Option<SpacingStyles>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lists: Option<ListStyles>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub links: Option<LinkStyles>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub entry: Option<EntryStyles>,
    /// BCP 47 tag used by the shared date formatter in both renderers
    /// (`en`, `de`, `fr`, `vi`, `ja` today).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub locale: Option<String>,
    // Deprecated 0.3 spellings, accepted for one minor release and mapped
    // to their sheet keys by the legacy shim.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub accent_color: Option<ColorSpec>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub body_alignment: Option<TextAlign>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub header_alignment: Option<HeaderAlign>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub link_underline: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub highlight_bullet: Option<String>,
}

impl StyleOverrides {
    /// Whether the table sets nothing, so it can be omitted from manifests.
    pub fn is_empty(&self) -> bool {
        self.page.is_none()
            && self.fonts.is_none()
            && self.colors.is_none()
            && self.text.is_none()
            && self.header.is_none()
            && self.headings.is_none()
            && self.spacing.is_none()
            && self.lists.is_none()
            && self.links.is_none()
            && self.entry.is_none()
            && self.locale.is_none()
            && self.accent_color.is_none()
            && self.body_alignment.is_none()
            && self.header_alignment.is_none()
            && self.link_underline.is_none()
            && self.highlight_bullet.is_none()
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, JsonSchema, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct PageStyles {
    pub paper: Option<Paper>,
    /// Per-side margins; the legacy single `page.margin` applies to all sides.
    pub margin: Option<MarginSpecs>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, JsonSchema, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct MarginSpecs {
    pub top: Option<LengthSpec>,
    pub right: Option<LengthSpec>,
    pub bottom: Option<LengthSpec>,
    pub left: Option<LengthSpec>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, JsonSchema, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct FontSpecs {
    pub body: Option<String>,
    pub heading: Option<String>,
    pub mono: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, JsonSchema, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct ColorOverrides {
    pub text: Option<ColorSpec>,
    pub muted: Option<ColorSpec>,
    pub accent: Option<ColorSpec>,
    /// Color of section heading rules and the header rule; defaults to `accent`.
    pub rule: Option<ColorSpec>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, JsonSchema, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct TextStyles {
    pub size: Option<LengthSpec>,
    pub leading: Option<LengthSpec>,
    pub paragraph_gap: Option<LengthSpec>,
    pub align: Option<TextAlign>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, JsonSchema, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct HeaderStyles {
    pub align: Option<HeaderAlign>,
    pub rule: Option<RuleStyle>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, JsonSchema, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct HeadingStyles {
    /// The profile name heading (heading level 1 today).
    pub name: Option<HeadingSlot>,
    /// Section titles (heading level 2 today).
    pub section: Option<SectionHeadingSlot>,
    /// Entry titles (heading level 3 today).
    pub entry: Option<HeadingSlot>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, JsonSchema, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct HeadingSlot {
    /// `None` inherits the body text size.
    pub size: Option<LengthSpec>,
    pub weight: Option<Weight>,
    pub color: Option<ColorSpec>,
    pub space_before: Option<LengthSpec>,
    pub space_after: Option<LengthSpec>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, JsonSchema, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct SectionHeadingSlot {
    pub size: Option<LengthSpec>,
    pub weight: Option<Weight>,
    pub color: Option<ColorSpec>,
    pub transform: Option<HeadingTransform>,
    /// Rule drawn under section titles; the color comes from `colors.rule`.
    pub rule: Option<RuleStyle>,
    pub space_before: Option<LengthSpec>,
    pub space_after: Option<LengthSpec>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, JsonSchema, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct SpacingStyles {
    /// Space above each section block.
    pub section: Option<LengthSpec>,
    /// Space between a section heading and its entries (the 0.3
    /// `spacing.section_heading`).
    pub heading_gap: Option<LengthSpec>,
    /// Space after each entry.
    pub entry: Option<LengthSpec>,
    /// Space between list items; the default follows the text leading.
    pub item: Option<LengthSpec>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, JsonSchema, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct ListStyles {
    pub bullet: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, JsonSchema, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct LinkStyles {
    pub underline: Option<bool>,
}

/// Presentation of dated entries; the layout families read these.
#[derive(Clone, Debug, Default, PartialEq, Eq, JsonSchema, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct EntryStyles {
    pub date_position: Option<DatePosition>,
    pub subtitle_style: Option<SubtitleStyle>,
    /// Keep each entry on one page when it fits. The legacy
    /// `pagination.allow_entry_page_break` is the inverse of this key.
    pub keep_together: Option<bool>,
}

// ---------------------------------------------------------------------------
// Authoring enums
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq, JsonSchema, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Paper {
    A4,
    UsLetter,
}

impl Paper {
    /// The CSS/Typst page-size spelling.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::A4 => "a4",
            Self::UsLetter => "us-letter",
        }
    }
}

impl Display for Paper {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, JsonSchema, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum TextAlign {
    Left,
    Justified,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, JsonSchema, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum HeaderAlign {
    Left,
    Center,
    Right,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, JsonSchema, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RuleStyle {
    None,
    Line,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, JsonSchema, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Weight {
    Light,
    Regular,
    Medium,
    SemiBold,
    Bold,
    Black,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, JsonSchema, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum HeadingTransform {
    None,
    Caps,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, JsonSchema, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum DatePosition {
    Right,
    Left,
    Below,
    Inline,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, JsonSchema, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SubtitleStyle {
    Italic,
    Plain,
    Muted,
}

// ---------------------------------------------------------------------------
// Authoring layer: theme.toml
// ---------------------------------------------------------------------------

/// A theme manifest: `.cac/themes/<name>/theme.toml` or a registry entry.
#[derive(Clone, Debug, Default, PartialEq, Eq, JsonSchema, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ThemeManifest {
    pub theme: ThemeMeta,
    #[serde(default, skip_serializing_if = "StyleOverrides::is_empty")]
    pub style: StyleOverrides,
    /// Values for options the layout family declares; validated by cac.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub options: BTreeMap<String, OptionValue>,
    /// Regions, photo, and backgrounds for layout families; declarative only.
    #[serde(default, skip_serializing_if = "LayoutSpec::is_empty")]
    pub layout: LayoutSpec,
    /// What a form template demands; universal themes leave this absent.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub requires: Option<Requires>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, JsonSchema, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ThemeMeta {
    pub name: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub description: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub author: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub license: String,
    /// The built-in family (`classic`, `sidebar`, `table`, `compact`) or
    /// another theme this one extends.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub extends: Option<String>,
    /// `universal` renders any CV; `form` may require packs and reject
    /// sections it cannot place.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub kind: Option<ThemeKind>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, JsonSchema, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ThemeKind {
    Universal,
    Form,
}

/// A scalar theme option value.
#[derive(Clone, Debug, PartialEq, Eq, JsonSchema, Serialize, Deserialize)]
#[serde(untagged)]
pub enum OptionValue {
    Text(String),
    Integer(i64),
    Boolean(bool),
}

/// Declarative layout: regions, the photo slot, and per-page backgrounds.
#[derive(Clone, Debug, Default, PartialEq, Eq, JsonSchema, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct LayoutSpec {
    pub photo: Option<PhotoSlot>,
    /// Full-bleed artwork per page; PDF only, paths relative to the theme.
    pub backgrounds: Option<BackgroundSpecs>,
    /// Region name to the section kinds or ids it places; `*` catches the rest.
    pub regions: BTreeMap<String, Vec<String>>,
    pub options: LayoutOptions,
}

impl LayoutSpec {
    fn is_empty(&self) -> bool {
        self.photo.is_none()
            && self.backgrounds.is_none()
            && self.regions.is_empty()
            && self.options == LayoutOptions::default()
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, JsonSchema, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct PhotoSlot {
    /// The region that places the photo (`header-right`, `sidebar`, …).
    pub region: String,
    pub shape: Option<PhotoShape>,
    pub size: Option<LengthSpec>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, JsonSchema, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum PhotoShape {
    Square,
    Rounded,
    Circle,
    Hexagon,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, JsonSchema, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct BackgroundSpecs {
    /// Artwork on the first page.
    pub first: Option<String>,
    /// Artwork on following pages; `None` repeats `first`.
    pub rest: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, JsonSchema, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct LayoutOptions {
    /// How skill levels render: bars, dots, or the named label.
    pub levels: Option<LevelStyle>,
    /// Number sections (01, 02, …).
    pub numbering: Option<bool>,
    /// Place dates in a left gutter.
    pub date_gutter: Option<bool>,
    /// Declares whether the design survives ATS parsing; `cac check --ats`
    /// warns when a decorative theme is built for an application pipeline.
    pub ats: Option<AtsMode>,
    /// A checkable claim: the build reports overflow past this page count.
    pub max_pages: Option<u32>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, JsonSchema, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum LevelStyle {
    Bar,
    Dots,
    Label,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, JsonSchema, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AtsMode {
    Friendly,
    Decorative,
}

/// What a `kind = "form"` theme demands before it builds.
#[derive(Clone, Debug, Default, PartialEq, Eq, JsonSchema, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Requires {
    pub packs: Vec<String>,
    pub sections: Vec<String>,
    /// Profile keys the form cannot render without (`photo`, `birth_date`, …).
    pub profile: Vec<String>,
}

// ---------------------------------------------------------------------------
// Resolved layer
// ---------------------------------------------------------------------------

/// A length as the renderers read it: a number plus an explicit unit, so
/// Typst never evaluates a string.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct Length {
    pub value: f64,
    pub unit: LengthUnit,
}

impl Length {
    pub fn new(value: f64, unit: LengthUnit) -> Self {
        Self { value, unit }
    }
}

impl Display for Length {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}{}", self.value, self.unit.suffix())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum LengthUnit {
    Pt,
    Mm,
    Cm,
    In,
    Em,
    Rem,
    #[serde(rename = "%")]
    Percent,
}

impl LengthUnit {
    fn suffix(&self) -> &'static str {
        match self {
            Self::Pt => "pt",
            Self::Mm => "mm",
            Self::Cm => "cm",
            Self::In => "in",
            Self::Em => "em",
            Self::Rem => "rem",
            Self::Percent => "%",
        }
    }
}

pub fn parse_length(text: &str) -> Option<Length> {
    const UNITS: [(&str, LengthUnit); 7] = [
        ("pt", LengthUnit::Pt),
        ("mm", LengthUnit::Mm),
        ("cm", LengthUnit::Cm),
        ("in", LengthUnit::In),
        ("em", LengthUnit::Em),
        ("rem", LengthUnit::Rem),
        ("%", LengthUnit::Percent),
    ];
    let text = text.trim();
    UNITS.iter().find_map(|(suffix, unit)| {
        let number = text.strip_suffix(suffix)?.trim().parse::<f64>().ok()?;
        (number.is_finite() && number >= 0.0).then_some(Length::new(number, *unit))
    })
}

fn is_hex_color(text: &str) -> bool {
    text.len() == 7
        && text.starts_with('#')
        && text.bytes().skip(1).all(|byte| byte.is_ascii_hexdigit())
}

/// The fully merged sheet both renderers read. Every value is concrete.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ResolvedStyle {
    pub page: ResolvedPage,
    pub fonts: ResolvedFonts,
    pub colors: ResolvedColors,
    pub text: ResolvedText,
    pub header: ResolvedHeader,
    pub headings: ResolvedHeadings,
    pub spacing: ResolvedSpacing,
    pub lists: ResolvedLists,
    pub links: ResolvedLinks,
    pub entry: ResolvedEntry,
    pub locale: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ResolvedPage {
    pub paper: Paper,
    pub margin: ResolvedMargins,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ResolvedMargins {
    pub top: Length,
    pub right: Length,
    pub bottom: Length,
    pub left: Length,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ResolvedFonts {
    pub body: String,
    pub heading: String,
    pub mono: String,
}

/// Concrete colors. Hex values whose channels are all equal resolve to the
/// grayscale color the renderers emit for that value today.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ResolvedColors {
    pub text: String,
    pub muted: String,
    pub accent: String,
    pub rule: String,
}

impl ResolvedColors {
    /// Authoring-layer view of the resolved tokens, so authoring color specs
    /// can resolve against an already-resolved sheet.
    pub fn specs(&self) -> ColorSpecs {
        ColorSpecs {
            text: self.text.clone(),
            muted: self.muted.clone(),
            accent: self.accent.clone(),
            rule: self.rule.clone(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ResolvedText {
    pub size: Length,
    pub leading: Length,
    pub paragraph_gap: Length,
    pub align: TextAlign,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ResolvedHeader {
    pub align: HeaderAlign,
    pub rule: RuleStyle,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ResolvedHeadings {
    pub name: ResolvedHeading,
    pub section: ResolvedSectionHeading,
    pub entry: ResolvedHeading,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ResolvedHeading {
    /// `None` means the body text size.
    pub size: Option<Length>,
    pub weight: Weight,
    pub color: String,
    pub space_before: Length,
    pub space_after: Length,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ResolvedSectionHeading {
    pub size: Option<Length>,
    pub weight: Weight,
    pub color: String,
    pub transform: HeadingTransform,
    pub rule: RuleStyle,
    pub space_before: Length,
    pub space_after: Length,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ResolvedSpacing {
    pub section: Length,
    pub entry: Length,
    /// `None` follows the text leading (the legacy `auto`).
    pub item: Option<Length>,
    /// Internal: space inserted between a section heading and its entries.
    pub heading_gap: Length,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ResolvedLists {
    pub bullet: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ResolvedLinks {
    pub underline: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ResolvedEntry {
    pub date_position: DatePosition,
    pub subtitle_style: SubtitleStyle,
    pub keep_together: bool,
}

// ---------------------------------------------------------------------------
// Color token resolution for the authoring layer
// ---------------------------------------------------------------------------

/// The four color tokens an authoring-layer color spec may reference.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ColorSpecs {
    pub text: String,
    pub muted: String,
    pub accent: String,
    pub rule: String,
}

impl ColorSpecs {
    /// Look a token name up; hex passthrough is handled by [`ColorSpec`].
    pub fn lookup(&self, name: &str) -> Result<String, StyleContractError> {
        let resolved = match name {
            "text" => &self.text,
            "muted" => &self.muted,
            "accent" => &self.accent,
            "rule" => &self.rule,
            _ => return Err(StyleContractError::UnknownColorToken(name.to_string())),
        };
        Ok(resolved.clone())
    }
}

// ---------------------------------------------------------------------------
// Legacy key mapping
// ---------------------------------------------------------------------------

/// One old settings key and the new style-sheet key that replaces it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LegacyKeyMap {
    /// The old `settings.json` key path, for example `typography.font_size`.
    pub old: &'static str,
    /// The new style-sheet key path, for example `text.size`.
    pub new: &'static str,
    /// Extra mapping rules a reader must know (`inverted` flips booleans,
    /// `all-sides` applies one margin to every side).
    pub note: &'static str,
}

/// Every legacy settings key that carries style. Project keys (`root`,
/// `naming`, `theme`, `themeProject`) are not style and stay where they are.
pub static LEGACY_SETTINGS_MAP: &[LegacyKeyMap] = &[
    LegacyKeyMap {
        old: "page.paper",
        new: "page.paper",
        note: "",
    },
    LegacyKeyMap {
        old: "page.margin",
        new: "page.margin",
        note: "all-sides",
    },
    LegacyKeyMap {
        old: "page.margins.top",
        new: "page.margin.top",
        note: "",
    },
    LegacyKeyMap {
        old: "page.margins.bottom",
        new: "page.margin.bottom",
        note: "",
    },
    LegacyKeyMap {
        old: "page.margins.left",
        new: "page.margin.left",
        note: "",
    },
    LegacyKeyMap {
        old: "page.margins.right",
        new: "page.margin.right",
        note: "",
    },
    LegacyKeyMap {
        old: "typography.font",
        new: "fonts.body",
        note: "",
    },
    LegacyKeyMap {
        old: "typography.heading_font",
        new: "fonts.heading",
        note: "",
    },
    LegacyKeyMap {
        old: "typography.font_size",
        new: "text.size",
        note: "",
    },
    LegacyKeyMap {
        old: "typography.line_spacing",
        new: "text.leading",
        note: "",
    },
    LegacyKeyMap {
        old: "style.accent_color",
        new: "colors.accent",
        note: "",
    },
    LegacyKeyMap {
        old: "style.body_alignment",
        new: "text.align",
        note: "",
    },
    LegacyKeyMap {
        old: "style.header_alignment",
        new: "header.align",
        note: "",
    },
    LegacyKeyMap {
        old: "style.link_underline",
        new: "links.underline",
        note: "",
    },
    LegacyKeyMap {
        old: "style.highlight_bullet",
        new: "lists.bullet",
        note: "",
    },
    LegacyKeyMap {
        old: "spacing.list_item",
        new: "spacing.item",
        note: "",
    },
    LegacyKeyMap {
        old: "spacing.section",
        new: "spacing.section",
        note: "",
    },
    LegacyKeyMap {
        old: "spacing.section_heading",
        new: "headings.section.space_after",
        note: "",
    },
    LegacyKeyMap {
        old: "spacing.entry",
        new: "spacing.entry",
        note: "",
    },
    LegacyKeyMap {
        old: "pagination.allow_entry_page_break",
        new: "entry.keep_together",
        note: "inverted",
    },
];

/// Look up the mapping for a legacy settings key.
pub fn legacy_key(old: &str) -> Option<&'static LegacyKeyMap> {
    LEGACY_SETTINGS_MAP.iter().find(|entry| entry.old == old)
}

// ---------------------------------------------------------------------------
// Schema documents
// ---------------------------------------------------------------------------

/// Wrap a generated schema as a standalone JSON Schema document with a
/// stable `$id`, so editors can fetch and validate against it.
fn schema_document(
    title: &str,
    description: &str,
    id: &str,
    schema: schemars::Schema,
) -> serde_json::Value {
    use serde_json::Value;
    let mut root = serde_json::to_value(&schema).expect("schema serializes");
    let object = root.as_object_mut().expect("schema is an object");
    object.insert(
        "$schema".into(),
        Value::String("https://json-schema.org/draft/2020-12/schema".into()),
    );
    object.insert("$id".into(), Value::String(id.into()));
    object.insert("title".into(), Value::String(title.into()));
    object.insert("description".into(), Value::String(description.into()));
    root
}

/// The JSON Schema for the `[style]` table shared by theme.toml and settings.json.
pub fn style_schema_document() -> serde_json::Value {
    schema_document(
        "Code a CV style sheet",
        "One style vocabulary shared by defaults, theme.toml [style], and settings.json [style]. Every key is optional; layers merge over the built-in defaults.",
        "https://raw.githubusercontent.com/voxvanhieu/code-a-cv/main/crates/cac-render/schema/style.schema.json",
        schemars::schema_for!(StyleOverrides),
    )
}

/// The JSON Schema for a theme manifest (`theme.toml`).
pub fn theme_schema_document() -> serde_json::Value {
    schema_document(
        "Code a CV theme manifest",
        "A declarative theme: metadata, style overrides, family options, layout regions, and form requirements.",
        "https://raw.githubusercontent.com/voxvanhieu/code-a-cv/main/crates/cac-render/schema/theme.schema.json",
        schemars::schema_for!(ThemeManifest),
    )
}
