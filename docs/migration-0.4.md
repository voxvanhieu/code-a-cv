# Migrating from 0.3 to 0.4

Version 0.4 rebuilds the Markdown grammar, resolves styling in Rust, and
retires the legacy theme wiring. Almost everything keeps working unchanged —
legacy settings and 0.3 themes render through compatibility layers for one
release — and the sections below list what to check and what changed
behavior to expect.

## Markdown documents

**Reformatting is now safe.** Field blocks (`Email:`, `Organization:`,
`Period:`, …) are recognized by content rather than by adjacency to a
heading. Blank lines around them carry no meaning, so Prettier- and
markdownlint-formatted files build the same CV. Two behavior changes to
review:

- Prose that starts with a number is no longer mistaken for a date
  (0.3 rejected such lines as invalid dates).
- Undeclared `Key:` values under a section or entry become **open
  fields** instead of errors. They ride through the model and render in
  the default layouts. Declare them in a field pack
  (`cac check --pack fields.toml`) to type and require them.
- `Summary:` and `Contact:` now work as profile fields; a bare
  `mailto:` line under the name is contact sugar, not an error.
- YAML front matter is accepted for `locale`, `packs`, and profile
  fields.

`cac fmt` gains `--style explicit|compact|preserve`. The default
`preserve` keeps the spelling your file used; `explicit` spells every
field; `compact` uses heading/date sugar. Both styles parse to the same
document, and formatted output is a Prettier/markdownlint no-op.

## Settings

Every 0.3 style key still works: `typography`, `spacing`, `pagination`,
`page`, and the legacy leaves of `style` (`accent_color`,
`body_alignment`, `header_alignment`, `link_underline`,
`highlight_bullet`). Each one in use prints a `NOTE` naming its
replacement in the style sheet — for example `typography.font_size`
becomes `text.size` and `pagination.allow_entry_page_break` becomes
`entry.keep_together` (the boolean flips). The new spelling is the
`[style]` table documented in
[`schema/style.schema.json`](../crates/cac-render/schema/style.schema.json);
the resolved sheet both renderers read is `.cac/style.json`.

New: `document_date` (YYYY-MM-DD) gives Typst's `today()` a fixed date
so builds that reference it stay reproducible.

## Themes

- `cac theme init` writes real files under `.cac/` (base.typ,
  style.typ, layout.typ, main.typ) so `typst watch .cac/main.typ` and
  editor extensions preview the theme outside `cac`.
- The `themeProject` marker in settings.json is no longer written or
  required; a theme project is identified by its directory under
  `.cac/themes` and `settings.theme`.
- A 0.3 theme exporting `components` (including
  `base.classic_components(...)`) keeps rendering through an adapter
  for one release, and prints a deprecation note. Migrate to layout
  hooks: export `layout` instead — see the
  [shared theme contract](development/shared-theme-contract.md).
- Built-in families (classic, table, compact, sidebar, form) are
  selectable with `layout: (family: "...")`. Form themes declare
  `requires` and `allowed` and reject unsupported sections by name; the
  registry lists them separately from universal themes.

## Registry and packs

- `file://` registry URLs work on Windows (drive letters and
  percent-encoded paths).
- Open fields can be typed and required through field packs:
  `cac check --pack fields.toml` — see the
  [theme contract appendix](authoring/themes.md) for the TOML shape.

## Rendering

- Builds referencing `today()` should set `document_date`; the world's
  clock otherwise returns nothing and such code fails.
- Pages whose content extends beyond the page edge print a `NOTE`
  naming the page — layout bugs surface instead of clipping silently.
- The `leaves` and `shapes` registry themes are decorative: their
  corner artwork is PDF-only; HTML keeps the sheet colors.
