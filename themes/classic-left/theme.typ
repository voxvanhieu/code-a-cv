#import "/.cac/base.typ" as base

#let header(ctx) = align(ctx.styles.header.alignment)[
  #(ctx.components.heading)(ctx, 1, [#ctx.cv.profile.name])
  #if ctx.cv.profile.contacts.len() > 0 [#ctx.cv.profile.contacts.join(" | ")]
  #line(length: 100%, stroke: 0.5pt + ctx.tokens.colors.accent)
]

#let section-heading(ctx, section) = {
  (ctx.components.heading)(ctx, 2, [#section.title])
  line(length: 100%, stroke: 0.5pt + ctx.tokens.colors.accent)
}

#let entry(ctx, entry) = pad(left: 0.15in)[
  #block(breakable: ctx.styles.entry.allow_page_break)[
    #if entry.kind == "text" {
      let items = (entry.primary,) + entry.highlights
      (ctx.components.highlight_list)(ctx, items)
    } else {
      grid(
        columns: (1fr, auto),
        column-gutter: 1em,
        [#(ctx.components.styled_text)(ctx, ctx.styles.heading_3, (ctx.components.rich)(ctx, entry.primary))],
        if entry.period != none { entry.period } else { none },
      )
      if entry.secondary != none [
        #emph((ctx.components.rich)(ctx, entry.secondary))
      ]
      (ctx.components.highlight_list)(ctx, entry.highlights)
    }
  ]
]

#let section(ctx, section) = if section.entries.len() == 0 {
  (ctx.components.section_heading)(ctx, section)
} else {
  block(breakable: ctx.styles.entry.allow_page_break)[
    #(ctx.components.section_heading)(ctx, section)
    #v(ctx.styles.section.space_after_heading)
    #(ctx.components.entry)(ctx, section.entries.first())
  ]
  for entry in section.entries.slice(1) {
    v(ctx.styles.entry.space_after)
    (ctx.components.entry)(ctx, entry)
  }
}

#let theme = base.extend(
  tokens: (
    fonts: (body: "New Computer Modern", heading: "New Computer Modern"),
  ),
  styles: (
    body: (font_size: 10pt),
    header: (alignment: left),
    heading_1: (font_size: 24pt),
    heading_2: (font_size: 12pt, weight: "regular"),
    heading_3: (font_size: 10pt),
  ),
  page: (paper: "us-letter", margin: 12.7mm),
  components: (
    header: header,
    section: section,
    section_heading: section-heading,
    entry: entry,
  ),
)
