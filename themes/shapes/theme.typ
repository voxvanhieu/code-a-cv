#import "/.cac/base.typ" as base

// The shapes theme: a sidebar layout with soft geometric shapes behind the
// sidebar column, an accent palette, and level bars for languages.
// Original design in the sidebar genre — not a clone of any commercial
// template.

#let accent = rgb("2d5a7a")
#let accent-soft = rgb("2d5a7a").lighten(88%)

#let shapes-background(ctx) = {
  // Soft circles anchored to the sidebar's outer edge; content flows above.
  place(top + left, dx: -28pt, dy: -30pt,
    circle(radius: 110pt, fill: accent-soft))
  place(bottom + left, dx: -34pt, dy: 24pt,
    circle(radius: 80pt, fill: accent-soft))
  place(top + right, dx: 24pt, dy: -36pt,
    square(size: 60pt, fill: accent-soft, radius: 12pt))
}

#let shapes-section-heading(ctx, section, number) = {
  let h = ctx.helpers
  h.at("heading")(ctx, 2, [
    #if number != none [#number  ]
    #section.title
  ])
}

#let shapes-language(ctx, entry) = {
  let h = ctx.helpers
  block(above: 0.55em)[
    #text(weight: "semibold")[#h.at("rich")(ctx, entry.primary)]

    #for highlight in entry.at("highlights", default: ()) [
      #text(size: 0.9em)[#h.at("rich")(ctx, highlight)]
    ]
  ]
}

#let shapes-document(ctx) = {
  let body = ctx.styles.body
  set document(title: ctx.cv.profile.name + " CV", author: ctx.cv.profile.name)
  set page(
    paper: ctx.page.paper,
    margin: (top: 16mm, bottom: 16mm, left: 14mm, right: 14mm),
    numbering: none,
    footer: (ctx.components.footer)(ctx),
  )
  set text(
    font: "New Computer Modern",
    size: body.font_size,
    weight: body.weight,
    fill: ctx.tokens.colors.at(body.color),
  )
  set par(leading: body.line_spacing, spacing: body.paragraph_spacing)
  set list(spacing: ctx.styles.list.item_spacing)
  if ctx.styles.link.underline { show link: underline }

  shapes-background(ctx)

  // Sidebar panel behind the skills/languages column.
  let sidebar-sections = ctx.cv.sections.filter(section => {
    let name = section.id
    name == "skills" or name == "languages" or name == "references"
  })
  let main-sections = ctx.cv.sections.filter(section => {
    let name = section.id
    not (name == "skills" or name == "languages" or name == "references")
  })

  grid(
    columns: (34%, 1fr),
    column-gutter: 1.5em,
    align(top, {
      let h = ctx.helpers
      if ctx.cv.profile.photo != none {
        align(center, circle(radius: 15mm, image("/" + ctx.cv.profile.photo, width: 30mm, height: 30mm)))
        v(0.9em)
      }
      text(size: 1.05em, weight: "bold")[#ctx.cv.profile.name]
      v(0.5em)
      if ctx.cv.profile.headline != none {
        text(size: 0.88em, fill: ctx.tokens.colors.at("muted"))[#ctx.cv.profile.headline]
        v(0.7em)
      }
      for contact in ctx.cv.profile.contacts {
        // The view keeps href on contacts; render links for targets.
        if contact.at("href", default: none) != none {
          block[#link(contact.href, contact.label)]
        } else {
          block[#contact.label]
        }
      }
      v(0.9em)
      if ctx.cv.profile.summary != none {
        block[#h.at("rich")(ctx, ctx.cv.profile.summary)]
        v(0.9em)
      }
      for section in sidebar-sections {
        shapes-section-heading(ctx, section, none)
        v(0.35em)
        for entry in section.entries {
          if entry.kind == "language" {
            shapes-language(ctx, entry)
          } else {
            for highlight in entry.at("highlights", default: ()) {
              block[- #highlight.map(node => node.text).join()]
            }
          }
        }
        v(0.9em)
      }
    }),
    align(top, {
      let number = 1
      for section in main-sections {
        shapes-section-heading(ctx, section, number)
        number += 1
        v(0.45em)
        for (index, entry) in section.entries.enumerate() {
          (ctx.components.entry)(ctx, entry)
          if index + 1 < section.entries.len() { v(ctx.styles.entry.space_after) }
        }
        v(1em)
      }
    }),
  )
}

// A layout-only theme: hooks + tokens straight in the theme dictionary.
#let theme = (
  tokens: (
    colors: (accent: accent),
  ),
  layout: (
    family: "sidebar",
    document: shapes-document,
    section_heading: shapes-section-heading,
  ),
)
