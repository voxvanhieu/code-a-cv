#import "/.cac/base.typ" as base

// The rirekisho theme: a JIS-style 履歴書 form built on the form family.
// CJK glyphs come from the theme's fonts/ directory; dates render in 和暦
// (Reiwa) computed against the document date setting; the fixed grid and
// allowed/requires lists follow the form contract.

#let jp = "Noto Sans JP"

// 和暦 comes from the view, not from era maths here: `era_period` carries
// the formatter's spellings (era boundaries by month, 元年, 西暦 before
// Meiji), so this theme keeps no second era table.

#let label-cell(text) = align(top, text(size: 0.85em, fill: rgb("444444"))[#text])

#let row(body) = grid(columns: (28mm, 1fr), column-gutter: 1em, align(top, body))

// The fixed grid: 平成/令和 era column omitted for space, standard
// 履歴書 rows. Sections the form cannot place are rejected by name.
#let rirekisho-document(ctx) = {
  let layout = ctx.layout
  let h = ctx.helpers
  let requires = layout.at("requires", default: (:))
  let allowed = layout.at("allowed", default: requires.at("sections", default: ()))
  let body = ctx.styles.body
  set document(title: "履歴書 — " + ctx.cv.profile.name, author: ctx.cv.profile.name)
  set page(paper: ctx.page.paper, margin: (top: 14mm, bottom: 14mm, left: 12mm, right: 12mm))
  set text(font: (name: jp), size: body.font_size, fill: rgb("111111"))
  set par(leading: body.line_spacing, spacing: body.paragraph_spacing)

  // Profile requirements: photo and furigana are the form's identity block.
  for key in requires.at("profile", default: ()) {
    if key == "photo" and ctx.cv.profile.photo == none {
      panic("this form requires profile `photo`")
    }
  }
  let furigana = ctx.cv.profile.fields.at("furigana", default: none)
  if furigana == none {
    panic("this form requires profile `furigana` (フリガナ)")
  }

  // Rejection: sections the form cannot place fail by name.
  for section in ctx.cv.sections {
    let kind = section.kind
    let placed = allowed.contains(kind) or allowed.contains(section.id)
    if not placed {
      panic("this form does not support the section `" + section.title + "` (kind `" + kind + "`); it accepts: " + allowed.join(", "))
    }
  }

  // 氏名 block: furigana line, name, photo cell right.
  let photo = if ctx.cv.profile.photo != none {
    circle(radius: 16mm, image("/" + ctx.cv.profile.photo, width: 32mm, height: 32mm))
  } else {
    box(width: 32mm, height: 32mm, stroke: 0.5pt + rgb("888888"))[ ]
  }
  grid(
    columns: (1fr, 34mm),
    column-gutter: 1em,
    align(top, {
      text(size: 0.8em)[フリガナ: #furigana]
      v(0.1em)
      text(size: 1.5em, weight: "bold")[#ctx.cv.profile.name]
      v(0.3em)
      if ctx.cv.profile.address != none {
        let a = ctx.cv.profile.address
        text(size: 0.9em)[
          #if a.at("postal_code", default: none) != none [〒#a.at("postal_code") ]
          #if a.at("city", default: none) != none [#a.at("city") ]
        ]
      }
    }),
    align(top, photo),
  )
  v(0.5em)
  line(length: 100%, stroke: 1pt + rgb("111111"))
  v(0.8em)

  // 日付: the 和暦 spelling of the document_date setting, computed by the
  // renderer (deterministic builds never read the clock). Without a
  // document date the line is omitted instead of failing the build.
  let dated = json("/.cac/document-date.json")
  if dated.at("era", default: none) != none {
    align(right, text(size: 0.9em)[#dated.era現在])
    v(0.6em)
  }

  // Sections as labeled fixed rows.
  for section in ctx.cv.sections {
    if section.entries.len() == 0 { continue }
    block(above: 0.8em, below: 0.4em)[
      #text(size: 1.05em, weight: "bold")[#section.title]
    ]
    line(length: 100%, stroke: 0.5pt + rgb("111111"))
    for entry in section.entries {
      v(0.4em)
      grid(
        columns: (22mm, 1fr),
        column-gutter: 1em,
        align(top, {
          // The formatter's 和暦 spelling of the whole period.
          entry.at("era_period", default: none)
        }),
        align(top, {
          h.at("rich")(ctx, entry.primary)
          for highlight in entry.at("highlights", default: ()) {
            block[#h.at("rich")(ctx, highlight)]
          }
        }),
      )
      v(0.3em)
    }
    line(length: 100%, stroke: 0.5pt + rgb("999999"))
  }
}

#let theme = (
  layout: (
    family: "form",
    document: rirekisho-document,
    requires: (sections: ("education", "certifications", "languages"), profile: ("photo",)),
    allowed: ("education", "certifications", "languages"),
  ),
)
