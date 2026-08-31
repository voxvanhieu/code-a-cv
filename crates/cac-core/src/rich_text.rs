use std::fmt::{self, Display, Formatter};

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
    Link { href: Url, body: Vec<Inline> },
}

impl RichText {
    pub fn parse(markdown: &str) -> Self {
        let mut roots = Vec::new();
        let mut stack: Vec<(InlineKind, Vec<Inline>)> = Vec::new();
        for event in Parser::new_ext(markdown, Options::empty()) {
            match event {
                Event::Start(Tag::Emphasis) => stack.push((InlineKind::Emph, Vec::new())),
                Event::Start(Tag::Strong) => stack.push((InlineKind::Strong, Vec::new())),
                Event::Start(Tag::Link { dest_url, .. }) => {
                    stack.push((InlineKind::Link(dest_url.into_string()), Vec::new()))
                }
                Event::End(TagEnd::Emphasis | TagEnd::Strong | TagEnd::Link) => {
                    if let Some((kind, body)) = stack.pop() {
                        let node = match kind {
                            InlineKind::Emph => Inline::Emph(body),
                            InlineKind::Strong => Inline::Strong(body),
                            InlineKind::Link(href) => Url::parse(&href)
                                .map(|href| Inline::Link {
                                    href,
                                    body: body.clone(),
                                })
                                .unwrap_or_else(|_| Inline::Text(plain_inlines(&body))),
                        };
                        push_inline(&mut roots, &mut stack, node);
                    }
                }
                Event::Text(value) => {
                    push_inline(&mut roots, &mut stack, Inline::Text(value.into_string()))
                }
                Event::Code(value) => {
                    push_inline(&mut roots, &mut stack, Inline::Code(value.into_string()))
                }
                Event::SoftBreak | Event::HardBreak => {
                    push_inline(&mut roots, &mut stack, Inline::Text(" ".into()))
                }
                _ => {}
            }
        }
        Self(roots)
    }

    pub fn plain(&self) -> String {
        plain_inlines(&self.0)
    }

    pub fn is_empty(&self) -> bool {
        self.plain().trim().is_empty()
    }

    pub fn to_markdown(&self) -> String {
        fn render(nodes: &[Inline], output: &mut String) {
            for node in nodes {
                match node {
                    Inline::Text(value) => output.push_str(value),
                    Inline::Emph(body) => {
                        output.push('*');
                        render(body, output);
                        output.push('*');
                    }
                    Inline::Strong(body) => {
                        output.push_str("**");
                        render(body, output);
                        output.push_str("**");
                    }
                    Inline::Code(value) => {
                        output.push('`');
                        output.push_str(value);
                        output.push('`');
                    }
                    Inline::Link { href, body } => {
                        output.push('[');
                        render(body, output);
                        output.push_str("](");
                        output.push_str(href.as_str());
                        output.push(')');
                    }
                }
            }
        }
        let mut output = String::new();
        render(&self.0, &mut output);
        output
    }
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
        String::deserialize(deserializer).map(|value| Self::parse(&value))
    }
}

enum InlineKind {
    Emph,
    Strong,
    Link(String),
}

fn push_inline(roots: &mut Vec<Inline>, stack: &mut [(InlineKind, Vec<Inline>)], node: Inline) {
    if let Some((_, body)) = stack.last_mut() {
        body.push(node);
    } else {
        roots.push(node);
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
        }
    }
    output
}
