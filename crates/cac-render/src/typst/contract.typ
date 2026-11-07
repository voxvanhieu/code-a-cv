// The 0.4 layout contract (C3): four hooks over pure helpers.
//
// This file is the compiled reference for the hook signatures, the ctx
// shape, and the structured-date helpers that the base library ships in
// 0.4 (task L1). It imports nothing from cac, so it compiles with plain
// `typst` — editors and `typst watch` can check layout code against real
// signatures without the cac virtual world.
//
// The four hooks replace the twelve late-bound components of 0.3:
//
//   document (ctx, children)  owns the page setup and the whole document
//   header  (ctx)             renders the name block
//   section (ctx, section)    renders one section, dispatching to ctx.layout.entry
//   entry   (ctx, entry)      renders one entry from its named slots
//
// Everything else — rich text, headings, bullets, page-break measurement —
// is style (the resolved sheet) or a pure helper a hook calls explicitly.
// The one dynamic hop is `ctx.layout.entry`, the only place a layout needs
// to interpose between a section and its entries.

// ---------------------------------------------------------------------------
// ctx shape
// ---------------------------------------------------------------------------
//
// ctx.cv      the view (see crates/cac-render/src/view/types.rs):
//             profile (name, headline, photo, address, contacts, summary)
//             and ordered sections; entries carry the named slots
//             kind, title, subtitle, period, date, location, url, fields,
//             highlights, content, tags. Dates are structured dicts, never
//             display strings.
// ctx.style   the resolved style sheet (see crates/cac-render/src/style):
//             page, fonts, colors, text, header, headings, spacing, lists,
//             links, entry, locale. Lengths are numeric values with a unit
//             tag; colors are concrete. Nothing here is evaluated.
// ctx.layout  the active hook table: document, header, section, entry.
//             A hook that overrides `section` still reaches the theme's
//             entry hook through this table — the single dynamic hop.

// ---------------------------------------------------------------------------
// Structured dates and the locale-aware formatter
// ---------------------------------------------------------------------------
//
// A date point is `(y: 2023)` / `(y: 2023, m: 1)` / `(y: 2023, m: 1, d: 15)`
// or the string "present". A period is `(start: point or none, end: point or
// none)`. Formatters receive the sheet's locale; unknown locales fall back
// to English. The Rust renderer shares this vocabulary through
// crates/cac-render/src/view/dates.rs (landing with V1) — month names and
// the "present" spelling must stay in lockstep across the two.

#let months = (
  en: ("January", "February", "March", "April", "May", "June", "July",
       "August", "September", "October", "November", "December"),
  de: ("Januar", "Februar", "März", "April", "Mai", "Juni", "Juli",
       "August", "September", "Oktober", "November", "Dezember"),
  fr: ("janvier", "février", "mars", "avril", "mai", "juin", "juillet",
       "août", "septembre", "octobre", "novembre", "décembre"),
  vi: ("Tháng Một", "Tháng Hai", "Tháng Ba", "Tháng Tư", "Tháng Năm",
       "Tháng Sáu", "Tháng Bảy", "Tháng Tám", "Tháng Chín", "Tháng Mười",
       "Tháng Mười Một", "Tháng Mười Hai"),
  ja: ("1月", "2月", "3月", "4月", "5月", "6月", "7月", "8月", "9月",
       "10月", "11月", "12月"),
)

#let present-word = (
  en: "Present",
  de: "Heute",
  fr: "Aujourd'hui",
  vi: "Nay",
  ja: "現在",
)

#let locale-of(locale) = if locale in months { locale } else { "en" }

// Format one date point: `2023`, `January 2023`, `January 15 2023`, `Present`.
#let format-date(locale, date) = {
  if type(date) == str { return present-word.at(locale-of(locale)) }
  let name = locale-of(locale)
  let parts = ()
  if "m" in date { parts.push(months.at(name).at(date.m - 1)) }
  if "d" in date { parts.push(str(date.d)) }
  // Joining an empty array yields none, not an empty string.
  let rendered = if parts.len() == 0 { "" } else { parts.join(" ") }
  if "y" in date {
    if rendered == "" { return str(date.y) }
    return rendered + " " + str(date.y)
  }
  rendered
}

// Format a period as `start – end`, omitting absent endpoints.
#let format-period(locale, period) = {
  let start = if period.start != none { format-date(locale, period.start) } else { "" }
  let end = if period.end != none { format-date(locale, period.end) } else { "" }
  if start == "" and end == "" { return none }
  if start == "" { return end }
  if end == "" { return start }
  start + " – " + end
}

// ---------------------------------------------------------------------------
// The four hooks
// ---------------------------------------------------------------------------
//
// Signatures are fixed. Stub bodies render minimally; the base library
// replaces them with the pure helpers (entries.dated_row, sections.flow,
// sections.labels, contacts) in L1.

#let document(ctx, children) = {
  // Owns paper, margins, fonts, and footer. The resolved sheet supplies
  // every value; nothing here reads settings.json.
  set page(paper: "a4")
  set text(size: 10pt)
  children
}

#let header(ctx) = {
  // The name block: name, optional headline and photo, contacts. Alignment
  // comes from ctx.style.header.align.
  text(weight: "bold", ctx.cv.profile.name)
}

#let entry(ctx, entry) = {
  // One entry from its named slots. Titles take ctx.style.headings.entry;
  // period placement takes ctx.style.entry.date_position; dates format
  // through format-period with ctx.style.locale. The stub renders plain
  // text nodes; the base library's pure helpers replace this in L1.
  let plain(nodes) = nodes.filter(node => node.kind == "text").map(node => node.text).join(" ")
  if entry.subtitle != none {
    emph(plain(entry.title) + " — " + plain(entry.subtitle))
  } else {
    emph(plain(entry.title))
  }
}

#let section(ctx, section) = {
  // One section. The single dynamic hop: entries render through
  // ctx.layout.entry, so an overridden entry hook applies everywhere.
  // (Typst requires the extra parens to call a value stored in a dict.)
  text(weight: "bold", section.title)
  for entry in section.entries {
    (ctx.layout.entry)(ctx, entry)
  }
}

// The hook table a theme exports as `layout`. `document` may be omitted to
// keep the base document.
#let layout = (
  document: document,
  header: header,
  section: section,
  entry: entry,
)
