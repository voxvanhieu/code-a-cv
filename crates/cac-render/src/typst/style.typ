// The style sheet bridge (S2): the resolved sheet from /.cac/style.json
// becomes the tokens, styles, and page dictionaries the base library
// consumes. Nothing here evaluates a string: lengths arrive as
// (value, unit) pairs, colors as hex, enums as spellings.
//
// The same builder reads the settings layer (/.cac/settings-style.json),
// which is sparse: only the keys settings actually set appear, so the
// legacy-adapter overlay in base.typ cannot clobber keys below it.

#let load() = json("/.cac/style.json")

// A hex color as the resolved sheet carries it. Equal channels become
// luma, matching the grayscale the renderer emitted before the sheet
// existed, so the default palette stays byte-identical.
#let color-of(hex) = {
  let digits = hex.slice(1)
  let channel = slice => int(slice, base: 16)
  let red = channel(digits.slice(0, 2))
  let green = channel(digits.slice(2, 4))
  let blue = channel(digits.slice(4, 6))
  if red == green and green == blue {
    luma(red)
  } else {
    rgb(hex)
  }
}

#let length(pair) = {
  let value = pair.value
  let unit = pair.unit
  if unit == "pt" { value * 1pt }
  else if unit == "mm" { value * 1mm }
  else if unit == "cm" { value * 1cm }
  else if unit == "in" { value * 1in }
  else if unit == "em" { value * 1em }
  // Typst has no rem unit; em carries the same relative meaning here.
  else if unit == "rem" { value * 1em }
  else if unit == "%" { value * 1% }
  else { value * 1pt }
}

#let alignment-of(spelling) = {
  if spelling == "center" { center }
  else if spelling == "right" { right }
  else { left }
}

// A group that may be absent in a sparse layer.
#let group(sheet, name) = sheet.at(name, default: (:))

// The token name for a slot color: a known token when the color matches
// one, otherwise a generated key registered alongside the tokens.
#let color-ref(colors, hex, key) = {
  for name in ("text", "muted", "accent", "rule") {
    if colors.at(name) == hex {
      return name
    }
  }
  key
}

#let colors-contains(colors, color) = {
  for name in ("text", "muted", "accent", "rule") {
    if colors.at(name) == color {
      return true
    }
  }
  false
}

#let heading-slot(slot, color-name) = {
  let dict = (:)
  if slot.at("size", default: none) != none {
    dict.insert("font_size", length(slot.size))
  }
  dict.insert("weight", slot.at("weight", default: "regular"))
  // The styles dictionaries reference colors by token name; the bridge
  // registers non-token colors in the tokens dictionary for them.
  dict.insert("color", color-name)
  dict.insert("space_before", length(slot.at("space_before", default: (value: 0.0, unit: "pt"))))
  dict.insert("space_after", length(slot.at("space_after", default: (value: 0.65, unit: "em"))))
  dict
}

#let tokens-of(sheet) = {
  let fonts = group(sheet, "fonts")
  let colors = group(sheet, "colors")
  (
    fonts: (
      body: fonts.at("body", default: "Libertinus Serif"),
      heading: fonts.at("heading", default: "Libertinus Serif"),
      mono: fonts.at("mono", default: "DejaVu Sans Mono"),
    ),
    colors: (
      text: color-of(colors.at("text", default: "#000000")),
      muted: color-of(colors.at("muted", default: "#aaaaaa")),
      accent: color-of(colors.at("accent", default: "#000000")),
      rule: color-of(colors.at("rule", default: colors.at("accent", default: "#000000"))),
    ),
  )
}

