# Shared theme contract

Themes consume one complete CV data schema and follow one styling contract.
The [FieldMark authoring contract](../authoring/fieldmark.md) defines how user input maps to
that schema. Optional source fields may be omitted; the complete view below
always supplies keys, not fabricated values.
Import `/.cac/base.typ` and export `theme = base.extend(...)`, or export a
plain dictionary containing the overrides you need. `tokens`, `styles`, `page`,
and `components` are optional dictionaries; omitted values inherit the shared
base defaults. Invalid dictionary types produce a field-specific rendering error.

The renderer supplies CV data through `ctx.cv` and applies user settings before
calling components. Layouts may be arbitrarily decorative, but must preserve
supported content and honor the resolved styling settings described below.
Compatibility is demonstrated by rendering the shared corpus and checking content
and settings behavior. No version declaration is required in the theme or manifest.
Existing manifests should remove the obsolete version field; package metadata
continues to reject unknown fields. Custom components should tolerate optional
fields and use generic fallbacks for unfamiliar entry kinds.

## Author obligations

A shared theme presents arbitrary supported CV content. Select layouts using
semantic entry kinds and field presence, preserve source section and entry order,
and display every populated visible slot exactly once. Names, titles, employers,
and wording are display content. IDs, tags, and kinds are metadata and need not
be printed. Do not fabricate facts, silently filter tags, or rewrite content to
fit a layout. Repeated decorative column headings and page headers are allowed.

Mixed or unfamiliar content must use a complete generic fallback. Empty sections
still display their headings. Optional fields must not produce invented values
or meaningless empty columns. Full document and section overrides inherit these
obligations. A project-specific theme may instead document assumptions, select
stable IDs, or reject unsupported content with a useful diagnostic. Installation
location does not determine whether a theme is shared.

Common settings remain final authority for applicable elements. Use resolved
styles and child components for body and heading typography, links, bullets,
alignment, spacing, margins, and pagination. Every theme must follow this styling
contract, including custom decorative layouts. A layout that cannot apply an
explicit applicable setting must report the limitation. Theme-specific widths
remain Typst options.

## Complete view

`ctx.cv` contains `profile` and ordered `sections`. All keys below are present;
optional values are `none`, collections may be empty. Strings and rich nodes are
data, never evaluated as Typst source. Tags are intentionally absent.

| View | Fields |
|---|---|
| Profile | `name: string`, `summary: rich or none`, `contacts: array` |
| Contact | `kind: string`, `label: string`, `href: string or none` |
| Section | `id: string`, `title: string`, `kind: string`, `entries: array` |
| Entry | `kind: string`, `primary: rich`, `secondary: rich or none`, `period: string or none`, `metadata: array`, `highlights: array of rich` |
| Metadata | `role: string`, `body: rich` |

Contacts are ordered email, phone, location, website, omitting absent values.
Email and phone targets use `mailto:` and `tel:`; websites retain their URL;
locations have no target. Each label displays the corresponding source value.
Additional `profile.contacts` follow these legacy contacts in source order, keeping
their labels and optional targets. Their view kinds are derived from the target
(email for `mailto:`, phone for `tel:`, website for web links, custom without a target).

Built-in section kinds are `experience`, `education`, `projects`, `publications`,
`skills`, and `custom`; additional theme-defined identifiers are preserved. Entry kinds use `experience`, `education`, `project`, `publication`,
`skill-group`, `custom`, `text`, and `prose`. These vocabularies are distinct.

| Entry kind | Primary | Secondary | Period | Metadata | Highlights |
|---|---|---|---|---|---|
| experience | role | organization | date range | location | highlights |
| education | qualification | institution | date range | empty | highlights |
| project | name | none | date range | linked URL | highlights |
| publication | title | publisher | date | linked URL | highlights |
| skill-group | name | none | none | empty | skills |
| custom | heading | none | date range | empty | highlights |
| text | first body | none | none | empty | subsequent adjacent text bodies |
| prose | complete rich body | none | none | empty | empty |

Empty secondary rich text becomes `none`. Dates use the existing English display
formatter; ranges use `start – end`. Unknown endpoints remain empty, distinct from
`Present`. Experience, education, project, and custom entries may supply a single
date instead of a range. Metadata roles are `location`, `url`, and `description`.
An entry's optional ordered rich `content` becomes a final `description` metadata
item, following location/URL. It is an alternative to highlights/skills, so prose
and lists retain their source order without duplication.
URL bodies contain a link node whose visible label and target are the source URL.
Adjacent text entries retain their existing grouping as a single list; a custom
heading with highlights remains a heading followed by a list.

Rich text is an ordered array of tagged nodes: `text` and `code` have `text`;
`emph`, `strong`, and `paragraph` have rich `body`; `link` has `href` and rich
`body`; `break` represents an explicit line break; `list` has `start: integer or
none` and `items: array of rich`. An absent list start means unordered; an integer
preserves ordered-list numbering. Nested list items are rich content. Use
`ctx.components.rich` to retain literal characters and links. Paragraph and list nodes must retain their block structure and use resolved
body/list spacing and bullets. Prose entries use body typography without an
artificial heading or bullet. New recognized
entry types must populate generic slots; older parsers still reject unknown
source enum variants.

