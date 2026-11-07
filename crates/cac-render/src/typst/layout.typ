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

#let families = (classic: classic, table: table, compact: compact)
