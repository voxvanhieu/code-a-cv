# Writing a CV with FieldMark

[Usage guide](README.md)

**FieldMark** is Code a CV's Markdown writing style: readable Markdown with
structured fields such as `Kind:`, `Id:`, and `Organization:`. Keep using `.md`
files and the existing Markdown commands; no special editor is required.

## Writing FieldMark

1. Use `#` for your name, `##` for sections, and `###` for entries.
2. Put optional `Field: value` lines immediately below their heading. Below `##`,
   `Kind:` describes the section and `Id:` gives it a stable identity. Below `###`,
   `Kind:` selects the entry structure and fields such as `Organization:` supply
   its details.
3. Insert a blank line before prose or lists. Fields do not resume after it.
4. Omit fields you do not need. Use supported field names, including the spelling
   `Organization:`; put additional information in prose or bullets.
5. Use `cac check cv.md --explain` to inspect the interpretation and
   `cac fmt cv.md` to format the source in place. Use `cac fmt cv.md --dry-run`
   to check formatting without changing it.

This excerpt uses the fictional Ada Lovelace profile from the
[complete FieldMark example](../examples/theme-defined-sections/cv.md):

```markdown
# Ada Lovelace
Email: ada@example.com
Location: London, United Kingdom

Software engineer focused on reliable developer tools and clear technical communication.

## Experience
Kind: experience
Id: career

### Senior Software Engineer
Organization: Analytical Engines Ltd
Period: Jan 2023–Present

- Reduced deployment time by **35%** through parallel build stages
- Led 4 engineers in delivering a new release workflow within 3 months
```

Field lines are visible in ordinary Markdown previews. Code a CV interprets them
as structured data and lets the theme arrange their values in the rendered CV.

## Minimal CV

A CV needs one nonempty name heading. Contacts, summary, sections, employers,
institutions, dates, URLs, and achievements are optional. Remove information you
do not want to provide; there is no need to write `N/A` or invent dates.

```markdown
# Ada Lovelace

## Experience

### Independent Consultant
```

This builds. `cac check` may suggest adding contact information or achievements;
those suggestions do not stop `cac build`. `cac check --strict` deliberately fails
on any advisory diagnostic.

## Headings and explicit fields

Use `#` for your name, `##` for sections, and `###` for entries. Metadata is
optional. Put metadata immediately after a name, section, or entry heading, before the
first blank line. A blank line ends metadata and begins ordinary Markdown.

```markdown
# Ada Lovelace
Email: ada@example.com
Phone: +44 1632 960 000
Location: London, United Kingdom
Website: https://example.com
Contact: [GitHub](https://github.com/example)

<!-- Contacts, dates, organizations, and highlights are optional. Delete what you do not want to provide. -->

Software engineer focused on reliable developer tools and clear technical communication.

## Experience
Kind: experience
Id: career

### Senior Software Engineer
Organization: Analytical Engines Ltd
Period: Jan 2023–Present

- Reduced deployment time by **35%** through parallel build stages
- Led 4 engineers in delivering a new release workflow within 3 months

### Software Engineer
Organization: Difference Engine Co
Period: Jul 2020–Dec 2022

- Improved test coverage from 72% to 91% across 6 services
```

With explicit metadata, the whole entry heading is the primary text. For example,
`### Senior Software Engineer, Developer Tools` remains one role when followed by
`Organization: Analytical Engines Ltd`. Commas in headings have no special meaning in the explicit
form. With no metadata, legacy experience and education headings still split at
the first comma followed by a space: `### Software Engineer, Difference Engine Co`. Add `Kind: experience`
to keep a comma in a role without supplying an employer.

Field names are case-insensitive. Built-in kinds use the lowercase names below;
theme-defined section kinds preserve their spelling.
An unknown metadata field, duplicate singleton field, or incompatible field
produces an error with its source line. For example, `Organzation:` must be
corrected to `Organization:`. To write ordinary prose containing a colon, insert
a blank line first:

```markdown
### Engineer

Impact: improved reliability across the team.
```

An empty optional field is equivalent to omitting it. A blank required heading is
an error. Metadata does not resume after a blank line or a prose/list line.

## Profile fields

| Field | Meaning |
|---|---|
| `Email` | One email address; optional |
| `Phone` | One display phone number; local numbers are accepted |
| `Location` | Any display location, including a city or `Remote` |
| `Website` | One absolute URL |
| `Contact` | Repeatable plain label or Markdown link, e.g. `[Portfolio](https://example.com)` |

