use std::fmt::Write;

use cac_core::{CvDocument, DatePoint, EntryKind, Inline, Period, RichText};

use crate::style::ResolvedStyle;
use crate::view::dates::format_event_locale;

pub fn render_html(cv: &CvDocument) -> String {
    render_html_for_sheet(cv, &crate::defaults())
}

/// Render the HTML document from the resolved style sheet: CSS custom
/// properties carry the sheet's values and dates print through the shared
/// locale-aware formatter, so the HTML output matches the PDF's.
pub fn render_html_for_sheet(cv: &CvDocument, style: &ResolvedStyle) -> String {
    let locale = style.locale.clone();
    render_html_with(cv, style, move |period, date| {
        format_event_locale(&locale, period, date)
    })
}

/// Render with an explicit date formatter; the generic form the shared
/// formatter plugs into.
pub fn render_html_with(
    cv: &CvDocument,
    style: &ResolvedStyle,
    format_dates: impl Fn(Option<&Period>, Option<&DatePoint>) -> Option<String>,
) -> String {
    // The document language follows the sheet's locale (primary subtag).
    let language = style.locale.split(['-', '_']).next().unwrap_or("en");
    let mut output = format!(
        "<!doctype html><html lang=\"{language}\"><head><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width,initial-scale=1\"><title>",
    );
    output.push_str(&html_escape::encode_text(&format!(
        "{} CV",
        cv.profile.name
    )));
    output.push_str("</title><style>");
    output.push_str(&css_variables(style));
    output.push_str("body{font-family:var(--font-body);color:var(--color-text);font-size:var(--text-size);line-height:var(--text-leading);background:#fff}p{margin:0 0 var(--text-paragraph-gap) 0;text-align:var(--text-align)}header{text-align:var(--header-align)}h1{margin-bottom:var(--heading-name-space-after);font-size:var(--heading-name-size,var(--text-size));font-weight:var(--heading-name-weight);color:var(--heading-name-color)}h2{font-size:var(--heading-section-size,var(--text-size));font-weight:var(--heading-section-weight);color:var(--heading-section-color);text-transform:var(--heading-section-transform);border-bottom:var(--heading-section-rule-width) var(--heading-section-rule-style) var(--color-rule);margin-top:var(--spacing-section);margin-bottom:var(--spacing-heading-gap);padding-bottom:.15rem}h3,.entry-title{font-size:var(--heading-entry-size,var(--text-size));font-weight:var(--heading-entry-weight);color:var(--heading-entry-color)}h1,h2,h3{font-family:var(--font-heading)}.contact{display:flex;gap:.75rem;justify-content:var(--header-align);flex-wrap:wrap}a{color:inherit;text-decoration:var(--link-underline)}article.entry{margin:0 0 var(--spacing-entry) 0}.entry-head{display:flex;justify-content:space-between;gap:1rem}.period{white-space:nowrap}.field-label{color:var(--color-muted);margin-right:.35rem}ul{margin:.35rem 0;padding-left:1.3rem}::marker{color:var(--color-accent)}@media print{body{margin:0;max-width:none}}");
    output.push_str("</style></head><body itemscope itemtype=\"https://schema.org/Person\"><header><h1 itemprop=\"name\">");
    output.push_str(&html_escape::encode_text(&cv.profile.name));
    output.push_str("</h1><div class=\"contact\">");
    if let Some(email) = &cv.profile.email {
        let _ = write!(
            output,
            "<a itemprop=\"email\" href=\"mailto:{}\">{}</a>",
            html_escape::encode_double_quoted_attribute(email),
            html_escape::encode_text(email)
        );
    }
    if let Some(phone) = &cv.profile.phone {
        let _ = write!(
            output,
            "<span itemprop=\"telephone\">{}</span>",
            html_escape::encode_text(phone)
        );
    }
    if let Some(location) = &cv.profile.location {
        let _ = write!(
            output,
            "<span itemprop=\"address\">{}</span>",
            html_escape::encode_text(location)
        );
    }
    if let Some(website) = &cv.profile.website {
        let escaped = html_escape::encode_double_quoted_attribute(website.as_str());
        let _ = write!(
            output,
            "<a itemprop=\"url\" href=\"{escaped}\">{}</a>",
            html_escape::encode_text(website.as_str())
        );
    }
    for contact in &cv.profile.contacts {
        if let Some(href) = &contact.href {
            let _ = write!(
                output,
                "<a href=\"{}\">{}</a>",
                html_escape::encode_double_quoted_attribute(href.as_str()),
                html_escape::encode_text(&contact.label)
            );
        } else {
            let _ = write!(
                output,
                "<span>{}</span>",
                html_escape::encode_text(&contact.label)
            );
        }
    }
    for field in cv.profile.fields.iter() {
        render_field(
            &capitalize_label(&field.key),
            &field.value.as_text(),
            &mut output,
        );
    }
    output.push_str("</div></header>");
    if let Some(summary) = &cv.profile.summary {
        output.push_str("<div class=\"summary\">");
        render_rich_html(summary, &mut output);
        output.push_str("</div>");
    }
    for section in &cv.sections {
        let _ = write!(
            output,
            "<section id=\"{}\"><h2>{}</h2>",
            html_escape::encode_double_quoted_attribute(&section.id),
            html_escape::encode_text(&section.title)
        );
        let mut entries = section.entries.iter().peekable();
        while let Some(entry) = entries.next() {
            if let EntryKind::Prose(value) = &entry.kind {
                output.push_str("<div class=\"prose\">");
                render_rich_html(&value.body, &mut output);
                output.push_str("</div>");
                continue;
            }
            if let EntryKind::Text(value) = &entry.kind {
                output.push_str("<ul class=\"text-entries\"><li>");
                render_rich_html(&value.body, &mut output);
                output.push_str("</li>");
                while let Some(next) =
                    entries.next_if(|next| matches!(next.kind, EntryKind::Text(_)))
                {
                    let EntryKind::Text(value) = &next.kind else {
                        unreachable!("the iterator predicate accepts only text entries");
                    };
                    output.push_str("<li>");
                    render_rich_html(&value.body, &mut output);
                    output.push_str("</li>");
                }
                output.push_str("</ul>");
                continue;
            }
            let (primary, secondary) = entry.kind.heading();
            output.push_str("<article class=\"entry\"><div class=\"entry-head\"><div><strong>");
            render_rich_html(primary, &mut output);
            if let Some(value) = secondary.filter(|value| !value.is_empty()) {
                output.push_str(", </strong>");
                render_rich_html(value, &mut output);
            } else {
                output.push_str("</strong>");
            }
            output.push_str("</div>");
            if let Some(rendered) = format_dates(entry.kind.period(), entry.kind.date()) {
                let _ = write!(output, "<time class=\"period\">{rendered}</time>");
            }
            output.push_str("</div>");
            if let EntryKind::Experience(value) = &entry.kind
                && let Some(location) = &value.location
            {
                let _ = write!(
                    output,
                    "<div class=\"location\">{}</div>",
                    html_escape::encode_text(location)
                );
            }
            let url = match &entry.kind {
                EntryKind::Project(value) => value.url.as_ref(),
                EntryKind::Publication(value) => value.url.as_ref(),
                _ => None,
            };
            if let Some(url) = url {
                let _ = write!(
                    output,
                    "<div><a href=\"{}\">{}</a></div>",
                    html_escape::encode_double_quoted_attribute(url.as_str()),
                    html_escape::encode_text(url.as_str())
                );
            }
            for field in entry.fields.iter() {
                render_field(
                    &capitalize_label(&field.key),
                    &field.value.as_text(),
                    &mut output,
                );
            }
            if let Some(body) = &entry.content {
                output.push_str("<div class=\"content\">");
                render_rich_html(body, &mut output);
                output.push_str("</div>");
            }
            if !entry.kind.highlights().is_empty() {
                output.push_str("<ul>");
                for highlight in entry.kind.highlights() {
                    output.push_str("<li>");
                    render_rich_html(highlight, &mut output);
                    output.push_str("</li>");
                }
                output.push_str("</ul>");
            }
            output.push_str("</article>");
        }
        output.push_str("</section>");
    }
    output.push_str("</body></html>");
    output
}

