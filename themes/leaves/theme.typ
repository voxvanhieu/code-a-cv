#import "/.cac/base.typ" as base

// The leaves theme: a full-bleed warm-toned design with leaf-like
// decorative shapes, a header banner carrying the name and contacts, and
// a single-column flow below. Original design in the decorative genre —
// not a clone of any commercial template.

#let leaf-deep = rgb("3d5a45")
#let leaf-mid = rgb("5a7d5a").lighten(30%)
#let leaf-soft = rgb("e8efe4")
#let ink = rgb("22301f")

#let leaves-background(ctx) = {
  // Organic shapes anchored to the corners: overlapping circles and a
  // rounded leaf wedge. Content renders above them.
  place(top + left, dx: -34pt, dy: -40pt,
    circle(radius: 96pt, fill: leaf-mid))
  place(top + left, dx: 12pt, dy: -58pt,
    circle(radius: 60pt, fill: leaf-deep))
  place(bottom + right, dx: 30pt, dy: 30pt,
    circle(radius: 88pt, fill: leaf-soft))
  place(bottom + right, dx: -20pt, dy: 44pt,
    square(size: 46pt, fill: leaf-mid, radius: 20pt))
}

#let leaves-section-heading(ctx, section, number) = {
  let padded = if number < 10 { "0" + str(number) } else { str(number) }
  let title = padded + "  " + section.title
  block(above: ctx.styles.section.space_before, below: 0.45em)[
    #text(size: 1.05em, weight: "bold", fill: leaf-deep)[#title]
  ]
  line(length: 30%, stroke: 1pt + leaf-mid)
}

#let leaves-document(ctx) = {
  let body = ctx.styles.body
  set document(title: ctx.cv.profile.name + " CV", author: ctx.cv.profile.name)
  set page(
    paper: ctx.page.paper,
    margin: (top: 34mm, bottom: 20mm, left: 16mm, right: 16mm),
    numbering: none,
    footer: (ctx.components.footer)(ctx),
  )
  set text(font: "New Computer Modern", size: body.font_size, fill: ink)
  set par(leading: body.line_spacing, spacing: body.paragraph_spacing)
  set list(spacing: ctx.styles.list.item_spacing)
  if ctx.styles.link.underline { show link: underline }

  leaves-background(ctx)

  // Header banner: name and contacts on the soft band.
  place(top + left, dy: 12pt, rect(width: 100%, height: 44pt, fill: leaf-soft))
  place(top + left, dy: 20pt, dx: 16pt)[
    #text(size: 1.6em, weight: "bold", fill: leaf-deep)[#ctx.cv.profile.name]
    #h(1em)
    #text(size: 0.85em)[
      #for contact in ctx.cv.profile.contacts [
        #if contact.href != none [#link(contact.href, contact.label) #h(0.6em)] else [#contact.label #h(0.6em)]
      ]
    ]
  ]

  v(38pt)
  if ctx.cv.profile.summary != none {
    block[#(ctx.components.rich)(ctx, ctx.cv.profile.summary)]
    v(0.6em)
  }

  let number = 1
  for section in ctx.cv.sections {
    leaves-section-heading(ctx, section, number)
    number += 1
    v(0.45em)
    for (index, entry) in section.entries.enumerate() {
      (ctx.components.entry)(ctx, entry)
      if index + 1 < section.entries.len() { v(ctx.styles.entry.space_after) }
    }
    v(1em)
  }
}

#let theme = (
  layout: (
    document: leaves-document,
    section_heading: leaves-section-heading,
  ),
)
