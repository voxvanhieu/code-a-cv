
#import "style.typ": dictionaries, dictionaries-sparse
#import "layout.typ": families

// The built-in dictionaries come from the resolved style sheet
// (/.cac/style.json); no defaults are written here anymore.

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

// The view's `period` slot is already display-formatted ("Jan 2023 –
// Present"); pass it through. Structured dates arrive Typst-side with the
// locale tables in a later slice.

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
  // Open fields (V1): author-written Key: value lines the model does not
  // type, rendered after the typed metadata. Absent for documents without
  // them, so existing output is unchanged.
  if entry.fields != none {
    for (key, value) in entry.fields {
      let label = key.split("-").map(part => upper(part.slice(0, 1)) + part.slice(1)).join(" ")
      block(above: 0pt, below: ctx.styles.body.line_spacing)[#text(fill: ctx.tokens.colors.at("muted"), size: 0.9em)[#label:] #value]
    }
  }
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

// The 0.3 adapter: legacy themes call classic_components(...) and extend
// `components`; removed from the family path, kept for one release.
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

#let render(cv, theme, style, settings-style) = {
  assert(type(theme) == dictionary, message: "theme must be a dictionary; export theme = base.extend(...)")
  for field in ("tokens", "styles", "page", "components") {
    assert(type(theme.at(field, default: (:))) == dictionary, message: "theme." + field + " must be a dictionary")
  }
  let sheet = dictionaries(style)
  // The legacy adapter: a 0.3 theme dictionary overlays the sheet, and the
  // settings layer overlays the theme, keeping the 0.3 precedence (settings
  // beat project themes) until themes migrate to theme.toml.
  let settings-dicts = dictionaries-sparse(settings-style)
  let tokens = merge(merge(sheet.tokens, theme.at("tokens", default: (:))), settings-dicts.tokens)
  let styles = merge(merge(sheet.styles, theme.at("styles", default: (:))), settings-dicts.styles)
  let page = merge(merge(sheet.page, theme.at("page", default: (:))), settings-dicts.page)
  let resolved-styles = resolve-styles(styles)

  let helpers = (
    heading: heading,
    entry_details: entry_details,
    entry_flow: entry_flow,
    section_start: section_start,
    keep_entry: keep-entry,
    highlight_list: highlight-list,
    rich: rich,
    styled_text: styled-text,
    merge: merge,
    format-period: (ctx, period) => period,
    section_table: section_table,
    section_labels: section_labels,
    table_kind: table_kind,
  )
  let theme-layout = theme.at("layout", default: none)
  let resolved-components = if theme-layout != none {
    // Hooks path: a family plus optional hook overrides. Hooks left as
    // none reuse the base helper verbatim (document, header).
    let family = merge(families.at(theme-layout.at("family", default: "classic"), default: families.classic), theme-layout)
    // Base defaults, then the family (which carries theme-layout extras
    // like requires/allowed), then the resolved hooks on top.
    merge(merge(components, family), (
      document: if family.document == none { document-component } else { family.document },
      header: if family.header == none { header } else { family.header },
      section: if family.section == none { section_flow } else { family.section },
      entry: if family.entry == none { entry_flow } else { family.entry },
      section_heading: if family.section_heading == none { section-heading } else { family.section_heading },
      entry_details: if family.entry_details == none { entry_details } else { family.entry_details },
    ))
  } else {
    // Legacy adapter: 0.3 component dictionaries, for one release.
    merge(components, theme.at("components", default: (:)))
  }
  let ctx = (
    cv: cv,
    tokens: tokens,
    styles: resolved-styles,
    page: page,
    components: resolved-components,
    layout: resolved-components,
    helpers: helpers,
  )
  (resolved-components.document)(ctx)
}
