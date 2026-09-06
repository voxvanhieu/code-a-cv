use std::fmt::{self, Display, Formatter};
use std::iter::Peekable;

use pulldown_cmark::{Event, Options, Parser, Tag, TagEnd};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use url::Url;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RichText(pub Vec<Inline>);

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Inline {
    Text(String),
    Emph(Vec<Inline>),
    Strong(Vec<Inline>),
    Code(String),
    Link {
        href: Url,
        body: Vec<Inline>,
    },
    Paragraph(Vec<Inline>),
    Break,
    List {
        start: Option<u64>,
        items: Vec<RichText>,
    },
}

#[derive(Clone, Debug, thiserror::Error)]
#[error("{message}")]
pub struct RichTextError {
    pub offset: usize,
    pub message: String,
}

impl RichText {
    pub fn parse(markdown: &str) -> Self {
        Self::try_parse(markdown).unwrap_or_else(|_| Self::literal(markdown))
    }

    pub fn literal(text: &str) -> Self {
        let mut nodes = Vec::new();
        let text = text.replace("\r\n", "\n");
        for (index, line) in text.trim_end_matches('\n').split('\n').enumerate() {
            if index > 0 {
                nodes.push(Inline::Break);
            }
            if !line.is_empty() {
                nodes.push(Inline::Text(line.to_owned()));
            }
        }
        Self(nodes)
    }

    pub fn try_parse(markdown: &str) -> Result<Self, RichTextError> {
        let parser = Parser::new_ext(markdown, Options::ENABLE_TABLES);
        if let Some((_, definition)) = parser
            .reference_definitions()
            .iter()
            .min_by_key(|(_, definition)| definition.span.start)
        {
            return Err(RichTextError { offset: definition.span.start, message: "reference link definitions are not supported; use [label](https://example.com) inline".into() });
        }
        let mut events = parser.into_offset_iter().peekable();
        Ok(Self(unwrap_paragraph(parse_nodes(&mut events, None)?)))
    }

    pub fn plain(&self) -> String {
        plain_inlines(&self.0)
    }

    pub fn is_empty(&self) -> bool {
        self.plain().trim().is_empty()
    }

    pub fn is_inline(&self) -> bool {
        fn inline(nodes: &[Inline]) -> bool {
            nodes.iter().all(|node| match node {
                Inline::Paragraph(_) | Inline::List { .. } | Inline::Break => false,
                Inline::Emph(body) | Inline::Strong(body) | Inline::Link { body, .. } => {
                    inline(body)
                }
                _ => true,
            })
        }
        inline(&self.0)
    }

    pub fn to_markdown(&self) -> String {
        fn render(nodes: &[Inline], output: &mut String) {
            for node in nodes {
                match node {
                    Inline::Text(value) => output.push_str(&escape_markdown(value)),
                    Inline::Emph(body) | Inline::Strong(body) => {
                        let delimiter = if matches!(node, Inline::Strong(_)) {
                            "**"
                        } else {
                            "*"
                        };
                        output.push_str(delimiter);
                        render(body, output);
                        output.push_str(delimiter);
                    }
                    Inline::Code(value) => {
                        let longest = value.split(|c| c != '`').map(str::len).max().unwrap_or(0);
                        let fence = "`".repeat(longest + 1);
                        let pad = value.starts_with('`')
                            || value.ends_with('`')
                            || (value.starts_with(' ')
                                && value.ends_with(' ')
                                && !value.trim().is_empty());
                        output.push_str(&fence);
                        if pad {
                            output.push(' ');
                        }
                        output.push_str(value);
                        if pad {
                            output.push(' ');
                        }
                        output.push_str(&fence);
                    }
                    Inline::Link { href, body } => {
                        output.push('[');
                        render(body, output);
                        output.push_str("](<");
                        output.push_str(&href.as_str().replace('<', "%3C").replace('>', "%3E"));
                        output.push_str(">)");
                    }
                    Inline::Paragraph(body) => {
                        separate_block(output);
                        render(body, output);
                        output.push_str("\n\n");
                    }
                    Inline::Break => output.push_str("\\\n"),
                    Inline::List { start, items } => {
                        separate_block(output);
                        for (index, item) in items.iter().enumerate() {
                            let marker = start.map_or_else(
                                || "- ".to_owned(),
                                |start| format!("{}. ", start.saturating_add(index as u64)),
                            );
                            let indent = " ".repeat(marker.len());
                            let body = item.to_markdown();
                            for (line, text) in body.lines().enumerate() {
                                output.push_str(if line == 0 { &marker } else { &indent });
                                output.push_str(text);
                                output.push('\n');
                            }
                        }
                        output.push('\n');
                    }
                }
            }
        }
        let mut output = String::new();
        render(&self.0, &mut output);
        output.trim_end_matches('\n').to_owned()
    }
}

