//! Phase one of the FieldMark parser: CommonMark structure.
//!
//! The source walks a pulldown-cmark event stream with offsets, and only
//! block structure survives: headings, paragraphs (raw markdown, kept for
//! rich-text parsing), lists (inside their raw range), and YAML front
//! matter. Whitespace, list markers, indentation, and emphasis style never
//! reach phase two — a formatter that preserves the AST cannot change the
//! CV.
//!
//! HTML comments are stripped line-wise before the event walk (the way a
//! formatter or editor sees them: an open `<!--` swallows lines until a
//! closing `-->`), then merged back into the block stream in source order.

use super::markdown_error;
use crate::ParseError;
use pulldown_cmark::{Event, Options, Parser, Tag, TagEnd};

/// One structural block with its first source line number (1-based).
#[derive(Debug)]
pub(super) struct Block {
    pub kind: BlockKind,
    pub line: usize,
}

#[derive(Debug)]
pub(super) enum BlockKind {
    /// YAML front matter, raw text without the `---` fences.
    FrontMatter(String),
    /// An ATX heading with its level and its raw text after the markers.
    Heading { level: u32, text: String },
    /// A paragraph or list's raw markdown, soft breaks included as newlines.
    Paragraph(String),
    /// One HTML comment block, `<!--` through `-->`.
    Comment(String),
    /// A thematic break, treated as prose by the grammar.
    Rule,
}

/// Split the source into structural blocks.
pub(super) fn blocks(source: &str) -> Result<Vec<Block>, ParseError> {
    let (stripped, comments, terminated) = strip_comments(source);
    if !terminated {
        return markdown_error(source.lines().count(), "unterminated HTML comment; add -->");
    }
    // Link reference definitions produce no events and would vanish; the
    // grammar keeps links inline, so call the line out instead.
    for (index, line) in stripped.lines().enumerate() {
        if let Some((label, target)) = link_reference_definition(line.trim()) {
            let _ = target;
            return markdown_error(
                index + 1,
                &format!(
                    "link reference definition `[{}]` is not supported; write the link inline where it is used",
                    label
                ),
            );
        }
    }
    let mut options = Options::empty();
    options.insert(Options::ENABLE_YAML_STYLE_METADATA_BLOCKS);
    let mut blocks: Vec<Block> = Vec::new();
    // Container arms (blockquote, code block) emit one block from the
    // container's own end range; the inner paragraph events are suppressed.
    let mut suppressed = 0usize;
    let line_of = |offset: usize| {
        stripped[..offset]
            .bytes()
            .filter(|byte| *byte == b'\n')
            .count()
            + 1
    };
    for (event, range) in Parser::new_ext(&stripped, options).into_offset_iter() {
        match event {
            Event::Start(Tag::BlockQuote(_))
            | Event::Start(Tag::CodeBlock(_))
            | Event::Start(Tag::List(_)) => {
                suppressed += 1;
            }
            Event::End(TagEnd::MetadataBlock(_)) => {
                let raw = raw_between_metadata_fences(&stripped[range.clone()]);
                blocks.push(Block {
                    kind: BlockKind::FrontMatter(raw),
                    line: line_of(range.start),
                });
            }
            Event::End(TagEnd::Heading(level)) => {
                let raw = stripped[range.clone()].trim();
                let text = raw.trim_start_matches('#').trim().to_string();
                blocks.push(Block {
                    kind: BlockKind::Heading {
                        level: level as u32,
                        text,
                    },
                    line: line_of(range.start),
                });
            }
            Event::End(TagEnd::BlockQuote(_))
            | Event::End(TagEnd::CodeBlock)
            | Event::End(TagEnd::List(_)) => {
                suppressed = suppressed.saturating_sub(1);
                if suppressed > 0 {
                    // A nested container hands its content to the outermost
                    // block, whose raw range already includes it.
                    continue;
                }
                let raw = stripped[range.clone()].trim().to_string();
                // The container's end range can start after its marker line;
                // find the raw text itself for an accurate line number.
                let start = stripped.find(raw.as_str()).unwrap_or(range.start);
                blocks.push(Block {
                    kind: BlockKind::Paragraph(raw),
                    line: line_of(start),
                });
            }
            Event::End(TagEnd::Paragraph) => {
                if suppressed == 0 {
                    let raw = stripped[range.clone()].trim().to_string();
                    blocks.push(Block {
                        kind: BlockKind::Paragraph(raw),
                        line: line_of(range.start),
                    });
                }
            }
            Event::Rule => {
                blocks.push(Block {
                    kind: BlockKind::Rule,
                    line: line_of(range.start),
                });
            }
            Event::Html(html) | Event::InlineHtml(html) => {
                // Raw HTML that is not a comment is prose content; the rich
                // parser rejects it with a line-accurate error.
                blocks.push(Block {
                    kind: BlockKind::Paragraph(html.trim().to_string()),
                    line: line_of(range.start),
                });
            }
            _ => {}
        }
    }
    // Re-insert the comments where they appeared; the stable sort keeps a
    // comment on the same line as content after it.
    for (raw, line) in comments {
        blocks.push(Block {
            kind: BlockKind::Comment(raw),
            line,
        });
    }
    blocks.sort_by_key(|block| block.line);
    Ok(blocks)
}

