#import "/.cac/base.typ" as base

// English presentation labels; section titles never select a layout.
#let labels = (
  experience: (period: "Period", details: "Role and organization"),
  education: (period: "Period", details: "Qualification and institution"),
  project: (period: "Period", details: "Project"),
  publication: (period: "Date", details: "Publication"),
)

#let section(ctx, section) = {
  let kind = base.table_kind(section)
  if kind == "skill-group" { base.section_labels(ctx, section) }
  else if kind != none { base.section_table(ctx, section, headers: labels.at(kind)) }
  else { base.section_flow(ctx, section) }
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