pub fn escape_markdown(value: &str) -> String {
    let mut output = String::new();
    let mut characters = value.char_indices().peekable();
    while let Some((index, character)) = characters.next() {
        let prefix = value[..index].rsplit('\n').next().unwrap_or("");
        let whitespace_follows = characters
            .peek()
            .is_some_and(|(_, next)| next.is_whitespace());
        let list_marker = whitespace_follows
            && (("+-".contains(character) && prefix.is_empty())
                || (".)".contains(character)
                    && !prefix.is_empty()
                    && prefix.chars().all(|c| c.is_ascii_digit())));
        if "\\`*_[]<>&#!|".contains(character)
            || (character == '-' && prefix.is_empty())
            || list_marker
        {
            output.push('\\');
        }
        output.push(character);
    }
    output
}

fn separate_block(output: &mut String) {
    if !output.is_empty() && !output.ends_with("\n\n") {
        if !output.ends_with('\n') {
            output.push('\n');
        }
        output.push('\n');
    }
}

fn unwrap_paragraph(mut nodes: Vec<Inline>) -> Vec<Inline> {
    if nodes
        .iter()
        .any(|node| matches!(node, Inline::Paragraph(_) | Inline::List { .. }))
    {
        let mut normalized = Vec::new();
        let mut run = Vec::new();
        for node in nodes {
            if matches!(node, Inline::Paragraph(_) | Inline::List { .. }) {
                if !run.is_empty() {
                    normalized.push(Inline::Paragraph(std::mem::take(&mut run)));
                }
                normalized.push(node);
            } else {
                run.push(node);
            }
        }
        if !run.is_empty() {
            normalized.push(Inline::Paragraph(run));
        }
        nodes = normalized;
    }
    if nodes.len() == 1 && matches!(nodes[0], Inline::Paragraph(_)) {
        let Inline::Paragraph(body) = nodes.remove(0) else {
            unreachable!()
        };
        body
    } else {
        nodes
    }
}

fn push(nodes: &mut Vec<Inline>, node: Inline) {
    if let Inline::Text(text) = &node {
        if text.is_empty() {
            return;
        }
        if let Some(Inline::Text(previous)) = nodes.last_mut() {
            previous.push_str(text);
            return;
        }
    }
    match (nodes.last_mut(), &node) {
        (Some(Inline::Emph(previous)), Inline::Emph(body))
        | (Some(Inline::Strong(previous)), Inline::Strong(body)) => {
            for node in body {
                push(previous, node.clone());
            }
            return;
        }
        (Some(Inline::Code(previous)), Inline::Code(text)) => {
            previous.push_str(text);
            return;
        }
        _ => {}
    }
    nodes.push(node);
}

type Events<'a> = Peekable<pulldown_cmark::OffsetIter<'a>>;

