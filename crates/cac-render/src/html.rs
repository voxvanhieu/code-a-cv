use std::fmt::Write;

use cac_core::{CvDocument, EntryKind, Inline, RichText};

pub fn render_html(cv: &CvDocument) -> String {
    let mut output = String::from(
        "<!doctype html><html lang=\"en\"><head><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width,initial-scale=1\"><title>",
    );
    output.push_str(&html_escape::encode_text(&format!(
        "{} CV",
        cv.profile.name
    )));
    output.push_str("</title><style>:root{font-family:system-ui,sans-serif;color:#171717}body{max-width:800px;margin:2rem auto;padding:0 1.5rem;line-height:1.4}header{text-align:center}h1{margin-bottom:.25rem}h2{font-size:1.15rem;border-bottom:1px solid;margin-top:1.4rem}.contact{display:flex;gap:.75rem;justify-content:center;flex-wrap:wrap}.entry{margin:.8rem 0}.entry-head{display:flex;justify-content:space-between;gap:1rem}.period{white-space:nowrap}ul{margin:.35rem 0;padding-left:1.3rem}@media print{body{margin:0;max-width:none}@page{size:A4;margin:15mm}}</style></head><body itemscope itemtype=\"https://schema.org/Person\"><header><h1 itemprop=\"name\">");
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
    output.push_str("</div></header>");
    if let Some(summary) = &cv.profile.summary {
        output.push_str("<p>");
        render_rich_html(summary, &mut output);
        output.push_str("</p>");
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
            if let Some(period) = entry.kind.period() {
                let _ = write!(
                    output,
                    "<time class=\"period\">{}–{}</time>",
                    period.start, period.end
                );
            }
            output.push_str("</div>");
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