#let styles-of(sheet, tokens) = {
  let text-group = group(sheet, "text")
  let header = group(sheet, "header")
  let links = group(sheet, "links")
  let spacing = group(sheet, "spacing")
  let entry = group(sheet, "entry")
  let lists = group(sheet, "lists")
  let headings = group(sheet, "headings")
  let colors = tokens.colors
  let body = (
    font: "body",
    color: "text",
    weight: "regular",
    justify: text-group.at("align", default: "justified") == "justified",
  )
  if text-group.at("size", default: none) != none {
    body.insert("font_size", length(text-group.size))
  }
  if text-group.at("leading", default: none) != none {
    body.insert("line_spacing", length(text-group.leading))
  }
  if text-group.at("paragraph_gap", default: none) != none {
    body.insert("paragraph_spacing", length(text-group.paragraph_gap))
  }
  let section-slot = headings.at("section", default: (:))
  let name-slot = headings.at("name", default: (:))
  let entry-slot = headings.at("entry", default: (:))
  (
    body: body,
    heading: (
      font: "heading",
      color: "text",
      weight: "bold",
      space_before: 0pt,
      space_after: 0.65em,
    ),
    heading_1: heading-slot(name-slot, color-ref(colors, color-of(name-slot.at("color", default: "#000000")), "heading-name")),
    heading_2: heading-slot(section-slot, color-ref(colors, color-of(section-slot.at("color", default: "#000000")), "heading-section")),
    heading_3: heading-slot(entry-slot, color-ref(colors, color-of(entry-slot.at("color", default: "#000000")), "heading-entry")),
    heading_4: (:),
    heading_5: (:),
    list: (
      item_spacing: if spacing.at("item", default: none) != none { length(spacing.item) } else { auto },
    ),
    header: (alignment: alignment-of(header.at("align", default: "left"))),
    link: (underline: links.at("underline", default: true)),
    section: (
      space_before: length(spacing.at("section", default: (value: 1.2, unit: "em"))),
      space_after_heading: length(spacing.at("heading_gap", default: (value: 0.65, unit: "em"))),
    ),
    entry: (
      space_after: length(spacing.at("entry", default: (value: 1.2, unit: "em"))),
      // The sheet speaks keep_together; the renderer speaks page breaks.
      allow_page_break: not entry.at("keep_together", default: true),
    ),
    highlight: (bullet: text(lists.at("bullet", default: "•"))),
    footer: (color: "muted"),
  )
}
#let page-of(sheet) = {
  let page = group(sheet, "page")
  let margins = group(page, "margin")
  let top = margins.at("top", default: (value: 18.0, unit: "mm"))
  let right = margins.at("right", default: top)
  let bottom = margins.at("bottom", default: top)
  let left = margins.at("left", default: top)
  let margin = if top == right and right == bottom and bottom == left {
    length(top)
  } else {
    (top: length(top), right: length(right), bottom: length(bottom), left: length(left))
  }
  (paper: page.at("paper", default: "a4"), margin: margin)
}



