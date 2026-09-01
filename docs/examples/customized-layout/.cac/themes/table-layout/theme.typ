#import "/.cac/base.typ" as base

#let entry-details(ctx, entry) = [
  #strong((ctx.components.rich)(ctx, entry.primary))
  #if entry.secondary != none [
    #linebreak()
    #emph((ctx.components.rich)(ctx, entry.secondary))
  ]
  #(ctx.components.highlight_list)(ctx, entry.highlights)
]

#let dated-table(ctx, entries, first-title, second-title, date-width: 30mm) = {
  let cells = ()
  for entry in entries {
    cells.push(if entry.period != none { [#entry.period] } else { [] })
    cells.push(entry-details(ctx, entry))
  }
  table(
    columns: (date-width, 1fr),
    column-gutter: 8pt,
    inset: (x: 5pt, y: 4pt),
    stroke: (x, y) => if y == 0 { (bottom: 0.6pt + ctx.tokens.colors.accent) } else { none },
    table.header(
      [#strong(first-title)],
      [#strong(second-title)],
    ),
    ..cells,
  )
}

#let details-first-table(ctx, entries, first-title, second-title) = {
  let cells = ()
  for entry in entries {
    cells.push(entry-details(ctx, entry))
    cells.push(if entry.period != none { [#entry.period] } else { [] })
  }
  table(
    columns: (1fr, auto),
    column-gutter: 8pt,
    inset: (x: 5pt, y: 4pt),
    stroke: (x, y) => if y == 0 { (bottom: 0.6pt + ctx.tokens.colors.accent) } else { none },
    table.header(
      [#strong(first-title)],
      [#strong(second-title)],
    ),
    ..cells,
  )
}

#let skills-table(ctx, entries) = {
  let cells = ()
  for entry in entries {
    cells.push([#strong((ctx.components.rich)(ctx, entry.primary))])
    cells.push((ctx.components.highlight_list)(ctx, entry.highlights))
  }
  table(
    columns: (36mm, 1fr),
    column-gutter: 8pt,
    inset: (x: 5pt, y: 4pt),
    stroke: none,
    ..cells,
  )
}

#let fallback-entries(ctx, entries) = {
  for (index, entry) in entries.enumerate() {
    (ctx.components.entry)(ctx, entry)
    if index + 1 < entries.len() { v(ctx.styles.entry.space_after) }
  }
}

#let section(ctx, section) = {
  (ctx.components.section_heading)(ctx, section)
  if section.entries.len() > 0 {
    v(ctx.styles.section.space_after_heading)
    let kind = section.entries.first().kind
    if kind == "experience" {
      dated-table(ctx, section.entries, "Period", "Role and organization")
    } else if kind == "education" {
      dated-table(ctx, section.entries, "Period", "Qualification and institution")
    } else if kind == "project" {
      details-first-table(ctx, section.entries, "Project", "Period")
    } else if kind == "publication" {
      details-first-table(ctx, section.entries, "Publication", "Date")
    } else if kind == "skill-group" {
      skills-table(ctx, section.entries)
    } else {
      fallback-entries(ctx, section.entries)
    }
  }
}

#let section-heading(ctx, section) = {
  (ctx.components.heading)(ctx, 2, [#section.title])
  line(length: 100%, stroke: 1pt + ctx.tokens.colors.accent)
}

#let theme = base.extend(
  tokens: (
    fonts: (body: "New Computer Modern", heading: "New Computer Modern"),
  ),
  styles: (
    heading: (color: "accent"),
    heading_1: (font_size: 24pt),
    heading_2: (font_size: 12pt),
    heading_3: (font_size: 10pt),
    list: (item_spacing: 0.25em),
    entry: (space_after: 0.7em),
  ),
  page: (paper: "a4", margin: 16mm),
  components: (
    section: section,
    section_heading: section-heading,
  ),
)
