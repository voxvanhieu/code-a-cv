#import "/.cac/base.typ" as base

#let theme = base.extend(
  tokens: (
    fonts: (body: "New Computer Modern", heading: "New Computer Modern"),
  ),
  styles: (
    body: (font_size: 10pt),
    header: (alignment: center),
    heading_1: (font_size: 24pt),
    heading_2: (font_size: 12pt, weight: "regular"),
    heading_3: (font_size: 10pt),
  ),
  page: (paper: "us-letter", margin: 12.7mm),
  components: base.classic_components(),
)
