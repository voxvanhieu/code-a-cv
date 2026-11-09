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

pub use write::{MarkdownStyle, to_markdown, to_markdown_checked, to_markdown_with_style};

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
        // A named kind whose spelling names an entry kind defaults that
        // section's entries to it (`## Languages` → language entries).
        SectionKind::Named(value) => {
            for known in [
                "experience",
                "education",
                "project",
                "publication",
                "skill-group",
                "custom",
                "language",
                "reference",
            ] {
                if value.eq_ignore_ascii_case(known) {
                    return known;
                }
            }
            "custom"
        }
        SectionKind::Custom => "custom",
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
        "language" | "languages" => SectionKind::Named("language".into()),
        "reference" | "references" => SectionKind::Named("reference".into()),
        _ => SectionKind::Custom,
    }
}

/// What a Markdown parse learned about the source beyond the model, for
/// tools that rewrite the file: the spelling the source used, the
/// declarations front matter carried, and how many plain comments the model
/// dropped.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct MarkdownParseInfo {
    /// The spelling the source used: compact when sugar appeared (comma
    /// headings, bare date lines, `·` contacts), explicit otherwise.
    pub style: MarkdownStyle,
    /// The `locale:`/`packs:` declarations, ready to re-emit between `---`
    /// fences; `None` when the source declared none.
    pub declarations: Option<String>,
    /// Plain (non-directive) comments the model has no slot for. A
    /// formatter that would drop them must refuse instead.
    pub plain_comments: usize,
}

/// Parse and report what the source used, for `fmt` and checked writers.
pub fn parse_markdown_with_info(
    source: &str,
) -> Result<(CvDocument, MarkdownParseInfo), ParseError> {
    let blocks = ast::blocks(source)?;
    let mut blocks = blocks.into_iter().peekable();
    let mut profile_updates = Vec::new();
    let mut declarations = None;
    if matches!(
        blocks.peek().map(|block| &block.kind),
        Some(ast::BlockKind::FrontMatter(_))
    ) {
        let first = blocks.next().expect("peeked");
        let ast::BlockKind::FrontMatter(raw) = first.kind else {
            unreachable!("peeked front matter");
        };
        let (front, fields) = front_matter::parse(&raw, first.line)?;
        profile_updates = fields;
        let mut lines = Vec::new();
        if let Some(locale) = &front.locale {
            lines.push(format!("locale: {locale}"));
        }
        if !front.packs.is_empty() {
            lines.push("packs:".into());
            for pack in &front.packs {
                lines.push(format!("  - {pack}"));
            }
        }
        declarations = (!lines.is_empty()).then(|| lines.join("\n"));
    }
    let mut map_info = entries::MapInfo::default();
    let mapped = entries::map(blocks.collect(), &mut map_info)?;
    let mut cv = CvDocument {
        profile: mapped.profile,
        sections: mapped.sections,
    };
    for field in profile_updates {
        entries::apply_profile_field(&mut cv.profile, &field.key, &field.value, field.line)?;
    }
    crate::codec::normalize(&mut cv);
    crate::validate(&cv)?;
    let info = MarkdownParseInfo {
        style: if map_info.sugar {
            MarkdownStyle::Compact
        } else {
            MarkdownStyle::Explicit
        },
        declarations,
        plain_comments: map_info.plain_comments,
    };
    Ok((cv, info))
}

pub fn parse_markdown(source: &str) -> Result<CvDocument, ParseError> {
    parse_markdown_with_info(source).map(|(cv, _)| cv)
}
