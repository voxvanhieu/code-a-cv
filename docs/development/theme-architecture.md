# Theme Architecture

## Goals

The rendering architecture should provide a complete default CV while allowing users to customize it at two levels:

* Use `settings.json` to identify the root CV and set common formatting values
* Use a Typst theme for structural and advanced visual customization
* Build without configuration by using the embedded `classic` theme
* Produce the same result without depending on fonts installed on the operating system

## Rendering layers

Formatting is resolved in this order:

```text
base.typ < selected theme < settings.json
```

The Typst theme model separates three kinds of formatting:

* Design tokens name the font families and colors shared by semantic styles
* Semantic styles define body text, heading levels, lists, sections, entries, and the footer
* Page configuration defines physical page properties such as paper and margins

`base.typ` contains the complete definitions for all three layers and every component. The selected theme recursively overrides only the values it needs. `settings.json` then applies a limited set of final user overrides.

`settings.json` is the final authority for every value it supports. The renderer maps its grouped rendering properties into tokens, styles, and page configuration before invoking components.

## Project layout

A CV project can contain:

```text
my-cv/
├── cv.md
├── settings.json
└── .cac/
    ├── settings.schema.json
    └── themes/
        └── oxford/
            ├── theme.typ
            ├── assets/
            └── fonts/
```

Only the CV source file is required. When `settings.json` is absent, `cac build` reads `cv.md` by default and uses the embedded `classic` theme and its defaults.

## Settings

The selected theme is stored in `settings.json`:

```json
{
  "$schema": ".cac/settings.schema.json",
  "root": "cv.md",
  "theme": "oxford",
  "page": {
    "paper": "a4",
    "margin": "18mm",
    "margins": { "top": "16mm", "bottom": "16mm" }
  },
  "typography": {
    "font": "Libertinus Serif",
    "heading_font": "Libertinus Serif",
    "font_size": "10pt",
    "line_spacing": "0.65em"
  },
  "style": {
    "accent_color": "#1f4e79",
    "body_alignment": "justified",
    "link_underline": true,
    "header_alignment": "center",
    "highlight_bullet": "•"
  },
  "spacing": {
    "list_item": "0.65em",
    "section": "1.2em",
    "section_heading": "0.65em",
    "entry": "1.2em"
  },
  "pagination": {
    "allow_entry_page_break": false
  }
}
```

`cac init` and `cac build` synchronize `.cac/settings.schema.json` from the
schema embedded in the running binary. Run `cac schema` to refresh it explicitly
and validate the current `settings.json`. The `$schema` property enables editor
completion and validation; `cac` remains the runtime authority for settings.

The supported settings should remain small and stable:

| Setting | Purpose |
|---|---|
| `root` | Select the CV file used when `cac build` has no explicit input |
| `theme` | Select a theme by name |
| `page.paper` | Set the page format, such as `a4` or `us-letter` |
| `page.margin` | Set a uniform page margin |
| `page.margins` | Override individual `top`, `bottom`, `left`, or `right` page margins |
| `typography.font` | Set the body and heading font families |
| `typography.heading_font` | Override the heading font family |
| `typography.font_size` | Set the body text size |
| `typography.line_spacing` | Set the clear leading added between line boxes; Typst's default is `0.65em` |
| `style.accent_color` | Set the theme accent to a six-digit hex color |
| `style.body_alignment` | Set body text alignment to `left` or `justified` |
| `style.link_underline` | Show or hide link underlines |
| `style.header_alignment` | Align the header to `left`, `center`, or `right` |
| `style.highlight_bullet` | Set the marker used by the shared highlight-list component |
| `spacing.list_item` | Set spacing between bullet or numbered-list items |
| `spacing.section` | Set spacing between sections |
| `spacing.section_heading` | Set the gap after a section heading |
| `spacing.entry` | Set spacing between CV entries |
| `pagination.allow_entry_page_break` | Allow an entry to continue on the next page |

`root` must be a file name in the same directory as `settings.json`. An explicit input such as `cac build cv.json` takes precedence. When `root` is absent, `cac build` reads `cv.md` for compatibility.

Missing formatting properties inherit from the selected theme. Unknown properties and invalid values must produce clear errors instead of being ignored. Per-heading sizes and spacing remain theme properties because settings intentionally describe only the common document defaults.

`page.margins` is applied after `page.margin`. Unspecified sides keep the uniform margin or the selected theme's corresponding margin. Component-related settings affect themes that use the shared styles and components. A custom component can intentionally implement different behavior.

Advanced customization belongs in `theme.typ`. This includes header layouts, date-column widths, section rules, and component-specific typography.

## Theme storage

