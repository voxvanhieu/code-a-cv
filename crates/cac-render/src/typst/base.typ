#let api_version = 1

#let default-tokens = (
  fonts: (
    body: "Libertinus Serif",
    heading: "Libertinus Serif",
    mono: "DejaVu Sans Mono",
  ),
  colors: (
    text: black,
    muted: gray,
    accent: black,
  ),
)

#let default-styles = (
  body: (
    font: "body",
    color: "text",
    font_size: 10pt,
    weight: "regular",
    line_spacing: 0.65em,
    paragraph_spacing: 1.2em,
    justify: true,
  ),
  heading: (
    font: "heading",
    color: "text",
    weight: "bold",
    space_before: 0pt,
    space_after: 0.65em,
  ),
  heading_1: (space_after: 1.2em),
  heading_2: (:),
  heading_3: (:),
  heading_4: (:),
  heading_5: (:),
  list: (item_spacing: auto),
  section: (
    space_before: 1.2em,
    space_after_heading: 0.65em,
  ),
  entry: (space_after: 1.2em),
  footer: (color: "muted"),
)

#let default-page = (
  paper: "a4",
  margin: 18mm,
)

#let merge(parent, child) = {
  let result = parent
  for (key, value) in child {
    result.insert(key, if key in result and type(result.at(key)) == dictionary and type(value) == dictionary {
      merge(result.at(key), value)
    } else {
      value
    })
  }
  result
}

#let rich(ctx, nodes) = {
  for node in nodes {
    if node.kind == "text" { node.text }
    else if node.kind == "code" { text(font: ctx.tokens.fonts.mono, raw(node.text)) }
    else if node.kind == "emph" { emph((ctx.components.rich)(ctx, node.body)) }
    else if node.kind == "strong" { strong((ctx.components.rich)(ctx, node.body)) }
    else if node.kind == "link" { link(node.href, (ctx.components.rich)(ctx, node.body)) }
  }
}

#let styled-text(ctx, style, body) = {
  set text(
    font: ctx.tokens.fonts.at(style.font),
    size: style.font_size,
    weight: style.weight,
    fill: ctx.tokens.colors.at(style.color),
  )
  body
}

#let heading(ctx, level, body) = {
  let style = ctx.styles.at("heading_" + str(level))
  block(above: style.space_before, below: style.space_after)[
    #set par(leading: style.line_spacing, spacing: style.paragraph_spacing)
    #(ctx.components.styled_text)(ctx, style, body)
  ]
}

#let resolve-styles(styles) = {
  let resolved = styles
  resolved.insert("heading", merge(resolved.body, resolved.heading))
  for level in range(1, 6) {
    let key = "heading_" + str(level)
    resolved.insert(key, merge(resolved.heading, resolved.at(key)))
  }
  resolved
}

#let header(ctx) = stack(
  dir: ttb,
  spacing: ctx.styles.heading_1.space_after,
  [#(ctx.components.styled_text)(ctx, ctx.styles.heading_1, [#ctx.cv.profile.name])],
  [#ctx.cv.profile.contacts.join(", ")],
)

#let summary(ctx) = if ctx.cv.profile.summary != none [
  #(ctx.components.rich)(ctx, ctx.cv.profile.summary)
]

#let section-heading(ctx, section) = (ctx.components.heading)(ctx, 2, [#section.title])

#let highlight-list(ctx, items) = if items.len() > 0 {
  set list(spacing: ctx.styles.list.item_spacing)
  for item in items [
    - #(ctx.components.rich)(ctx, item)
  ]
}

#let entry(ctx, entry) = {
  block(above: 0pt, below: 0pt, breakable: false)[
    #if entry.kind == "text" {
      let items = (entry.primary,) + entry.highlights
      (ctx.components.highlight_list)(ctx, items)
    } else {
      (ctx.components.heading)(ctx, 3, (ctx.components.rich)(ctx, entry.primary))
      if entry.secondary != none [
        #(ctx.components.rich)(ctx, entry.secondary)
      ]
      if entry.period != none [#entry.period]
      (ctx.components.highlight_list)(ctx, entry.highlights)
    }
  ]
}