/// Remove `<!-- ... -->` comment lines from the source, returning the
/// remaining text (blank where comments were, so line numbers hold), the
/// extracted comments with their first lines, and whether every comment
/// closed.
fn strip_comments(source: &str) -> (String, Vec<(String, usize)>, bool) {
    let mut stripped = String::with_capacity(source.len());
    let mut comments = Vec::new();
    let mut in_comment = false;
    let mut current = String::new();
    let mut start_line = 0usize;
    for (index, line) in source.lines().enumerate() {
        let number = index + 1;
        let trimmed = line.trim();
        if in_comment {
            current.push('\n');
            current.push_str(line);
            if trimmed.contains("-->") {
                comments.push((std::mem::take(&mut current), start_line));
                in_comment = false;
            }
            stripped.push('\n');
            continue;
        }
        if trimmed.starts_with("<!--") {
            in_comment = true;
            start_line = number;
            current = line.trim_end().to_string();
            if trimmed.contains("-->") {
                comments.push((std::mem::take(&mut current), start_line));
                in_comment = false;
            }
            stripped.push('\n');
            continue;
        }
        stripped.push_str(line);
        stripped.push('\n');
    }
    if in_comment {
        comments.push((current, start_line));
    }
    (stripped, comments, !in_comment)
}

/// Match a CommonMark link reference definition line: `[label]: target`.
fn link_reference_definition(line: &str) -> Option<(String, String)> {
    if !line.starts_with('[') {
        return None;
    }
    let close = line.find(']')?;
    if close <= 1 {
        return None;
    }
    let rest = line[close + 1..].trim_start();
    let target = rest.strip_prefix(':')?.trim();
    (target.is_empty() || target.split_whitespace().count() <= 2)
        .then(|| (line[1..close].to_string(), target.to_string()))
}

/// Strip the `---` fences a metadata block's raw range carries.
fn raw_between_metadata_fences(raw: &str) -> String {
    let body = raw.trim();
    let body = body.strip_prefix("---").unwrap_or(body);
    let body = body.strip_suffix("---").unwrap_or(body);
    body.trim_matches('\n').to_string()
}

#[cfg(test)]
mod quote_dump {
    #[test]
    fn dump() {
        let source = "# A\n\n## Notes\n\n> A quotation\n";
        let (stripped, _, _) = super::strip_comments(source);
        let mut options = super::Options::empty();
        options.insert(super::Options::ENABLE_YAML_STYLE_METADATA_BLOCKS);
        for (event, range) in super::Parser::new_ext(&stripped, options).into_offset_iter() {
            if let super::Event::End(super::TagEnd::BlockQuote(_)) = event {
                println!("range {range:?} raw {:?}", &stripped[range.clone()]);
            }
        }
    }
}
