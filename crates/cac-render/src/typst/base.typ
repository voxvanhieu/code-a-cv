
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
    else if node.kind == "paragraph" { parbreak(); (ctx.components.rich)(ctx, node.body); parbreak() }
    else if node.kind == "break" { linebreak() }
    else if node.kind == "list" {
      parbreak()
      if node.start == none { (ctx.components.highlight_list)(ctx, node.items) }
      else { enum(start: node.start, spacing: ctx.styles.list.item_spacing, ..node.items.map(item => (ctx.components.rich)(ctx, item))) }
      parbreak()
    }
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

#let contacts(ctx, separator: " | ") = ctx.cv.profile.contacts.map(contact => {
  if contact.href == none { contact.label } else { link(contact.href, contact.label) }
}).join(separator)

#let header(ctx) = align(ctx.styles.header.alignment)[
  #(ctx.components.heading)(ctx, 1, [#ctx.cv.profile.name])
  #contacts(ctx)
]

// Measure against a full page, so short entries move together and oversized
// entries can continue. The caller supplies the actual column width.
#let can-break(ctx, body, width, reserve: 0pt) = {
  let margins = page.margin
  let side(name) = {
    let value = if type(margins) == dictionary { margins.at(name, default: auto) } else { margins }
    if value == auto { calc.min(page.width, page.height) * 2.5 / 21 }
    else if type(value) == relative { value.length.to-absolute() + value.ratio * page.height }
    else { value.to-absolute() }
  }
  let available = page.height - side("top") - side("bottom") - reserve.to-absolute()
  ctx.styles.entry.allow_page_break or measure(body, width: width).height > available
}

#let keep-entry(ctx, body) = context layout(size => {
  block(above: 0pt, below: 0pt, breakable: can-break(ctx, body, size.width), body)
})

#let entry_details(ctx, entry) = {
  (ctx.components.heading)(ctx, 3, (ctx.components.rich)(ctx, entry.primary))
  if entry.secondary != none { block(above: 0pt, below: ctx.styles.body.line_spacing, (ctx.components.rich)(ctx, entry.secondary)) }
  for item in entry.metadata { block(above: 0pt, below: ctx.styles.body.line_spacing, (ctx.components.rich)(ctx, item.body)) }
  (ctx.components.highlight_list)(ctx, entry.highlights)
}

#let entry_flow(ctx, entry) = keep-entry(ctx, {
  if entry.kind == "prose" {
    (ctx.components.rich)(ctx, entry.primary)
  } else if entry.kind == "text" {
    (ctx.components.highlight_list)(ctx, (entry.primary,) + entry.highlights)
    if entry.secondary != none { (ctx.components.rich)(ctx, entry.secondary) }
    if entry.period != none { block(entry.period) }
    for item in entry.metadata { block((ctx.components.rich)(ctx, item.body)) }
  } else {
    (ctx.components.entry_details)(ctx, entry)
    if entry.period != none { block(above: ctx.styles.body.line_spacing, below: 0pt, entry.period) }
  }
})

#let section_start(ctx, section) = block(above: 0pt, below: 0pt, sticky: section.entries.len() > 0, (ctx.components.section_heading)(ctx, section))

#let section_flow(ctx, section) = {
  section_start(ctx, section)
  if section.entries.len() > 0 { v(ctx.styles.section.space_after_heading) }
  for (index, entry) in section.entries.enumerate() {
    (ctx.components.entry)(ctx, entry)
    if index + 1 < section.entries.len() { v(ctx.styles.entry.space_after) }
  }
}

#let table_kind(section) = {
  if section.entries.len() == 0 { return none }
  let kind = section.entries.first().kind
  if not ("experience", "education", "project", "publication", "skill-group").contains(kind) { return none }
  if not section.entries.all(entry => entry.kind == kind) { return none }
  if kind == "skill-group" and not section.entries.all(entry => entry.secondary == none and entry.period == none and entry.metadata.len() == 0) { return none }
  kind
}

