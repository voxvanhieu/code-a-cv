mod codec;
mod json_resume;
mod markdown;
pub mod packs;

pub use codec::{InputFormat, ParseError, parse, validate};
pub use json_resume::{export_json_resume, export_json_resume_checked, import_json_resume};

pub use markdown::{
    MarkdownParseInfo, MarkdownStyle, STARTER_MARKDOWN, parse_date_point, parse_markdown,
    parse_markdown_with_info, slugify, to_markdown, to_markdown_checked, to_markdown_with_style,
};
