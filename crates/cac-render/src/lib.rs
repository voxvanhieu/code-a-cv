mod html;
mod pdf;

use cac_core::DatePoint;

pub use html::render_html;
pub use pdf::{RenderError, RenderedPdf, render_pdf};

pub fn format_date(value: &DatePoint) -> String {
    value.to_string()
}
