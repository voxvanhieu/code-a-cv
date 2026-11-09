# Improvement roadmap — 0.4

The ordered implementation plan derived from the architecture review
(a user-provided document, reviewed 2026-10-23 against the 0.3 codebase).
The ordering is dependency-driven: contracts and schemas first (pure
additive work, no rework), then implementations against the frozen
contracts, then features, then reference templates. Each task commits on
its own.

**Status**: implemented on the `feature/feature-improvements` branch and
under review. This file is the plan, not a completion record — the review
history and per-task commits live in the branch; the merge state lives in
the pull request.

## How this file is used

Work the list top to bottom: pick the first unchecked task, implement it,
run the four checks (`cargo fmt --all --check`, `cargo clippy --workspace
--all-features --all-targets --locked -- -D warnings`, `cargo test
--workspace --all-features --locked`, `bash
scripts/tests/test_release_resolution.sh`), commit, then tick the box.
Keep `[ ]`/`[x]` exactly parseable.

## Wave 0 — Architecture contracts (types, schemas, fixtures; no behavior change)

- [ ] **C1 · Model contract** — In `crates/cac-core`: a field table as data
  (key, label, accepted entry kinds, parser kind); `Entry.fields` as an
  ordered open map; core extension types for photo, headline, address, skill
  level, language proficiency, reference, education score. Types and serde
  round-trip tests only. Files: `crates/cac-core/src/fields.rs`,
  `document.rs`, `lib.rs`.
- [ ] **C2 · Style contract** — One style vocabulary as serde structs with
  `schemars`: page, fonts, colors, text, headings, spacing, lists, links,
  entry presentation, locale; `ResolvedStyle`; a `theme.toml` schema; a
  table mapping all 20 legacy settings keys to new keys. Committed schemas;
  defaults validate. Files: `crates/cac-render/src/style/`,
  `crates/cac-render/schema/`.
- [ ] **C3 · Layout contract** — The four hook signatures, the `ctx` shape,
  the named view slots, structured dates with locale/calendar; a stub
  `contract.typ` that compiles with plain `typst`. Files:
  `docs/development/shared-theme-contract.md`,
  `crates/cac-render/src/view/types.rs`,
  `crates/cac-render/src/typst/contract.typ`.
- [ ] **C4 · Grammar spec and corpus** — The AST-based FieldMark grammar as
  rules in `docs/authoring/markdown.md` (structure only); fixture pairs
  `input.md` + `expected.json` covering plain, labeled, mixed, post-Prettier,
  post-markdownlint, edge cases, legacy directives, and front matter.
  Files: `docs/authoring/markdown.md`,
  `crates/cac-io/tests/fixtures/markdown/`.
- [ ] **T1 · Tags to variants** — `cac build --tags a,b`: untagged content
  is shared, entries tagged for another job never leak, the artifact name
  comes from the normalized sorted tag set, `--only-tagged` keeps the
  strict spelling. Files: `crates/cac/src/commands/build.rs`,
  `crates/cac-core/src/document.rs`, `docs/examples/job-specific-cv/`.
- [ ] **T0 · Windows-clean tests and file:// registry** — Path-separator-
  agnostic assertions, `cac.exe` in help snapshots, `file://` handling in
  the registry fetch, a `windows-latest` CI job.

Gate 0: contracts read end to end and frozen — subtitle is the one word for
the view slot; dates cross the boundary as canonical strings; style groups
are the C2 set; entry kinds use kebab spellings; rich text stays the tagged
node array.

## Wave 1 — Implementations against the frozen contracts

- [ ] **P1 · Two-phase AST parser** — Phase one pulldown-cmark, phase two
  maps structure to model. Blank lines carry no meaning; undeclared keys
  become open fields; heading sugar fills the subtitle per field. Done when
  all C4 fixtures pass and the post-Prettier fixture parses identical to
  the labeled fixture. Files: `crates/cac-io/src/markdown/`.
- [ ] **P2 · Writer and fmt styles** — `cac fmt --style
  explicit|compact|preserve`; formatter-stable output; `preserve` takes its
  spelling from the parser, refuses to drop plain comments, and re-emits
  front-matter declarations. Files: `crates/cac-io/src/markdown/write.rs`,
  `crates/cac/src/commands/fmt.rs`.
- [ ] **S1 · Style resolver** — Defaults ← theme `[style]` ← settings
  `[style]`, merged and validated once into a resolved sheet; legacy key
  shim with advisories; the world serves `/.cac/style.json`. Files:
  `crates/cac-render/src/style/`.
- [ ] **S2 · Typst reads the sheet** — Tokens, page, typography from the
  resolved sheet; the sparse settings layer carries only set keys with
  lengths already in `(value, unit)` form. Files:
  `crates/cac-render/src/typst/style.typ`, `base.typ`, `main.typ`.
- [ ] **V1 · View slots and structured dates** — Named slots, structured
  dates, open fields in the view; one locale-aware date formatter shared by
  both renderers. Files: `crates/cac-render/src/view.rs`,
  `crates/cac-render/src/view/dates.rs`.
- [ ] **S3 · HTML on the sheet and the view** — CSS custom properties from
  the resolved sheet; the renderer walks the view; dates from the shared
  formatter. Files: `crates/cac-render/src/html.rs`.
- [ ] **L1 · Layout hooks and families** — Four hooks over pure helpers;
  classic, table, compact, sidebar families; the legacy components adapter
  accepted for one release. Files: `crates/cac-render/src/typst/layout.typ`,
  `themes/classic.typ`.