fn parse_nodes(events: &mut Events<'_>, end: Option<TagEnd>) -> Result<Vec<Inline>, RichTextError> {
    let mut nodes = Vec::new();
    while let Some((event, range)) = events.next() {
        let error = |message: &str| RichTextError {
            offset: range.start,
            message: message.into(),
        };
        let node = match event {
            Event::End(tag) if Some(tag) == end => break,
            Event::Text(value) => Inline::Text(value.into_string()),
            Event::Code(value) => Inline::Code(value.into_string()),
            Event::SoftBreak => Inline::Text(" ".into()),
            Event::HardBreak => Inline::Break,
            Event::Start(Tag::Paragraph) => {
                Inline::Paragraph(parse_nodes(events, Some(TagEnd::Paragraph))?)
            }
            Event::Start(Tag::Emphasis) => {
                Inline::Emph(parse_nodes(events, Some(TagEnd::Emphasis))?)
            }
            Event::Start(Tag::Strong) => Inline::Strong(parse_nodes(events, Some(TagEnd::Strong))?),
            Event::Start(Tag::Link {
                dest_url, title, ..
            }) => {
                if !title.is_empty() {
                    return Err(error(
                        "link tooltips are not supported; put that information in the visible link label",
                    ));
                }
                let href = Url::parse(&dest_url).map_err(|_| {
                    error(
                        "invalid link target; use an absolute https:, http:, mailto:, or tel: URL",
                    )
                })?;
                if !supported_link(&href) {
                    return Err(error(
                        "unsupported link target; use https:, http:, mailto:, or tel:",
                    ));
                }
                Inline::Link {
                    href,
                    body: parse_nodes(events, Some(TagEnd::Link))?,
                }
            }
            Event::Start(Tag::List(start)) => {
                let mut items = Vec::new();
                while let Some((event, _)) = events.peek() {
                    match event {
                        Event::End(TagEnd::List(_)) => {
                            events.next();
                            break;
                        }
                        Event::Start(Tag::Item) => {
                            events.next();
                            let item = RichText(unwrap_paragraph(parse_nodes(
                                events,
                                Some(TagEnd::Item),
                            )?));
                            if item.is_empty() {
                                return Err(error("list items must not be empty"));
                            }
                            items.push(item);
                        }
                        _ => return Err(error("invalid list structure")),
                    }
                }
                Inline::List { start, items }
            }
            Event::Html(value) | Event::InlineHtml(value)
                if value.trim().starts_with("<!--") && value.trim().ends_with("-->") =>
            {
                continue;
            }
            Event::Start(Tag::Image { .. }) => {
                return Err(error(
                    "images are not supported; use a labeled link instead",
                ));
            }
            Event::Start(Tag::Table(_)) => {
                return Err(error(
                    "Markdown tables are not supported; use entries or lists",
                ));
            }
            Event::Start(Tag::CodeBlock(_)) => {
                return Err(error(
                    "code blocks are not supported; use inline code or ordinary prose",
                ));
            }
            Event::Html(_) | Event::InlineHtml(_) => {
                return Err(error(
                    "raw HTML is not supported; escape literal angle brackets or use Markdown",
                ));
            }
            _ => {
                return Err(error(
                    "unsupported Markdown structure; use paragraphs, lists, and inline formatting",
                ));
            }
        };
        push(&mut nodes, node);
    }
    Ok(nodes)
}

pub fn supported_link(url: &Url) -> bool {
    matches!(url.scheme(), "https" | "http" | "mailto" | "tel")
}

impl From<&str> for RichText {
    fn from(value: &str) -> Self {
        Self::parse(value)
    }
}

impl Display for RichText {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.plain())
    }
}

impl Serialize for RichText {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_markdown())
    }
}

impl<'de> Deserialize<'de> for RichText {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Self::try_parse(&String::deserialize(deserializer)?).map_err(serde::de::Error::custom)
    }
}

fn plain_inlines(nodes: &[Inline]) -> String {
    let mut output = String::new();
    for node in nodes {
        match node {
            Inline::Text(value) | Inline::Code(value) => output.push_str(value),
            Inline::Emph(body) | Inline::Strong(body) | Inline::Link { body, .. } => {
                output.push_str(&plain_inlines(body))
            }
            Inline::Break => output.push('\n'),
            Inline::Paragraph(body) => {
                separate_block(&mut output);
                output.push_str(&plain_inlines(body));
                output.push_str("\n\n");
            }
            Inline::List { items, .. } => {
                separate_block(&mut output);
                for item in items {
                    output.push_str(&item.plain());
                    output.push('\n');
                }
            }
        }
    }
    output.trim_end_matches('\n').to_owned()
}
