# Your first theme in ten minutes

A theme changes how your CV looks. This tutorial builds one in ten minutes:
pick a layout family, set colors and fonts, and render it. No Rust, no
Typst knowledge required for the common case — and when you *do* want
more, the escape hatches are named.

Prerequisites: `cac` installed, and Node.js only if you want to run the
formatter checks at the end.

## 1. Start a theme project (2 minutes)

```sh
mkdir my-cv && cd my-cv
cac theme init
```

Answer the prompts (theme name, your name, email). You now have:

```
.cac/
  base.typ          the layout library (helpers the hooks call)
  style.typ         the style-sheet bridge (sheet → dictionaries)
  layout.typ        the built-in families (classic, table, compact, sidebar)
  main.typ          a root document rendering your CV
  themes/my-theme/  your theme: theme.json + theme.typ
cv.md               the starter CV
settings.json       selects your theme
```

Because `.cac/` holds **real files**, editors with a Typst extension and
`typst watch .cac/main.typ` preview the theme outside `cac`.

## 2. Write your CV in FieldMark (3 minutes)

Edit `cv.md`. The starter shows the shapes: `## Section` headings, `###
Role, Organization` entries, labeled fields (`Period:`, `Location:`),
and `-` bullets for highlights. Full grammar:
[the FieldMark guide](fieldmark.md).

## 3. Make it yours with a style sheet (3 minutes)

Create `my-style.toml` and point `settings.json` at it… actually — the
simplest styling has no extra file at all: open `.cac/themes/my-theme/theme.typ`
and set a family plus overrides. Two supported shapes:

**Pick a family and override style keys** (recommended):

```typst
#import "/.cac/base.typ" as base

#let theme = base.extend(
  tokens: (colors: (accent: rgb("1f4e79"))),
  styles: (header: (alignment: center)),
)
```

**Or select a built-in family** (classic, table, compact, sidebar):

```typst
#let theme = (layout: (family: "sidebar"))
```

Render and look:

```sh
cac build --format pdf,html
```

The HTML reads the same style sheet as the PDF, so what you tuned in the
sheet — fonts, colors, spacing — shows up in both.

## 4. Check it (1 minute)

```sh
cac check
```

`cac check` validates content and (with `--pack fields.toml`) your
open fields against a declaration file. Warnings are advisory; errors
fail.

## Where to go next

- **More layouts**: `layout: (family: "table")` for table rows, or
  `"compact"` for a tight one-page look.
- **Colors and fonts**: every style key is listed in the
  [style sheet schema](https://raw.githubusercontent.com/voxvanhieu/code-a-cv/main/schema/style.schema.json)
  — the same table `theme.toml` and `settings.json` `[style]` accept.
- **Deeper changes**: override one of the four layout hooks (document,
  header, section, entry) — see the [shared theme contract](../development/shared-theme-contract.md).
- **Reference**: the [theme contract appendix](themes.md) covers
  obligations, the complete view, and theme-defined sections.