#let section_table(ctx, section, headers: none, date_width: 30mm, details_first: auto) = {
  let kind = table_kind(section)
  if kind == none or kind == "skill-group" { return section_flow(ctx, section) }
  let first = if details_first == auto { ("project", "publication").contains(kind) } else { details_first }
  let dated = section.entries.any(entry => entry.period != none)
  section_start(ctx, section)
  v(ctx.styles.section.space_after_heading)
  context layout(size => {
    let detail_width = if dated { size.width - date_width - 8pt } else { size.width }
    let head = if headers == none { () } else {
      let labels = if not dated { (headers.details,) } else if first { (headers.details, headers.period) } else { (headers.period, headers.details) }
      (table.header(..labels.map(label => (ctx.components.styled_text)(ctx, ctx.styles.heading_3, label))),)
    }
    let columns = if not dated { (1fr,) } else if first { (1fr, date_width) } else { (date_width, 1fr) }
    let header_height = if headers == none { 0pt } else {
      measure(table(columns: columns, column-gutter: 8pt, inset: 0pt, stroke: none, ..head), width: size.width).height + ctx.styles.entry.space_after
    }
    let cells = ()
    for entry in section.entries {
      let details = (ctx.components.entry_details)(ctx, entry)
      let breakable = can-break(ctx, details, detail_width, reserve: header_height)
      let detail_cell = table.cell(breakable: breakable, details)
      let date_cell = table.cell(breakable: breakable, if entry.period == none { [] } else { entry.period })
      cells += if not dated { (detail_cell,) } else if first { (detail_cell, date_cell) } else { (date_cell, detail_cell) }
    }
    table(
      columns: columns,
      column-gutter: 8pt,
      row-gutter: ctx.styles.entry.space_after,
      inset: 0pt,
      stroke: none,
      ..head, ..cells,
    )
  })
}

#let section_labels(ctx, section, label_width: 36mm) = {
  if table_kind(section) != "skill-group" or section.entries.all(entry => entry.highlights.len() == 0) { return section_flow(ctx, section) }
  section_start(ctx, section)
  v(ctx.styles.section.space_after_heading)
  context layout(size => {
    let cells = ()
    for entry in section.entries {
      let label = (ctx.components.heading)(ctx, 3, (ctx.components.rich)(ctx, entry.primary))
      let values = (ctx.components.highlight_list)(ctx, entry.highlights)
      let breakable = can-break(ctx, values, size.width - label_width - 8pt) or can-break(ctx, label, label_width)
      cells.push(table.cell(breakable: breakable, label))
      cells.push(table.cell(breakable: breakable, values))
    }
    table(columns: (label_width, 1fr), column-gutter: 8pt, row-gutter: ctx.styles.entry.space_after, inset: 0pt, stroke: none, ..cells)
  })
}

#let classic_components(header_rule: false, entry_indent: 0.15in) = (
  header: ctx => {
    header(ctx)
    if header_rule { line(length: 100%, stroke: 0.5pt + ctx.tokens.colors.accent) }
  },
  section_heading: (ctx, section) => {
    (ctx.components.heading)(ctx, 2, [#section.title])
    line(length: 100%, stroke: 0.5pt + ctx.tokens.colors.accent)
  },
  entry_details: (ctx, entry) => entry_details(ctx, (
    ..entry,
    secondary: if entry.secondary == none { none } else { ((kind: "emph", body: entry.secondary),) },
  )),
  entry: (ctx, entry) => pad(left: entry_indent, if entry.kind in ("text", "prose") {
    entry_flow(ctx, entry)
  } else {
    let heading = ctx.components.heading
    let details_ctx = (
      ..ctx,
      components: merge(ctx.components, (
        heading: (ctx, level, body) => if level == 3 and entry.period != none {
          block(above: 0pt, below: ctx.styles.heading_3.space_after, grid(
            columns: (1fr, auto), column-gutter: 1em,
            heading(ctx, level, body),
            entry.period,
          ))
        } else { heading(ctx, level, body) },
      )),
    )
    keep-entry(ctx, (ctx.components.entry_details)(details_ctx, entry))
  }),
)


#let components = (
  document: document-component,
  header: header,
  summary: summary,
  section: section_flow,
  section_heading: section-heading,
  entry: entry_flow,
  entry_details: entry_details,
  highlight_list: highlight-list,
  footer: footer,
  heading: heading,
  styled_text: styled-text,
  rich: rich,
)

