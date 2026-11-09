pub mod diagnostics;
mod html;
mod pdf;
mod settings;
mod shared;
pub mod style;
mod view;

pub use html::{render_html, render_html_for_sheet, render_html_with};
pub use pdf::{BASE_TYP_SOURCE, LAYOUT_TYP_SOURCE, MAIN_TYP_SOURCE, STYLE_TYP_SOURCE};
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
    ColorSpec, HeaderAlign, HeadingTransform, Length, LengthUnit, ResolvedStyle, RuleStyle,
    ShimmedSettings, StyleContractError, StyleOverrides, TextAlign, Weight, classic_family,
    defaults, resolve, resolve_layered, resolved_style_for, shim, style_schema_document,
    theme_schema_document,
};
pub use view::dates::{format_date_locale, format_event_locale, format_period_locale};
#[doc(hidden)]
pub use view::test_render_view;
pub use view::types::{
    InlineView, RichSlot, ViewContact, ViewDocument, ViewEntry, ViewProfile, ViewSection,
};

// The English-only `format_date` is gone: `view::dates` is the one
// formatter, and both renderers print its strings.