Use multiple `Contact` lines for multiple websites or email addresses. Repeating
`Email`, `Phone`, `Location`, or `Website` is an error even if a value is blank.
Labels are literal display text; rich formatting in a contact label is reduced
to its visible text. Supported link targets use `https:`, `http:`, `mailto:`, or
`tel:`. Relative URLs and executable URL schemes are rejected.

Legacy standalone email/phone/website lines and contact lines separated by `·`
remain supported before the summary. They no longer overwrite previous contacts.
Use labeled fields for locations, multiple contacts, and ambiguous text. Ordinary
summary sentences containing `@company` or a URL remain summary sentences.

## Sections and entry kinds

Section titles are display text, in any language. These conventional titles have
built-in defaults (case-insensitive):

| Section title | Section kind | Default entry kind |
|---|---|---|
| Experience, Work Experience, Professional Experience, Employment, Employment History | `experience` | `experience` |
| Education | `education` | `education` |
| Project, Projects | `projects` | `project` |
| Publication, Publications | `publications` | `publication` |
| Skill, Skills, Technical Skills | `skills` | `skill-group` |
| Any other title | `custom` | `custom` |

For a translated or personalized title, put `Kind:` and optionally `Id:`
immediately below the section heading. A blank line ends metadata. Both fields
can be omitted or left blank to retain the defaults:

```markdown
## Опыт работы
Kind: experience
Id: career
```

The section kind supplies a default. An entry's explicit `Kind` overrides it, so
a section may contain jobs, projects, and prose together. Section kinds are open:
`awards`, `certifications`, or a theme-specific identifier are accepted and passed
unchanged to themes. Unrecognized section kinds default to custom entries.
Entry kinds below `###` still use the supported structural types listed below.
Kind identifiers are case-sensitive and contain letters, numbers, hyphens,
underscores, or dots. Prefer lowercase identifiers.

IDs are generated from titles when omitted. Repeated titles get distinct IDs such
as `experience` and `experience-2`. Explicit IDs are reserved before automatic IDs
are allocated; duplicate explicit IDs are errors. Generated Markdown preserves IDs
in `Id:` fields. Renaming a heading does not change an explicitly assigned ID.

| Entry `Kind` | Heading means | Optional fields |
|---|---|---|
| `experience` | Role | Organization, Location, Period **or** Date |
| `education` | Qualification | Institution, Period **or** Date |
| `project` | Project name | URL, Period **or** Date |
| `publication` | Publication title | Publisher, URL, Date |
| `skill-group` | Group name | No date or organization fields |
| `custom` | Any meaningful heading | Period **or** Date |
| `text` | One unheaded list item | No metadata beyond Kind |
| `prose` | One unheaded prose item | No metadata beyond Kind |

Use prose or bullets for additional facts such as GPA, credential IDs, publication
authors, expected graduation, or certification expiry. Do not put unsupported
fields into metadata: they would not have a defined semantic destination.

## Dates

Dates accept `YYYY`, `YYYY-MM`, `YYYY-MM-DD`, or an English month name and year
(`Jan 2023`, `January 2023`). A period uses an en dash or space-hyphen-space:

| Input | Meaning |
|---|---|
| Omit Date and Period | No date supplied |
| `Date: 2024` | One event in 2024 |
| `Period: 2020–2023` | Known start and end, with year precision |
| `Period: 2020-06 - 2023-02` | Known start and end, with month precision |
| `Period: 2022–Present` | Explicitly ongoing; `Current` is also accepted in Markdown |
| `Period: 2022–` | Known start, unknown end; **not** ongoing by implication |
| `Period: –2024` | Known end, unknown start |

Future dates, overlapping jobs, and mixed date precision are valid. An impossible
date, definitely reversed range, two unknown endpoints, `Present` as a start/date,
or both Date and Period on the same entry are errors. `03/04/2023` is ambiguous;
use an unambiguous supported date. Localized wording such as “Summer 2021” can be
ordinary prose without claiming a parsed calendar date.

Legacy bare date/range lines before entry content still work. Dates on skills and
ranges on publications now fail instead of being discarded or truncated.

## Prose, lists, and formatting

Prose is preserved as prose, including paragraph boundaries. `-`, `*`, and `+`
produce unordered lists. Indented continuation lines, nested lists, ordered lists,
and explicit Markdown hard breaks are supported. A simple unordered list under an
entry becomes highlights (skills for a skill group). Content mixing prose and
lists preserves its complete order.

Direct simple bullets under a section remain text entries, or empty skill groups
under a skills section. More complex standalone prose/lists use a prose entry.
Section and entry order are preserved. Empty sections retain their headings.