Themes can be installed for one project or for the current user.

Project-local theme:

```text
<project>/.cac/themes/<theme-name>/theme.typ
```

User-global theme:

```text
~/.cac/themes/<theme-name>/theme.typ
```

On Windows, `~` represents the user profile directory.

Each theme uses its own directory so that it can include assets and fonts:

```text
oxford/
├── theme.typ
├── assets/
└── fonts/
```

Theme names should contain only lowercase ASCII letters, numbers, hyphens, and underscores. This keeps theme identifiers portable and prevents path traversal.

## Theme resolution

Given this setting:

```json
{
  "theme": "oxford"
}
```

`cac` resolves the theme in this order:

1. `<project>/.cac/themes/oxford/theme.typ`
2. `~/.cac/themes/oxford/theme.typ`
3. An embedded theme named `oxford`
4. A clear theme-not-found error

A project-local theme has the highest priority because it can be committed with the CV and used consistently in automated builds. A user-global theme is convenient but can produce different output on another computer.

When `settings.json` does not contain `theme`, the selected theme is `classic`.

## Typst structure

The embedded renderer should use this layout:

```text
crates/cac-render/src/typst/
├── main.typ
├── base.typ
└── themes/
    └── classic.typ
```

### `base.typ`

`base.typ` is the stable parent implementation and the complete theme contract. It contains:

* All default design tokens, semantic styles, and page properties
* Explicit defaults for line, paragraph, heading, list, section, and entry spacing
* Complete style keys for heading levels 1 through 5
* Rich-text rendering
* Neutral, full-width header, section, entry, and list components
* Recursive token, style, page, and component merging
* Translation from the flat user settings into the three theme layers
* The public theme extension API
* The final document rendering function

Its conceptual interface is:

```typst
#let default-tokens = (
  fonts: (
    body: "Libertinus Serif",
    heading: "Libertinus Serif",
    mono: "DejaVu Sans Mono",
  ),
  colors: (text: black, muted: gray, accent: black),
)

#let default-styles = (
  body: (
    font_size: 10pt,
    line_spacing: 0.65em,
    paragraph_spacing: 1.2em,
  ),
  heading: (
    weight: "bold",
    space_before: 0pt,
    space_after: 0.65em,
  ),
  heading_1: (space_after: 1.2em),
  heading_2: (:),
  heading_3: (:),
  heading_4: (:),
  heading_5: (:),
  list: (item_spacing: auto),
  header: (alignment: left),
  link: (underline: true),
  section: (
    space_before: 1.2em,
    space_after_heading: 0.65em,
  ),
  entry: (space_after: 1.2em, allow_page_break: false),
  highlight: (bullet: [•]),
)

#let default-page = (
  paper: "a4",
  margin: 18mm,
)

#let render(cv, theme, settings) = {
  let tokens = merge(default-tokens, theme.tokens)
  let styles = merge(default-styles, theme.styles)
  let page = merge(default-page, theme.page)
  // Apply settings and invoke the resolved document component.
}
```

The real merge implementation merges dictionaries recursively. Scalars and arrays replace inherited values. The shared `heading` style inherits body typography, including line and paragraph spacing, then each `heading_N` style inherits the result. Vertical layout properties are independent: changing line leading does not silently change paragraph, heading, section, or entry gaps. Automatic list spacing follows the active paragraph leading for tight lists. A child theme can override any property without redefining the others.

### `classic.typ`

`classic.typ` adapts the centered `default` template from `cv-cli` and is the
only embedded theme. Downloadable themes, including `classic-left`, inherit
the complete theme contract from `base.typ` and override only their typography,
page settings, header, section heading, and entry layout.

Conceptually:

```typst
#import "../base.typ" as base

#let theme = base.extend(
  styles: (...),
  page: (...),
  components: (...),
)
```

### Custom themes

Every installed theme must export a value named `theme` from `theme.typ`:

```typst
#import "/.cac/base.typ" as base

#let section-heading(ctx, section) = (ctx.components.heading)(ctx, 2, [
  #section.title
])

#let theme = base.extend(
  tokens: (
    colors: (accent: rgb("1f4e79")),
  ),
  styles: (
    body: (paragraph_spacing: 0.9em),
    heading_1: (
      line_spacing: 0.8em,
      space_before: 0.4em,
      space_after: 0.3em,
    ),
    heading_5: (line_spacing: 1.2em),
    section: (
      space_before: 4mm,
      space_after_heading: 0.5em,
    ),
  ),
  page: (margin: 16mm),
  components: (
    section_heading: section-heading,
  ),
)
```

