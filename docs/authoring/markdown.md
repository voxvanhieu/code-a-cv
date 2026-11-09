# The FieldMark grammar (0.4 spec)

FieldMark is Markdown. A CV is a Markdown file whose **block structure**
carries meaning and whose **whitespace never does**: a file reformatted by
Prettier or markdownlint — tools that preserve the block structure — must
build to the same CV. This document is the normative grammar for the 0.4
parser (`crates/cac-io/src/markdown/`); the corpus under
`crates/cac-io/tests/fixtures/markdown/` pins it as `input.md` +
`input.md.expected.json` pairs, with the 0.3.0 output recorded as the
before-baseline.

## The principle

A grammar may not assign meaning to anything the CommonMark ecosystem treats
as free. Free (never meaningful): blank lines, list markers (`-` vs `*`),
indentation, emphasis style, escaping, line wrapping inside a paragraph.
Structure (meaningful): headings, paragraphs, lists, HTML comments, YAML
front matter. Everything below follows from this split.

## Phase 1 — the CommonMark AST

The parser walks a CommonMark event stream (pulldown-cmark) and uses:

| Structure | Role |
|---|---|
| YAML front matter | Declarations: `locale`, `packs`; profile fields |
| Heading level 1 | The profile name |
| Heading level 2 | A section |
| Heading level 3+ | An entry |
| Paragraph | A **field block** if it earns the role (rule F), else prose |
| List | Highlights, skills, or entry content |
| HTML comment `<!-- ... -->` | Legacy directives; rewritten by `cac fmt` |

Whitespace between these never reaches phase two: blank lines before, inside,
or after a field block mean nothing; list markers and indentation are free.

## Phase 2 — mapping structure to the model

### Rule P — profile

The level-one heading's text is the name. A field block (rule F) immediately
after it fills `Email`, `Phone`, `Location`, `Website`, `Summary`, and the
declared profile fields. The legacy `first · second · third` contact line and
`Contact:` entries stay valid as sugar. Undeclared keys inside a field block
become open fields with an advisory, never errors.

### Rule S — sections

A level-two heading starts a section. Its title is the display title. A field
block after it may set `Kind` (open section kind, default from the title per
the built-in table), `Id`, and `Tags`. Sections run to the next level-two
heading or the end of file.

### Rule E — entries

A level-three heading starts an entry. The heading text is sugar (rule G);
the entry continues until the next heading of any level or the end of file.
Fields, a period/date line, highlights, and prose may follow in any order.

### Rule F — the field block

A paragraph is a **field block** if and only if:

1. every non-empty line is a `Key: value` field, a date or period line, or —
   under the name only — a legacy contact line; and
2. at least one key is **known** (in the core field table,
   `crates/cac-core/src/fields.rs`) or **declared** (by a pack listed in
   front matter or settings).

Otherwise the paragraph is prose. A paragraph of only undeclared `Word:`
lines is prose with an advisory, not an error. This rule is the replacement
for the 0.3 "metadata block ends at the first blank line" — the property
Prettier's blank-line insertion exploited to corrupt CVs.

### Rule G — heading sugar

`### A, B` fills the subtitle with `B` only when the entry kind has a
subtitle slot **and** no explicit subtitle field is present. Explicit fields
always win; sugar fills gaps, identically for every entry in the file — there
is no per-entry mode. `### A — B` (em dash) is the unambiguous separator that
never conflicts with a comma in a title. `cac check` reports each applied
split. A bare period/date line directly after a level-three heading sets the
entry's period or date; no other line is ever tried as a date, so prose
beginning `10 - 15 engineers…` cannot become one.

### Rule D — directives

Directives are fields, not a third syntax. `Tags: a, b` is a field on
sections and entries. `<!-- cac:summary -->` and `<!-- cac:entry kind=... -->`
remain escape hatches that the writer emits only when content would otherwise
be misread (rule F decides; the directives stay for the rare tie). The legacy
`<!-- cac:section id=... kind=... -->` is accepted and rewritten by
`cac fmt`. Front matter is the standard place for declarations, because every
Markdown tool preserves it.

## Field packs

A pack is a `fields.toml` file that types and requires open fields.
`cac check cv.md --pack fields.toml` (repeatable) validates against it:

```toml
name = "jp-rirekisho"

[fields.entry.commute]
label = "通勤時間"
type = "text"
required = true
```

The loading and checking rules:

- The declared `name` identifies the pack in diagnostics — not the file
  stem. Unknown keys at any level (`requird`), a scope other than
  `profile`/`entry`, and two keys that normalize to the same form are load
  errors naming the pack file.
