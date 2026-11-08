// The layout contract (L1): four hooks over pure helpers.
//
// Families live here; the pure helpers arrive through ctx.helpers, which
// base.typ injects. A theme exports `layout` — either
// `(family: "classic")` to pick a built-in family with overrides, or a
// dictionary of hook overrides directly. The single dynamic hop is
// ctx.components.entry: a section renders its entries through the active
// entry hook, so an overridden entry hook applies everywhere. heading,
// styled-text, rich, highlight-list, entry-details, and section-heading
// are helpers a hook calls explicitly — no longer override points.

#let helpers-of(ctx) = ctx.helpers

// -- classic --

#let classic-section-heading(ctx, section) = {
  let h = helpers-of(ctx)
  h.at("heading")(ctx, 2, [#section.title])
  line(length: 100%, stroke: 0.5pt + ctx.tokens.colors.accent)
}

#let classic-entry-details(ctx, entry) = {
  let h = helpers-of(ctx)
  h.at("entry_details")(ctx, (
    ..entry,
    secondary: if entry.secondary == none { none } else { ((kind: "emph", body: entry.secondary),) },
  ))
}

#let classic-section-flow(ctx, section) = {
  let h = helpers-of(ctx)
  h.at("section_start")(ctx, section)
  if section.entries.len() > 0 { v(ctx.styles.section.space_after_heading) }
  for (index, entry) in section.entries.enumerate() {
    (ctx.components.entry)(ctx, entry)
    if index + 1 < section.entries.len() { v(ctx.styles.entry.space_after) }
  }
}

#let classic-entry(ctx, entry) = {
  let h = helpers-of(ctx)
  pad(left: 0.15in, if entry.kind in ("text", "prose") {
    h.at("entry_flow")(ctx, entry)
  } else {
    let details_ctx = (
      ..ctx,
      components: (h.at("merge"))(ctx.components, (
        heading: (ctx, level, body) => if level == 3 and entry.period != none {
          block(above: 0pt, below: ctx.styles.heading_3.space_after, grid(
            columns: (1fr, auto), column-gutter: 1em,
            h.at("heading")(ctx, level, body),
            entry.period,
          ))
        } else { h.at("heading")(ctx, level, body) },
      )),
    )
    h.at("keep_entry")(ctx, classic-entry-details(details_ctx, entry))
  })
}

#let classic = (document: none, header: none, section: classic-section-flow, entry: classic-entry, section_heading: classic-section-heading, entry_details: classic-entry-details)

// -- table --

// Section titles keep a rule; homogeneous dated sections render as tables
// with the period in a gutter, skills as label rows, everything else in
// flow.
#let table-section(ctx, section) = {
  let h = helpers-of(ctx)
  h.at("section_table")(ctx, section)
}

#let table = (
  document: none,
  header: none,
  section: table-section,
  entry: none,
  section_heading: classic-section-heading,
  entry_details: classic-entry-details,
)

// -- compact --

// No indent, no rules: tight flow with the period beside the title.
#let compact-section-heading(ctx, section) = {
  let h = helpers-of(ctx)
  h.at("heading")(ctx, 2, [#section.title])
}

#let compact = (
  document: none,
  header: none,
  section: classic-section-flow,
  entry: classic-entry,
  section_heading: compact-section-heading,
  entry_details: classic-entry-details,
)



// -- sidebar --

// Placement: the sidebar column carries skills, languages, and
// references; the main column carries everything else, in source order.
#let sidebar-is-sidebar(section) = {
  let name = section.id
  name == "skills" or name == "languages" or name == "references"
}

// Level bars for a numeric 0–5 proficiency; named scales print as labels.
#let level-bars(level) = {
  let filled = if level < 0 { 0 } else if level > 5 { 5 } else { level }
  for i in range(5) {
    if i < filled {
      box(width: 12pt, height: 4pt, fill: rgb("222222"), radius: 1pt)
      h(2pt)
    } else {
      box(width: 12pt, height: 4pt, stroke: 0.5pt + rgb("999999"), radius: 1pt)
      h(2pt)
    }
  }
}