// The settings layer arrives sparse: only the keys settings actually set
// are present in the JSON (skip_serializing), so this builder includes a
// key only when its value exists. The result is merged over the theme in
// base.typ, preserving the 0.3 precedence (settings beat themes) without
// letting defaults clobber the layers beneath.
#let dictionaries-sparse(sheet) = {
  let tokens = (:)
  let fonts = group(sheet, "fonts")
  if fonts != (:) {
    let fonts-dict = (:)
    for name in ("body", "heading", "mono") {
      if fonts.at(name, default: none) != none {
        fonts-dict.insert(name, fonts.at(name))
      }
    }
    if fonts-dict != (:) {
      tokens.insert("fonts", fonts-dict)
    }
  }
  let colors = group(sheet, "colors")
  if colors != (:) {
    let colors-dict = (:)
    for name in ("text", "muted", "accent", "rule") {
      if colors.at(name, default: none) != none {
        colors-dict.insert(name, color-of(colors.at(name)))
      }
    }
    if colors-dict != (:) {
      tokens.insert("colors", colors-dict)
    }
  }
  let styles = (:)
  let text-group = group(sheet, "text")
  if text != (:) {
    let body = (:)
    if text-group.at("size", default: none) != none { body.insert("font_size", length(text-group.size)) }
    if text-group.at("leading", default: none) != none { body.insert("line_spacing", length(text-group.leading)) }
    if text-group.at("paragraph_gap", default: none) != none { body.insert("paragraph_spacing", length(text-group.paragraph_gap)) }
    if text-group.at("align", default: none) != none { body.insert("justify", text-group.align == "justified") }
    if body != (:) {
      styles.insert("body", body)
    }
  }
  let header = group(sheet, "header")
  if header != (:) {
    let header-dict = (:)
    if header.at("align", default: none) != none {
      header-dict.insert("alignment", alignment-of(header.align))
    }
    if header-dict != (:) {
      styles.insert("header", header-dict)
    }
  }
  let links = group(sheet, "links")
  if links.at("underline", default: none) != none {
    styles.insert("link", (underline: links.underline))
  }
  let lists = group(sheet, "lists")
  let spacing = group(sheet, "spacing")
  if lists.at("bullet", default: none) != none {
    // The 0.3 shim carried the bullet as a plain string; keep that shape.
    styles.insert("highlight", (bullet: lists.bullet))
  }
  if spacing.at("item", default: none) != none {
    let list-dict = styles.at("list", default: (:))
    list-dict.insert("item_spacing", length(spacing.item))
    styles.insert("list", list-dict)
  }
  let headings = group(sheet, "headings")
  for (key, slot-name) in (
    ("name", "heading_1"),
    ("section", "heading_2"),
    ("entry", "heading_3"),
  ) {
    let slot = headings.at(key, default: none)
    if slot != none {
      let slot-dict = heading-slot(slot, color-ref(tokens.colors, color-of(slot.at("color", default: "#000000")), "heading-" + key))
      styles.insert(slot-name, slot-dict)
    }
  }
  if spacing != (:) {
    let section-dict = (:)
    if spacing.at("section", default: none) != none {
      section-dict.insert("space_before", length(spacing.section))
    }
    if spacing.at("heading_gap", default: none) != none {
      section-dict.insert("space_after_heading", length(spacing.heading_gap))
    }
    if section-dict != (:) {
      styles.insert("section", section-dict)
    }
    if spacing.at("entry", default: none) != none {
      styles.insert("entry", (space_after: length(spacing.entry)))
    }
  }
  let pagination-entry = group(sheet, "entry")
  if pagination-entry.at("keep_together", default: none) != none {
    let entry-dict = styles.at("entry", default: (:))
    entry-dict.insert("allow_page_break", not pagination-entry.keep_together)
    styles.insert("entry", entry-dict)
  }
  let page = group(sheet, "page")
  let page-dict = (:)
  if page.at("paper", default: none) != none {
    page-dict.insert("paper", page.paper)
  }
  if page.at("margin", default: none) != none {
    let margins = page.margin
    if margins.at("top", default: none) != none and margins.top == margins.at("right", default: margins.top) and margins.top == margins.at("bottom", default: margins.top) and margins.top == margins.at("left", default: margins.top) {
      page-dict.insert("margin", length(margins.top))
    } else {
      let margin-dict = (:)
      for side in ("top", "right", "bottom", "left") {
        if margins.at(side, default: none) != none {
          margin-dict.insert(side, length(margins.at(side)))
        }
      }
      page-dict.insert("margin", margin-dict)
    }
  }
  (tokens: tokens, styles: styles, page: page-dict)
}

// The three dictionaries in the shape base.extend() documents.
#let dictionaries(sheet) = {
  let tokens = tokens-of(sheet)
  let headings = group(sheet, "headings")
  // Register slot colors that are not token names so their style entries
  // can reference them.
  for (key, slot) in (
    ("heading-name", headings.at("name", default: (:))),
    ("heading-section", headings.at("section", default: (:))),
    ("heading-entry", headings.at("entry", default: (:))),
  ) {
    if slot.at("color", default: none) != none {
      let resolved = color-of(slot.color)
      if colors-contains(tokens.colors, resolved) == false {
        tokens.colors.insert(key, resolved)
      }
    }
  }
  (tokens: tokens, styles: styles-of(sheet, tokens), page: page-of(sheet))
}