fn render_rich_html(value: &RichText, output: &mut String) {
    fn render(nodes: &[Inline], output: &mut String) {
        for node in nodes {
            match node {
                Inline::Paragraph(body) => {
                    output.push_str("<p>");
                    render(body, output);
                    output.push_str("</p>");
                }
                Inline::Break => output.push_str("<br>"),
                Inline::List { start, items } => {
                    if let Some(start) = start {
                        let _ = write!(output, "<ol start=\"{start}\">");
                    } else {
                        output.push_str("<ul>");
                    }
                    for item in items {
                        output.push_str("<li>");
                        render(&item.0, output);
                        output.push_str("</li>");
                    }
                    output.push_str(if start.is_some() { "</ol>" } else { "</ul>" });
                }
                Inline::Text(value) => output.push_str(&html_escape::encode_text(value)),
                Inline::Code(value) => {
                    output.push_str("<code>");
                    output.push_str(&html_escape::encode_text(value));
                    output.push_str("</code>");
                }
                Inline::Emph(body) => {
                    output.push_str("<em>");
                    render(body, output);
                    output.push_str("</em>");
                }
                Inline::Strong(body) => {
                    output.push_str("<strong>");
                    render(body, output);
                    output.push_str("</strong>");
                }
                Inline::Link { href, body } => {
                    let _ = write!(
                        output,
                        "<a href=\"{}\">",
                        html_escape::encode_double_quoted_attribute(href.as_str())
                    );
                    render(body, output);
                    output.push_str("</a>");
                }
            }
        }
    }
    render(&value.0, output);
}