#let sidebar-section-heading(ctx, section, number) = {
  let h = helpers-of(ctx)
  if number != none {
    h.at("heading")(ctx, 2, [#number  #section.title])
  } else {
    h.at("heading")(ctx, 2, [#section.title])
  }
}

#let sidebar-render-entry(ctx, entry) = {
  let h = helpers-of(ctx)
  if entry.kind == "language" {
    block(above: 0.5em)[
      #text(weight: "semibold")[#h.at("rich")(ctx, entry.primary)]
    ]
  } else {
    for highlight in entry.at("highlights", default: ()) {
      block[- #h.at("rich")(ctx, highlight)]
    }
    let content = entry.at("content", default: none)
    if content != none {
      block[#h.at("rich")(ctx, content)]
    }
  }
}

#let sidebar-column(ctx, sections, number) = {
  for section in sections {
    sidebar-section-heading(ctx, section, number)
    number += 1
    v(0.4em)
    for entry in section.entries {
      sidebar-render-entry(ctx, entry)
    }
    v(0.8em)
  }
}

#let sidebar-main-column(ctx, sections, number) = {
  for section in sections {
    sidebar-section-heading(ctx, section, number)
    number += 1
    v(0.4em)
    for (index, entry) in section.entries.enumerate() {
      (ctx.components.entry)(ctx, entry)
      if index + 1 < section.entries.len() { v(ctx.styles.entry.space_after) }
    }
    v(0.8em)
  }
}

#let sidebar-document(ctx) = {
  let body = ctx.styles.body
  set document(title: ctx.cv.profile.name + " CV", author: ctx.cv.profile.name)
  set page(paper: ctx.page.paper, margin: ctx.page.margin, numbering: none, footer: (ctx.components.footer)(ctx))
  set text(
    font: ctx.tokens.fonts.at(body.font),
    size: body.font_size,
    weight: body.weight,
    fill: ctx.tokens.colors.at(body.color),
  )
  set par(leading: body.line_spacing, spacing: body.paragraph_spacing)
  set list(spacing: ctx.styles.list.item_spacing)
  set enum(spacing: ctx.styles.list.item_spacing)
  if ctx.styles.link.underline { show link: underline }
  (ctx.components.header)(ctx)
  v(1em)
  // Photo slot: the profile photo cropped into a circle, when present.
  // The path is project-relative; the world resolves it inside the
  // project jail.
  let photo = if ctx.cv.profile.photo != none {
    circle(radius: 14mm, image("/" + ctx.cv.profile.photo, width: 28mm, height: 28mm))
  } else {
    none
  }
  let sidebar-sections = ctx.cv.sections.filter(section => sidebar-is-sidebar(section))
  let main-sections = ctx.cv.sections.filter(section => not sidebar-is-sidebar(section))
  grid(
    columns: (32%, 1fr),
    column-gutter: 1.5em,
    align(top, {
      if photo != none {
        align(center, photo)
        v(0.8em)
      }
      sidebar-column(ctx, sidebar-sections, 1)
    }),
    align(top, sidebar-main-column(ctx, main-sections, 1 + sidebar-sections.len())),
  )
}

// The sidebar family: two-column regions, numbered sections, language
// proficiencies. document covers the whole page; the other hooks stay at
// their base defaults.
#let sidebar = (
  document: sidebar-document,
  header: none,
  section: none,
  entry: none,
  section_heading: none,
  entry_details: none,
)



// -- form --

// The form family: a fixed grid with a photo cell, address block, and
// labeled rows. A form declares `requires` (sections it needs and profile
// keys it cannot do without) and `allowed` (the section kinds it can
// place); anything else is rejected by name instead of silently dropped.
#let form-document(ctx) = {
  let layout = ctx.layout
  let hh = helpers-of(ctx)
  let requires = layout.at("requires", default: (:))
  let allowed = layout.at("allowed", default: requires.at("sections", default: ()))
  let hh = helpers-of(ctx)
  let body = ctx.styles.body
  set document(title: ctx.cv.profile.name + " CV", author: ctx.cv.profile.name)
  set page(paper: ctx.page.paper, margin: ctx.page.margin, numbering: none)
  set text(
    font: ctx.tokens.fonts.at(body.font),
    size: body.font_size,
    weight: body.weight,
    fill: ctx.tokens.colors.at(body.color),
  )
  set par(leading: body.line_spacing, spacing: body.paragraph_spacing)
  // A form owns the document: sections it cannot place are rejected by
  // name rather than silently flowing into a generic fallback.
  for section in ctx.cv.sections {
    let kind-name = section.kind
    let placed = allowed.contains(kind-name) or allowed.contains(section.id)
    if not placed {
      panic(
        "this form does not support the section `" + section.title + "` (kind `" + section.kind
          + "`); it accepts: " + repr(allowed) + " / kind is " + repr(kind-name),
      )
    }
  }
  // Profile requirements: photo cell and address block need their data.
  for key in requires.at("profile", default: ()) {
    let missing = if key == "photo" { ctx.cv.profile.photo == none } else { false }
    if missing {
      panic("this form requires profile `" + key + "`")
    }
  }
  // Fixed profile grid: name across the top, photo left, address and
  // contacts right.
  let photo = if ctx.cv.profile.photo != none {
    circle(radius: 14mm, image("/" + ctx.cv.profile.photo, width: 28mm, height: 28mm))
  } else {
    box(width: 28mm, height: 28mm, stroke: 0.5pt + silver)[ ]
  }
  let address = if ctx.cv.profile.address != none {
    let a = ctx.cv.profile.address
    let lines = ()
    for field in ("street", "city", "region", "postal_code", "country") {
      if a.at(field, default: none) != none {
        lines.push(a.at(field))
      }
    }
    if lines.len() > 0 { lines.join(" \u{2022} ") } else { none }
  } else {
    none
  }
  grid(
    columns: (30mm, 1fr),
    column-gutter: 1em,
    align(top, photo),
    align(top, {
      text(size: 1.3em, weight: "bold")[#ctx.cv.profile.name]
      v(0.3em)
      if address != none { block(address) }
      v(0.3em)
      for contact in ctx.cv.profile.contacts {
        contact.label
        h(0.6em)
      }
    }),
  )
  v(1em)
  // Fixed line: 1pt rule, then each allowed section with content in a
  // labeled row grid.
  line(length: 100%, stroke: 1pt + ctx.tokens.colors.at("text"))
  v(0.6em)
  for section in ctx.cv.sections {
    if section.entries.len() > 0 {
      grid(
        columns: (30mm, 1fr),
        column-gutter: 1em,
        align(top, text(weight: "bold")[#section.title]),
        align(top, {
          for entry in section.entries {
            block(above: 0.6em, below: 0.6em)[
              #hh.at("rich")(ctx, entry.primary)
              #if entry.period != none [ #entry.period ]
            ]
          }
        }),
      )
      v(0.6em)
    }
  }
}

#let form = (
  document: form-document,
  header: none,
  section: none,
  entry: none,
  section_heading: none,
  entry_details: none,
  // Default placement: the standard kinds. A theme narrows or widens this
  // through layout.allowed.
  allowed: ("education", "experience", "projects", "publications", "skills", "custom", "languages", "certifications"),
)

#let families = (classic: classic, table: table, compact: compact, sidebar: sidebar, form: form)


