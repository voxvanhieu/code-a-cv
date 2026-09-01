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
  header: (alignment: left),
  link: (underline: true),
  section: (
    space_before: 1.2em,
    space_after_heading: 0.65em,
  ),
  entry: (space_after: 1.2em, allow_page_break: false),
  highlight: (bullet: [•]),
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

#let header(ctx) = align(ctx.styles.header.alignment, stack(
  dir: ttb,
  spacing: ctx.styles.heading_1.space_after,
  [#(ctx.components.styled_text)(ctx, ctx.styles.heading_1, [#ctx.cv.profile.name])],
  [#ctx.cv.profile.contacts.join(", ")],
))

#let summary(ctx) = if ctx.cv.profile.summary != none [
  #(ctx.components.rich)(ctx, ctx.cv.profile.summary)
]

#let section-heading(ctx, section) = (ctx.components.heading)(ctx, 2, [#section.title])

#let highlight-list(ctx, items) = if items.len() > 0 {
  set list(spacing: ctx.styles.list.item_spacing, marker: ctx.styles.highlight.bullet)
  for item in items [
    - #(ctx.components.rich)(ctx, item)
  ]
}

#let entry(ctx, entry) = {
  block(above: 0pt, below: 0pt, breakable: ctx.styles.entry.allow_page_break)[
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
  if ctx.styles.link.underline { show link: underline }
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
  if "page_margins" in settings {
    let current = resolved-page.margin
    let side = (name) => if type(current) == dictionary {
      let axis = if name == "top" or name == "bottom" { "y" } else { "x" }
      current.at(name, default: current.at(axis, default: 0pt))
    } else { current }
    let margins = settings.page_margins
    resolved-page.insert("margin", (
      top: if "top" in margins { eval(margins.top, mode: "code") } else { side("top") },
      bottom: if "bottom" in margins { eval(margins.bottom, mode: "code") } else { side("bottom") },
      left: if "left" in margins { eval(margins.left, mode: "code") } else { side("left") },
      right: if "right" in margins { eval(margins.right, mode: "code") } else { side("right") },
    ))
  }
  if "font" in settings {
    resolved-tokens.fonts.insert("body", settings.font)
    resolved-tokens.fonts.insert("heading", settings.font)
  }
  if "heading_font" in settings { resolved-tokens.fonts.insert("heading", settings.heading_font) }
  if "accent_color" in settings { resolved-tokens.colors.insert("accent", rgb(settings.accent_color)) }
  if "font_size" in settings { resolved-styles.body.insert("font_size", eval(settings.font_size, mode: "code")) }
  if "line_spacing" in settings { resolved-styles.body.insert("line_spacing", eval(settings.line_spacing, mode: "code")) }
  if "list_item_spacing" in settings { resolved-styles.list.insert("item_spacing", eval(settings.list_item_spacing, mode: "code")) }
  if "section_spacing" in settings { resolved-styles.section.insert("space_before", eval(settings.section_spacing, mode: "code")) }
  if "entry_spacing" in settings { resolved-styles.entry.insert("space_after", eval(settings.entry_spacing, mode: "code")) }
  if "body_alignment" in settings { resolved-styles.body.insert("justify", settings.body_alignment == "justified") }
  if "link_underline" in settings { resolved-styles.link.insert("underline", settings.link_underline) }
  if "header_alignment" in settings {
    let alignment = if settings.header_alignment == "left" { left }
      else if settings.header_alignment == "right" { right }
      else { center }
    resolved-styles.header.insert("alignment", alignment)
  }
  if "section_heading_spacing" in settings { resolved-styles.section.insert("space_after_heading", eval(settings.section_heading_spacing, mode: "code")) }
  if "highlight_bullet" in settings { resolved-styles.highlight.insert("bullet", settings.highlight_bullet) }
  if "allow_entry_page_break" in settings { resolved-styles.entry.insert("allow_page_break", settings.allow_entry_page_break) }

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