/// A string value inside a CSS `"..."`: every character that could close
/// the string or the surrounding `<style>` element becomes a CSS escape, so
/// a font name from settings or a theme can never inject CSS or HTML.
fn css_string(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len());
    for character in value.chars() {
        match character {
            '\\' => escaped.push_str("\\5c "),
            '"' => escaped.push_str("\\22 "),
            '<' => escaped.push_str("\\3c "),
            '>' => escaped.push_str("\\3e "),
            ';' => escaped.push_str("\\3b "),
            '{' => escaped.push_str("\\7b "),
            '}' => escaped.push_str("\\7d "),
            c if c.is_control() => escaped.push_str(&format!("\\{:x} ", c as u32)),
            c => escaped.push(c),
        }
    }
    escaped
}

/// One open field as HTML: both the label and the value are escaped here so
/// the profile and entry loops cannot drift apart on escaping.
fn render_field(label: &str, value: &str, output: &mut String) {
    let _ = write!(
        output,
        "<div class=\"field\"><span class=\"field-label\">{}</span> {}</div>",
        html_escape::encode_text(label),
        html_escape::encode_text(value)
    );
}

/// Capitalize a normalized field key into a display label.
fn capitalize_label(key: &str) -> String {
    let mut label = String::new();
    for (index, part) in key.split('-').enumerate() {
        if index > 0 {
            label.push(' ');
        }
        let mut characters = part.chars();
        if let Some(first) = characters.next() {
            label.extend(first.to_uppercase());
            label.push_str(characters.as_str());
        }
    }
    label
}

