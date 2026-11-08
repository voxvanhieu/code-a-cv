mod codec;
mod json_resume;
mod markdown;
pub mod packs;

pub use codec::{InputFormat, ParseError, parse, validate};
pub use json_resume::{export_json_resume, export_json_resume_checked, import_json_resume};

pub use markdown::{
    MarkdownStyle, STARTER_MARKDOWN, detect_style, parse_date_point, parse_markdown, slugify,
    to_markdown, to_markdown_checked, to_markdown_with_style,
};