#let section(ctx, section) = {
  (ctx.components.section_heading)(ctx, section)
  if section.entries.len() > 0 { v(ctx.styles.section.space_after_heading) }
  for (index, entry) in section.entries.enumerate() {
    (ctx.components.entry)(ctx, entry)
    if index + 1 < section.entries.len() { v(ctx.styles.entry.space_after) }
  }
}

#let footer(ctx) = align(center, text(
  fill: ctx.tokens.colors.at(ctx.styles.footer.color),
  context counter(page).display("1 / 1", both: true),
))

#let document-component(ctx) = {
  let body = ctx.styles.body
  set document(title: ctx.cv.profile.name + " CV", author: ctx.cv.profile.name)
  set page(
    paper: ctx.page.paper,
    margin: ctx.page.margin,
    numbering: none,
    footer: (ctx.components.footer)(ctx),
  )
  set text(
    font: ctx.tokens.fonts.at(body.font),
    size: body.font_size,
    weight: body.weight,
    fill: ctx.tokens.colors.at(body.color),
  )
  set par(justify: body.justify, leading: body.line_spacing, spacing: body.paragraph_spacing)
  set list(spacing: ctx.styles.list.item_spacing)
  set enum(spacing: ctx.styles.list.item_spacing)
  show link: underline
  (ctx.components.header)(ctx)
  (ctx.components.summary)(ctx)
  for section in ctx.cv.sections {
    v(ctx.styles.section.space_before)
    (ctx.components.section)(ctx, section)
  }
}

#let components = (
  document: document-component,
  header: header,
  summary: summary,
  section: section,
  section_heading: section-heading,
  entry: entry,
  highlight_list: highlight-list,
  footer: footer,
  heading: heading,
  styled_text: styled-text,
  rich: rich,
)

#let extend(tokens: (:), styles: (:), page: (:), components: (:)) = (
  api_version: api_version,
  tokens: tokens,
  styles: styles,
  page: page,
  components: components,
)

#let apply-settings(tokens, styles, page, settings) = {
  let resolved-tokens = tokens
  let resolved-styles = styles
  let resolved-page = page

  if "paper" in settings { resolved-page.insert("paper", settings.paper) }
  if "page_margin" in settings { resolved-page.insert("margin", eval(settings.page_margin, mode: "code")) }
  if "font" in settings {
    resolved-tokens.fonts.insert("body", settings.font)
    resolved-tokens.fonts.insert("heading", settings.font)
  }
  if "font_size" in settings { resolved-styles.body.insert("font_size", eval(settings.font_size, mode: "code")) }
  if "line_spacing" in settings { resolved-styles.body.insert("line_spacing", eval(settings.line_spacing, mode: "code")) }
  if "list_item_spacing" in settings { resolved-styles.list.insert("item_spacing", eval(settings.list_item_spacing, mode: "code")) }
  if "section_spacing" in settings { resolved-styles.section.insert("space_before", eval(settings.section_spacing, mode: "code")) }
  if "entry_spacing" in settings { resolved-styles.entry.insert("space_after", eval(settings.entry_spacing, mode: "code")) }

  (tokens: resolved-tokens, styles: resolved-styles, page: resolved-page)
}

#let render(cv, theme, settings) = {
  let tokens = merge(default-tokens, theme.tokens)
  let styles = merge(default-styles, theme.styles)
  let page = merge(default-page, theme.page)
  let resolved = apply-settings(tokens, styles, page, settings)
  let resolved-styles = resolve-styles(resolved.styles)
  let resolved-components = merge(components, theme.components)
  let ctx = (
    cv: cv,
    tokens: resolved.tokens,
    styles: resolved-styles,
    page: resolved.page,
    components: resolved-components,
  )
  (resolved-components.document)(ctx)
}