#let extend(tokens: (:), styles: (:), page: (:), components: (:)) = (
  tokens: tokens,
  styles: styles,
  page: page,
  components: components,
)

#let apply-settings(tokens, styles, page, settings) = {
  let resolved-tokens = tokens
  let resolved-styles = styles
  let resolved-page = page

  let page-settings = settings.at("page", default: (:))
  let typography = settings.at("typography", default: (:))
  let style = settings.at("style", default: (:))
  let spacing = settings.at("spacing", default: (:))
  let pagination = settings.at("pagination", default: (:))

  if "paper" in page-settings { resolved-page.insert("paper", page-settings.paper) }
  if "margin" in page-settings { resolved-page.insert("margin", eval(page-settings.margin, mode: "code")) }
  if "margins" in page-settings {
    let current = resolved-page.margin
    let side = (name) => if type(current) == dictionary {
      let axis = if name == "top" or name == "bottom" { "y" } else { "x" }
      current.at(name, default: current.at(axis, default: 0pt))
    } else { current }
    let margins = page-settings.margins
    resolved-page.insert("margin", (
      top: if "top" in margins { eval(margins.top, mode: "code") } else { side("top") },
      bottom: if "bottom" in margins { eval(margins.bottom, mode: "code") } else { side("bottom") },
      left: if "left" in margins { eval(margins.left, mode: "code") } else { side("left") },
      right: if "right" in margins { eval(margins.right, mode: "code") } else { side("right") },
    ))
  }
  if "font" in typography {
    resolved-tokens.fonts.insert("body", typography.font)
    resolved-tokens.fonts.insert("heading", typography.font)
  }
  if "heading_font" in typography { resolved-tokens.fonts.insert("heading", typography.heading_font) }
  if "font_size" in typography { resolved-styles.body.insert("font_size", eval(typography.font_size, mode: "code")) }
  if "line_spacing" in typography { resolved-styles.body.insert("line_spacing", eval(typography.line_spacing, mode: "code")) }
  if "accent_color" in style { resolved-tokens.colors.insert("accent", rgb(style.accent_color)) }
  if "body_alignment" in style { resolved-styles.body.insert("justify", style.body_alignment == "justified") }
  if "link_underline" in style { resolved-styles.link.insert("underline", style.link_underline) }
  if "header_alignment" in style {
    let alignment = if style.header_alignment == "left" { left }
      else if style.header_alignment == "right" { right }
      else { center }
    resolved-styles.header.insert("alignment", alignment)
  }
  if "highlight_bullet" in style { resolved-styles.highlight.insert("bullet", style.highlight_bullet) }
  if "list_item" in spacing { resolved-styles.list.insert("item_spacing", eval(spacing.list_item, mode: "code")) }
  if "section" in spacing { resolved-styles.section.insert("space_before", eval(spacing.section, mode: "code")) }
  if "section_heading" in spacing { resolved-styles.section.insert("space_after_heading", eval(spacing.section_heading, mode: "code")) }
  if "entry" in spacing { resolved-styles.entry.insert("space_after", eval(spacing.entry, mode: "code")) }
  if "allow_entry_page_break" in pagination { resolved-styles.entry.insert("allow_page_break", pagination.allow_entry_page_break) }

  (tokens: resolved-tokens, styles: resolved-styles, page: resolved-page)
}

#let render(cv, theme, settings) = {
  assert(type(theme) == dictionary, message: "theme must be a dictionary; export theme = base.extend(...)")
  for field in ("tokens", "styles", "page", "components") {
    assert(type(theme.at(field, default: (:))) == dictionary, message: "theme." + field + " must be a dictionary")
  }
  let tokens = merge(default-tokens, theme.at("tokens", default: (:)))
  let styles = merge(default-styles, theme.at("styles", default: (:)))
  let page = merge(default-page, theme.at("page", default: (:)))
  let resolved = apply-settings(tokens, styles, page, settings)
  let resolved-styles = resolve-styles(resolved.styles)
  let resolved-components = merge(components, theme.at("components", default: (:)))
  let ctx = (
    cv: cv,
    tokens: resolved.tokens,
    styles: resolved-styles,
    page: resolved.page,
    components: resolved-components,
  )
  (resolved-components.document)(ctx)
}
