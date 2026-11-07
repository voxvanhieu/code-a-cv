//! The FieldMark Markdown parser: two phases on the CommonMark AST.
//!
//! Phase one (`ast.rs`) reduces the source to block structure — headings,
//! paragraphs, lists, comments, front matter. Phase two (`entries.rs`)
//! maps that structure onto the document model under the rules in
//! `docs/authoring/markdown.md`. Whitespace between blocks never carries
//! meaning, so a formatter that preserves the AST cannot change the CV.
//! The writer lives in `write.rs`.

pub mod ast;
mod entries;
mod fields;
mod front_matter;
mod write;

use cac_core::{CvDocument, SectionKind};

use crate::ParseError;
pub use fields::{parse_date_point, slugify};
pub(crate) use write::difference;

pub use write::{
    MarkdownStyle, detect_style, to_markdown, to_markdown_checked, to_markdown_with_style,
};

pub const STARTER_MARKDOWN: &str = include_str!("../starter.md");

fn markdown_error<T>(line: usize, message: &str) -> Result<T, ParseError> {
    Err(ParseError::Markdown {
        line,
        message: message.into(),
    })
}

fn rich(value: &str, line: usize) -> Result<cac_core::RichText, ParseError> {
    cac_core::RichText::try_parse(value).map_err(|error| {
        let prefix = &value[..error.offset.min(value.len())];
        let offset = prefix.bytes().filter(|byte| *byte == b'\n').count();
        let column = prefix.rsplit('\n').next().unwrap_or("").chars().count() + 1;
        ParseError::Markdown {
            line: line + offset,
            message: format!("column {column}: {}", error.message),
        }
    })
}

fn inline(value: &str, line: usize) -> Result<cac_core::RichText, ParseError> {
    let value = rich(value.trim(), line)?;
    if !value.is_empty() && !value.is_inline() {
        return markdown_error(
            line,
            "this field accepts inline formatting, not paragraphs or lists",
        );
    }
    Ok(value)
}

fn heading(value: &str, line: usize) -> Result<String, ParseError> {
    let value = inline(value.trim(), line)?;
    if value.is_empty() {
        return markdown_error(line, "heading must contain visible text");
    }
    Ok(value.plain())
}

fn default_entry_kind(kind: &SectionKind) -> &'static str {
    match kind {
        SectionKind::Experience => "experience",
        SectionKind::Education => "education",
        SectionKind::Projects => "project",
        SectionKind::Publications => "publication",
        SectionKind::Skills => "skill-group",
        SectionKind::Custom | SectionKind::Named(_) => "custom",
    }
}

fn section_kind(title: &str) -> SectionKind {
    match title.to_ascii_lowercase().as_str() {
        "experience"
        | "work experience"
        | "professional experience"
        | "employment"
        | "employment history" => SectionKind::Experience,
        "education" => SectionKind::Education,
        "project" | "projects" => SectionKind::Projects,
        "publication" | "publications" => SectionKind::Publications,
        "skill" | "skills" | "technical skills" => SectionKind::Skills,
        _ => SectionKind::Custom,
    }
}

pub fn parse_markdown(source: &str) -> Result<CvDocument, ParseError> {
    let blocks = ast::blocks(source)?;
    let mut blocks = blocks.into_iter().peekable();
    let mut profile_updates = Vec::new();
    if matches!(
        blocks.peek().map(|block| &block.kind),
        Some(ast::BlockKind::FrontMatter(_))
    ) {
        let first = blocks.next().expect("peeked");
        let ast::BlockKind::FrontMatter(raw) = first.kind else {
            unreachable!("peeked front matter");
        };
        let (_, fields) = front_matter::parse(&raw, first.line)?;
        profile_updates = fields;
    }
    let mapped = entries::map(blocks.collect())?;
    let mut cv = CvDocument {
        profile: mapped.profile,
        sections: mapped.sections,
    };
    for field in profile_updates {
        entries::apply_profile_field(&mut cv.profile, &field.key, &field.value, field.line)?;
    }
    crate::codec::normalize(&mut cv);
    crate::validate(&cv)?;
    Ok(cv)
}
