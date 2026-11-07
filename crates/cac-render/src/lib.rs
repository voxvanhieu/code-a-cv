mod html;
mod pdf;
mod settings;
mod shared;
pub mod style;
mod view;

use cac_core::DatePoint;

pub use html::render_html;
pub use pdf::{
    EMBEDDED_THEME_NAMES, RenderError, RenderOptions, RenderedPdf, ThemeSource,
    embedded_theme_source, render_pdf, render_pdf_and_preview_with_options,
    render_pdf_with_options,
};
pub use settings::{
    DEFAULT_THEME, PageMargins, PageSettings, PaginationSettings, Settings, SettingsError,
    SpacingSettings, TypographySettings, settings_schema, validate_theme_name,
};

pub use shared::{SharedThemeArtifact, SharedThemeError, test_shared_theme};
pub use style::{
    ColorSpec, HeaderAlign, Length, LengthUnit, ResolvedStyle, ShimmedSettings, StyleContractError,
    StyleOverrides, TextAlign, classic_family, defaults, resolve, resolve_layered, shim,
    style_schema_document, theme_schema_document,
};
pub use view::types::{
    InlineView, RichSlot, ViewContact, ViewDocument, ViewEntry, ViewProfile, ViewSection,
};

pub fn format_date(value: &DatePoint) -> String {
    const MONTHS: [&str; 12] = [
        "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
    ];

    match value {
        DatePoint::Year(year) => year.to_string(),
        DatePoint::YearMonth(year, month) => {
            let name = MONTHS
                .get(usize::from(*month).saturating_sub(1))
                .copied()
                .unwrap_or("???");
            format!("{name} {year}")
        }
        DatePoint::Full(date) => date.format("%b %-d, %Y").to_string(),
        DatePoint::Present => "Present".into(),
    }
}