Supported inline formatting is bold, italic, inline code, and inline links.
Reference-link definitions and link tooltips are rejected; use `[label](URL)` and
put meaningful descriptions in the visible label. Escape literal
Markdown punctuation with a backslash, such as `\*literal stars\*`, or use inline
code. HTML layout, images, tables, blockquotes, fenced code blocks, and deeper
headings are rejected with suggestions. An image can be replaced with a labeled
link. Ordinary HTML comments are editing notes and are not rendered.

Tags remain invisible metadata:

```markdown
<!-- tags: backend, leadership -->
<!-- tags: ["tag containing, a comma", "Unicode Анна"] -->
```

Put a section's tags before its entries; put entry tags beneath its heading or
following a direct bullet/prose item. Reserved `cac:` directives are checked
strictly. Canonical Markdown may contain `<!-- cac:summary -->` to disambiguate a
summary from legacy contacts, or `<!-- cac:entry kind=prose -->` / `kind=text` to
preserve standalone content semantics. These directives have no visible output.

## Tools for FieldMark

See [checking and formatting](checking-and-formatting.md) to inspect parsed fields,
format the source, or install editor snippets. Use [format conversion](conversion.md)
to move between FieldMark and structured sources, and [building](building.md) to
render PDF or HTML.

## Relationship to themes and structured sources

Markdown becomes the same core CV data as native JSON, YAML, and TOML. Optional
organization and institution fields can be omitted in every native format.
Additional profile contacts use `contacts: [{label, href}]`. Period endpoints are
optional; a period must have at least one known endpoint. Experience, education,
projects, and custom entries now accept a single `date` as an alternative to
`period`.

An entry may supply `content` (a rich Markdown string containing ordered prose and
lists) instead of `highlights`/`skills`. They cannot be combined; include all prose
and lists in `content` when order matters. A standalone prose entry uses
`{"type": "prose", "body": "Paragraph one.\n\nParagraph two."}`.

The renderer translates these fields into the [shared theme view](../development/shared-theme-contract.md).
Themes must display populated visible slots, tolerate omissions, and preserve
order. A shared theme's inability to display supported content is a theme
compatibility problem, not a reason to require users to invent missing CV fields.

## Acceptance decisions

| Real-life case | Result |
|---|---|
| Student without jobs; self-taught applicant without education | Build |
| Anonymous draft or intentionally omitted contacts | Build; optional contact advice |
| Confidential employer or absent institution | Build |
| Empty section or heading-only entry | Build; optional content advice |
| Career break, volunteering, awards, caregiving, interests | Build as custom content |
| Multiple roles at one employer, overlap, repetition, arbitrary order | Build without deduplication |
| Translated headings, repeated titles, mixed entry kinds | Build |
| Additional contact links and local phone numbers | Build |
| Unknown date endpoint, single award date, future graduation | Build |
| Prose, soft wrapping, nested/numbered lists, Unicode | Build and preserve structure |
| Missing/blank identity or entry heading | Error |
| Unknown/duplicate metadata or conflicting dates | Error with source line |
| Malformed date/URL or a field incompatible with its entry kind | Error |
| Unsupported visible markup | Error with a supported alternative |
| Output format cannot preserve supplied information | Error before writing |
| Missing metrics, first-person wording, sparse achievements | Build; optional editorial advice |

## Theme-defined sections

The [complete example](../examples/theme-defined-sections/cv.md) works with any theme. A theme may choose a special layout
for `certifications` or for the specific `credentials` ID:

```markdown
# Ada Lovelace

## Professional credentials
Kind: certifications
Id: credentials

### Cloud Engineering Certificate
Date: 2024

- Completed practical assessments in infrastructure automation, monitoring, and incident response

## Selected consulting
Kind: consulting
Id: independent-work

### Technical Advisor
Kind: experience
Organization: Example Foundation
Period: 2021–2022

- Helped a team of 6 maintainers introduce repeatable releases and contributor documentation
```

The theme receives the literal section kind and ID; no theme registration or
compiler rebuild is needed to introduce another section category. Themes that
do not recognize the category use their normal section layout. Parsing,
formatting, and conversion work without a theme installed. A new section kind
does not introduce new structured entry fields; use existing entry kinds and prose.

Legacy `<!-- cac:section id=career kind=experience -->` annotations remain
accepted. Markdown conversion emits `Kind:` and `Id:` lines; `cac fmt` preserves the existing syntax. Do not declare
the same field in both forms. Duplicate fields are errors, including blank
duplicates. Unlike legacy comments, the new fields are visible in ordinary
Markdown previews.