Gate 1: the round-trip gate runs every Markdown fixture and the shared
corpus in all four native formats; HTML and PDF print identical date
strings through the shared formatter.

## Wave 2 — Features

- [ ] **M1 · Core extension behavior** — Markdown spellings and JSON Resume
  mapping both ways for photo, headline, address, skill levels, languages,
  references, score. Files: `crates/cac-io/src/json_resume.rs`,
  `crates/cac-io/src/markdown/`, `crates/cac-core/src/fields.rs`.
- [ ] **M2 · Field packs** — `fields.toml` loader; typed open fields with
  labels; `cac check --pack`; text coerces to the declared type; required
  keys check the model slots with the JSON Resume spellings as aliases;
  filesystem access lives in the `cac` crate. Files:
  `crates/cac-core/src/packs.rs`, `crates/cac-io/src/packs.rs`,
  `crates/cac-check/src/packs.rs`, `packs/`.
- [ ] **L2 · Layout capabilities** — Sidebar regions and numbering, photo
  slot with the project jail, overflow diagnostics. Skill level bars stay
  open until the view carries per-skill levels. Files:
  `crates/cac-render/src/typst/layout.typ`,
  `crates/cac-render/src/diagnostics.rs`.
- [ ] **L3 · Form template class** — `kind = "form"` with `requires`; the
  family contract is enforced by the dispatcher no matter which document
  hook a theme overrides; fixed grids; CJK fonts; era dates from the shared
  formatter. Files: `crates/cac-render/src/typst/layout.typ`,
  `crates/cac-render/src/view/dates.rs`.
- [ ] **Y1 · Lifecycle commands** — `cac theme init` writes real `.cac/`
  files plus the data inputs the bridge reads, so `typst watch --root .`
  works and `cac build` honours the on-disk scaffold; `theme check` is
  pure; `theme pack` generates README, preview, checksums. Files:
  `crates/cac/src/commands/theme/`, `crates/cac-render/src/pdf.rs`.
- [ ] **D1 · Diagnostics and document date** — Overflow surfaced from frame
  inspection; `document_date` for deterministic builds (never the clock);
  the world serves the date with its era spellings. Files:
  `crates/cac-render/src/diagnostics.rs`,
  `crates/cac-render/src/pdf.rs`.
- [ ] **Q1 · CI formatter fixed point** — CI runs Prettier + markdownlint
  on formatted examples and asserts a no-op with pinned tool versions;
  the job gates the merge. Files: `.github/workflows/ci.yaml`,
  `scripts/tests/test_formatter_fixed_point.sh`.
- [ ] **W1 · Tutorial and generated reference** — Ten-minute theme
  tutorial; the style-key reference generated from the schema and linked
  from the tutorial; FieldMark guide as the grammar appendix. Files:
  `docs/authoring/your-first-theme.md`, `docs/reference/style-keys.md`.

Gate 2: the corpus through every family; examples and previews
regenerated; changelog and migration notes.

## Wave 3 — Reference templates and release

Reference templates ship as their own theme-only pull requests (one per
theme, `themes/` files only), after the renderer changes they depend on
have merged. Each registers in the schema-2 registry (`themes/index.v2.json`)
with a `min_cac` gate; the frozen schema-1 index stays untouched for
released clients.

- [ ] **R1 · `shapes` sidebar theme** — Declarative sidebar theme with
  photo, background shapes, references (original design in the resume.io
  "shapes" genre, not a clone). Files: `themes/shapes/`.
- [ ] **R2 · `leaves` full-bleed theme** — Full-bleed background art, level
  bars, languages; the ATS diagnostic documents the trade-off. Files:
  `themes/leaves/`.
- [ ] **R3 · `rirekisho` Japanese form** — JIS-style 履歴書 form with the
  jp packs: era dates from the shared formatter, photo cell, fixed grids,
  CJK fonts with their licence; rejects sections it cannot place. Files:
  `themes/rirekisho/`, `packs/jp-rirekisho/`, `packs/jp-shokumu/`.
- [ ] **Q2 · Property-based round trips** — Generated documents through
  Markdown, JSON, YAML, TOML, and packs. Files:
  `crates/cac-io/tests/q2_property.rs`.
- [ ] **Z1 · Release 0.4** — Dated changelog section with the preserved
  history, version bump, migration guide, tag: a separate final release
  pull request, not part of a feature branch.

## Feature verification

Every shipped feature was exercised against the locally built CLI before
review, with the captured output embedded in the pull-request description
rather than the repository. Demoing found and fixed real defects: writer
payload gaps, an explicit `Summary:` clobbered by the prose path, a
spurious `location: {}` in JSON Resume export, and the rirekisho theme's
era column.

## Invariants (from the review; every task must preserve them)

- Dependency direction points inward; `cac-core` never touches the filesystem.
- Rendering safety: CV strings travel to Typst as JSON, never interpolated;
  themes read only the renderer-collected assets plus their own files;
  `cac check` read-only.
- Deterministic builds: no `today()` without `document_date`; pinned
  timestamps; byte-identical rebuilds.
- Lossy conversions are refused before writing.
- Themes must render the shared corpus (`crates/cac-render/fixtures/shared/`)
  without losing a marker.
- PRs touching `themes/` contain only theme files; code/changelog changes
  go separately.