## Helpers and extension points

```typst
#import "/.cac/base.typ" as base
#let section(ctx, section) = {
  if base.table_kind(section) == "skill-group" {
    base.section_labels(ctx, section)
  } else {
    base.section_table(ctx, section)
  }
}
#let theme = base.extend(components: (section: section))
```

| Helper | Behavior |
|---|---|
| `entry_details(ctx, entry)` | Heading, secondary, metadata, highlights; excludes period |
| `entry_flow(ctx, entry)` | Complete generic entry, including period and text-list handling |
| `section_flow(ctx, section)` | Heading and ordered resolved entry components |
| `section_table(ctx, section, headers: none, date_width: 30mm, details_first: auto)` | Complete date/detail rows or concrete generic fallback |
| `section_labels(ctx, section, label_width: 36mm)` | Skill label/value rows or concrete generic fallback |
| `table_kind(section)` | Compatible homogeneous entry kind, or `none` |
| `classic_components(header_rule: false, entry_indent: 0.15in)` | Shared header, section rule, italic secondary text, and indented entries with dates beside titles and full-width details below |
| `contacts(ctx, separator: " | ")` | Structured contact labels with link targets |

`entry_details` is also a resolved child component. Helpers call resolved rich,
heading, styled-text, highlight-list, section-heading, and entry components where
those elements are composed. Fallback calls `section_flow` directly, avoiding
redispatch to an overridden section and infinite recursion.

Table eligibility checks every entry. Homogeneous experience and education use
date-first rows; project and publication use details-first rows. Skill rows also
require absent secondary, period, and metadata. Skill groups with no values use generic flow. Mixed, custom, text, prose, empty, and
unrecognized kinds use generic flow. All rows without dates collapse to one
details column; partial dates retain empty cells. Headers are optional dictionaries
with `details` and `period` labels. The example centralizes English labels; titles
do not select layout or language. Automatic translation is outside this API.

Document flow owns section gaps; section helpers own heading and entry gaps;
entry helpers own internal spacing. Table row gutters use entry spacing once.
Rows normally stay together when entry page breaking is disabled. Helpers measure
content at its actual width against the page's usable height and permit oversized
entries to continue. Tables reserve room for repeated headers. With page breaking
enabled, rows may continue and supplied headers repeat. Inspect long-content
outputs visually when changing widths, fonts, headers, or pagination.

## Acceptance workflow

The bundled CV corpus lives in `crates/cac-render/fixtures/shared/` and is embedded
for CLI use. Test-only modules live in each crate's `tests/unit/` directory and
are loaded through `#[cfg(test)]` declarations so they can test private helpers.

Run `cac theme test --shared` in a theme-development project. It runs the normal
preview plus bundled fixtures and stages artifacts only after all checks pass.
Fixture PDFs are `offering/<theme>-shared-<fixture>.pdf`; they do not replace the
project preview or enter package inventory. `cac theme test` and `cac theme pack`
retain the project-specific workflow.

The corpus covers minimal and complete content, translated headings, empty and
reordered sections, repeated kinds, both mixed-entry orders, absent and partial
dates, unknown endpoints, single event dates, multiple labeled contacts, paragraphs,
nested and numbered lists, rich text, links, Unicode, literal markup, long titles/URLs, oversized
entries, and common settings. It checks distinct markers, dates, phone values,
entry/section order, and link destinations in the laid-out document used to export
the PDF. Renderer tests also exercise unknown kinds at the view boundary and
helper selection. These checks do not prove glyph appearance, clipping, or every
possible override. Review representative PDFs and source before registry admission.
Workspace tests run the same corpus for embedded classic, every registry entry,
and the table example; CI runs for theme-only changes as well as code changes.

The reference-theme migration was visually checked using the registry previews,
complete and settings fixtures, mixed fallback, and continuation pages from the
long date/detail and skill-label fixtures. The metadata spacing regression and
date-column positions are also checked from rendered text coordinates. Exported
complete-fixture PDFs were inspected for the email, project, publication, and
inline link annotations. The corpus remains a finite acceptance aid; new theme
compositions still need source and visual review.


## Theme-defined section categories

Section `kind` is an open string identifier, and `id` is user-defined identity.
The built-in kinds supply authoring defaults; additional kinds require no Rust
changes or theme manifest registration. All native source formats preserve them.
Themes may dispatch on either value while keeping a complete generic fallback:

```typst
#import "/.cac/base.typ" as base

#let section(ctx, section) = {
  if section.kind == "certifications" or section.id == "credentials" {
    base.section_table(ctx, section)
  } else {
    base.section_flow(ctx, section)
  }
}
#let theme = base.extend(components: (section: section))
```

Markdown authors write `Kind: certifications` and `Id: credentials` immediately
below a level-two heading. Kind identifiers are case-sensitive letters, numbers,
hyphens, underscores, or dots. IDs retain their literal text and must be unique.
Unknown section kinds default to custom entries; entry-level `Kind: experience`,
for example, can override that structural default. The entry and rich-text
schemas remain defined by the core model. Themes determine presentation, not
whether a CV can be parsed or converted.