A custom theme inherits every token, style, page property, and component it does not replace. The default renderer maps the CV name to `heading_1`, section titles to `heading_2`, and entry headings to `heading_3`. Custom components can use levels 4 and 5 through the shared `heading` component.

In Markdown custom sections, a level-three heading and its following bullets form one entry. The renderer passes the heading as `entry.primary` and the bullets as `entry.highlights`, matching the structure used by typed sections. Consecutive bullets written directly under a section are standalone text entries and render as one list. The `entry.kind` field lets child themes preserve or replace this distinction.

## Component interface

The base renderer should expose a small component interface:

```text
document(ctx)
header(ctx)
summary(ctx)
section(ctx, section)
section_heading(ctx, section)
entry(ctx, entry)
highlight_list(ctx, items)
footer(ctx)
heading(ctx, level, body)
styled_text(ctx, style, body)
rich(ctx, nodes)
```

Each component receives a context containing the CV and every resolved theme layer:

```typst
(
  cv: cv,
  tokens: resolved-tokens,
  styles: resolved-styles,
  page: resolved-page,
  components: resolved-components,
)
```

Components should use `heading` and `styled_text` so that semantic style inheritance remains consistent. This avoids relying on global Typst rules whose effects become unclear across parent and child themes.

The base components use ordinary full-width document flow and generate a
centered current and total page footer. They do not reserve date columns, draw
section rules, center profile content, or italicize metadata. Child themes own
those presentation decisions and can replace individual components when style
values alone are insufficient.

## Fonts and assets

The embedded renderer should include a portable default font. It must not require a matching system font.

Fonts in a theme's `fonts/` directory should be loaded into the Typst font store before compilation. Assets should resolve relative to `theme.typ`, allowing a theme to use paths such as:

```typst
#image("assets/logo.png")
```

The renderer should expose only the selected theme directory and internal renderer files to Typst. Paths that escape these roots must be rejected.

## Command behavior

The normal command remains:

```console
cac build
```

It automatically reads `settings.json` beside the CV source when that file exists. An alternative settings file can be selected with:

```console
cac build --settings settings-print.json
```

Theme selection remains in the settings file. There is no separate `--theme` option, which prevents two sources from selecting different themes.

The build should report the selected theme and its source:

```text
THEME oxford (project)
BUILT offering/cv.pdf
```

Possible theme-management commands are:

```console
cac theme list
cac theme search
cac theme search blue
cac theme info classic-blue
cac theme install oxford
cac theme install oxford --local
cac theme remove oxford
cac theme remove oxford --local
```

`classic` is embedded and available without installation. Downloadable names, including `classic-left`, are resolved through `themes/index.json`; `cac` then downloads the theme's `theme.json` manifest and files from the repository, verifies their SHA-256 checksums, and installs them. The default install location is `~/.cac/themes`. The `--local` option installs into `<project>/.cac/themes`, and `--force` replaces an existing installation. The system names `classic`, `base`, and `main` cannot be installed.

The contribution workflow and manifest contract are documented in [`themes/README.md`](../../themes/README.md). Contributors add a directory under `themes/`, list its searchable metadata in `themes/index.json`, verify every file checksum, and open a pull request. Registry themes inherit the bundled `/.cac/base.typ`; parent themes do not need to be downloaded or inspected.

## Implementation boundaries

The current monolithic Typst template should be divided without adding formatting data to the CV content model:

* `crates/cac-render/src/typst/base.typ` owns defaults, shared rendering, merging, and the component API
* `crates/cac-render/src/typst/themes/classic.typ` owns the default visual presentation
* `crates/cac-render/src/typst/main.typ` loads normalized CV data, the selected theme, and user settings
* `crates/cac-render/src/pdf.rs` resolves virtual Typst files, theme assets, and theme fonts
* `crates/cac-render/src/settings.rs` defines and validates the supported settings
* `crates/cac/src/commands/build.rs` locates `settings.json` and passes render options to `cac-render`
* `crates/cac/src/commands/theme.rs` lists and manages embedded, user-global, and project-local themes

`cac-core` remains concerned only with CV content. Theme selection and formatting settings belong to the rendering boundary.

## Compatibility and validation

Theme API 1 defines the `tokens`, `styles`, `page`, and `components` dictionaries. Theme API changes must produce an actionable compatibility error.

The implementation should test:

* Building with no settings or custom theme
* Selecting the embedded `classic` theme by default
* Applying settings after theme defaults
* Resolving project-local themes before user-global and embedded themes
* Loading theme-relative assets and fonts
* Rejecting invalid theme names and paths outside allowed roots
* Reporting missing themes, invalid settings, and unsupported theme API versions
* Producing reproducible PDFs from project-local themes
