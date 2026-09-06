mod codec;
mod json_resume;
mod markdown;

pub use codec::{InputFormat, ParseError, parse, validate};
pub use json_resume::{export_json_resume, export_json_resume_checked, import_json_resume};

pub use markdown::{
    STARTER_MARKDOWN, parse_date_point, parse_markdown, slugify, to_markdown, to_markdown_checked,
};