- Keys normalize like CV keys: case-, underscore-, and
  whitespace-insensitive, so `Commute_Time` and `commute-time` are one
  field.
- `type` is `text` (the default), `integer`, `boolean`, `length`, or
  `date`. Markdown and front matter carry every open field as text, so the
  checker coerces through the declared kind before it judges:
  `Dependents: 2` passes an `integer` field, `Remote: yes` a `boolean`
  field, `Available From: 2026-04` a `date` field. Values that already
  carry a type (JSON, YAML, TOML) validate directly.
- A required **profile** field is missing when the profile has neither the
  typed slot for the core key (`photo`, `email`, `phone`, `location`,
  `website`, `summary`, `headline`, `address`) nor a matching open field.
- A required **entry** field is checked only on entries that can carry it.
  Core keys use the core field table's entry scopes: a required
  `organization` flags experience entries only. Pack-only keys are
  required on an entry when the entry's section already uses the key —
  at least one sibling entry declares it. A rirekisho pack can therefore
  require `commute` without firing on a Skills section, while an entry
  whose siblings declare `Commute` and does not is an error.
- Identical diagnostics are reported once, however many loaded packs agree.

## The two spellings

The plain style (0.1) is sugar over the labeled style (0.3), defined by
rules F and G — not a separate mode:

| Plain spelling | Explicit meaning |
|---|---|
| `### Role, Organization` | heading sugar filling the subtitle |
| bare `Jan 2023–Present` line | the entry's period |
| `a · b · c · d` contact line | Email/Phone/Location/Website fields |
| section title `Experience` | `Kind: experience` by the built-in table |

`cac fmt` converts between spellings: `--style explicit` writes every field
(diff-friendly, lossless); `--style compact` uses sugar where unambiguous.
`parse(compact(x)) == parse(explicit(x)) == parse(x)` is a property test
(Q2). Formatted output is formatter-stable: a blank line after every heading,
one paragraph per field block, a Prettier and markdownlint fixed point
(enforced in CI, task Q1).

## Baseline: what 0.3.0 did

The corpus records current output so every change is a visible diff.
Notable baseline behaviors, all deliberate targets of this grammar:

| Fixture | 0.3.0 baseline | 0.4 rule |
|---|---|---|
| `after-prettier` | contacts became summary; `Kind:`/`Id:` became prose; every field became entry content | rule F: identical model to `labeled-style` |
| `edge-comma-heading-alone` vs `edge-comma-heading-with-field` | the same heading split two ways, chosen by an invisible per-entry flag | rule G: one rule for the whole file |
| `edge-false-date` | hard error: prose tried as a period | rule G: only the first paragraph after the heading may carry a date |
| `edge-mailto-name` | hard error: unknown field `mailto` | rule P: contact sugar; undeclared keys are open fields with an advisory |
| `edge-note-prose` | `Note:` prose parsed, fragile to formatting | prose by rule F, with an advisory |
| `front-matter` | hard error: "content must start with a level-one name heading" | rule D: declarations and profile fields |
| `edge-summary-lookalike` | directive placement error | rule D: summary is a profile field; the directive is a fallback |

## Corpus reference

| Fixture | Pins |
|---|---|
| `plain-style` | 0.1 sugar: contact line, comma heading, bare date lines |
| `labeled-style` | 0.3 fields: Kind/Id/Organization/Period, `Tags:` comments |
| `mixed-style` | both spellings in one file |
| `after-prettier` | blank line after every heading must not change the CV |
| `after-markdownlint` | MD022 blanks + `*` list markers must not change the CV |
| `edge-false-date` | prose starting with a number is never a date |
| `edge-note-prose` | `Note:` prose stays prose |
| `edge-mailto-name` | bare `mailto:` line is contact sugar, not an error |
| `edge-summary-lookalike` | summary text that resembles a contact line |
| `edge-comma-heading-alone` | heading sugar when nothing follows |
| `edge-comma-heading-with-field` | explicit field suppresses the split |
| `front-matter` | YAML front matter for declarations and profile fields |
| `legacy-directives` | all three directive forms |
| `minimal` | name only |

Adding a fixture: pair `input.md` with a reviewed `input.md.expected.json`
(recorded via `CAC_GENERATE_FIXTURE_BASELINE=1 cargo test -p cac-io --test
markdown_baseline`), then describe the pin here. The baseline test fails on
any drift, so grammar changes are always conscious.