fn css_variables(style: &ResolvedStyle) -> String {
    let mut css = String::from(":root{");
    let _ = write!(
        css,
        "--font-body:\"{}\";--font-heading:\"{}\";--font-mono:\"{}\";",
        css_string(&style.fonts.body),
        css_string(&style.fonts.heading),
        css_string(&style.fonts.mono)
    );
    let _ = write!(
        css,
        "--color-text:{};--color-muted:{};--color-accent:{};--color-rule:{};",
        style.colors.text, style.colors.muted, style.colors.accent, style.colors.rule
    );
    let _ = write!(
        css,
        "--text-size:{};--text-leading:{};--text-paragraph-gap:{};--text-align:{};",
        style.text.size,
        style.text.leading,
        style.text.paragraph_gap,
        css_align_text(style.text.align)
    );
    let _ = write!(css, "--header-align:{};", css_align(style.header.align));
    let _ = write!(
        css,
        "--heading-name-size:{};--heading-name-weight:{};--heading-name-color:{};--heading-name-space-after:{};",
        heading_size(&style.headings.name.size, &style.text.size),
        css_weight(style.headings.name.weight),
        style.headings.name.color,
        style.headings.name.space_after
    );
    let _ = write!(
        css,
        "--heading-section-size:{};--heading-section-weight:{};--heading-section-color:{};--heading-section-transform:{};--heading-section-rule-width:{};--heading-section-rule-style:{};",
        heading_size(&style.headings.section.size, &style.text.size),
        css_weight(style.headings.section.weight),
        style.headings.section.color,
        css_transform(style.headings.section.transform),
        rule_width(style.headings.section.rule),
        rule_style(style.headings.section.rule)
    );
    let _ = write!(
        css,
        "--heading-entry-size:{};--heading-entry-weight:{};--heading-entry-color:{};",
        heading_size(&style.headings.entry.size, &style.text.size),
        css_weight(style.headings.entry.weight),
        style.headings.entry.color
    );
    let _ = write!(
        css,
        "--spacing-section:{};--spacing-heading-gap:{};--spacing-entry:{};",
        style.spacing.section, style.spacing.heading_gap, style.spacing.entry
    );
    let _ = write!(
        css,
        "--link-underline:{};",
        if style.links.underline {
            "underline"
        } else {
            "none"
        }
    );
    let _ = write!(
        css,
        "--page-paper:{};--page-margin-top:{};--page-margin-right:{};--page-margin-bottom:{};--page-margin-left:{};",
        style.page.paper,
        style.page.margin.top,
        style.page.margin.right,
        style.page.margin.bottom,
        style.page.margin.left
    );
    // @page cannot resolve custom properties and needs a CSS page size,
    // so the paper and margins print as literals.
    let _ = write!(
        css,
        "@page{{size:{};margin:{} {} {} {}}}",
        css_paper(style.page.paper),
        style.page.margin.top,
        style.page.margin.right,
        style.page.margin.bottom,
        style.page.margin.left,
    );
    css.push_str("}\n");
    css
}

/// The CSS page-size spelling of the paper enum.
fn css_paper(paper: crate::style::schema::Paper) -> &'static str {
    match paper {
        crate::style::schema::Paper::A4 => "A4",
        crate::style::schema::Paper::UsLetter => "letter",
    }
}

fn heading_size(size: &Option<crate::Length>, text: &crate::Length) -> String {
    match size {
        Some(size) => size.to_string(),
        None => text.to_string(),
    }
}

fn css_weight(weight: crate::Weight) -> &'static str {
    match weight {
        crate::Weight::Light => "300",
        crate::Weight::Regular => "400",
        crate::Weight::Medium => "500",
        crate::Weight::SemiBold => "600",
        crate::Weight::Bold => "700",
        crate::Weight::Black => "900",
    }
}

fn css_align(align: crate::HeaderAlign) -> &'static str {
    match align {
        crate::HeaderAlign::Left => "left",
        crate::HeaderAlign::Center => "center",
        crate::HeaderAlign::Right => "right",
    }
}

fn css_align_text(align: crate::TextAlign) -> &'static str {
    match align {
        crate::TextAlign::Left => "left",
        crate::TextAlign::Justified => "justify",
    }
}

fn css_transform(transform: crate::HeadingTransform) -> &'static str {
    match transform {
        crate::HeadingTransform::None => "none",
        crate::HeadingTransform::Caps => "uppercase",
    }
}

fn rule_width(rule: crate::RuleStyle) -> &'static str {
    match rule {
        crate::RuleStyle::None => "0",
        crate::RuleStyle::Line => "1px",
    }
}

fn rule_style(rule: crate::RuleStyle) -> &'static str {
    match rule {
        crate::RuleStyle::None => "none",
        crate::RuleStyle::Line => "solid",
    }
}
