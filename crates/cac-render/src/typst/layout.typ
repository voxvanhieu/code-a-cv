// The layout contract (L1): four hooks over pure helpers.
//
// The classic family below carries the embedded classic layout verbatim
// (it was classic_components in base.typ); its typography comes from the
// resolved sheet and the pure helpers arrive through ctx.helpers, which
// base.typ injects. Families and themes may override any hook through
// `layout`. The single dynamic hop is ctx.components.entry: a section
// renders its entries through the active entry hook, so an overridden
// entry hook applies everywhere.

#let helpers-of(ctx) = ctx.helpers

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

// The classic family: centered header, rule under section titles, indented
// entries with the period beside the title. document and header reuse the
// base helpers verbatim via the injected helper table.
#let classic = (document: none, header: none, section: classic-section-flow, entry: classic-entry, section_heading: classic-section-heading, entry_details: classic-entry-details)

#let families = (classic: classic)
