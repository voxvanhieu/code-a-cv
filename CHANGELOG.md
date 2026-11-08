# Changelog

All notable changes to Code a CV are documented in this file.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and releases use [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

- Add the `rirekisho` registry theme: a JIS-style 履歴書 form on the form family — fixed grid with a photo cell, furigana from the profile's open field, 和暦 era dates computed against `document_date`, CJK glyphs from the theme's bundled Noto Sans JP font, and rejection of sections outside its allowed list. The registry lists form themes separately from universal themes.

- Add the `leaves` registry theme: a full-bleed decorative layout with leaf-toned shapes in the corners, a soft header banner with linked contacts, and numbered section headings with a short rule. Decorative shapes are PDF-only; HTML keeps the sheet colors.

- Add the `shapes` registry theme: a sidebar layout with soft geometric shapes behind the skills column, a muted accent palette, numbered main-column headings, a circular photo slot, and language/reference listings. Original design in the sidebar genre.

- Add the `form` layout family: a fixed-grid template with a photo cell, address block, and labeled rows. A form theme declares `requires` (sections and profile keys it needs) and `allowed` (section kinds it can place); a document containing anything else fails with the section named, instead of silently dropping content.

- Add a ten-minute first-theme tutorial (`docs/authoring/your-first-theme.md`): start a theme project, pick a family or override style keys, render both outputs, and check — with links into the style-key schema and the theme contract.

- Retire the `themeProject` marker: `cac theme init` writes real files under `.cac/` — the style sheet bridge, the base library, the layout families, and a root `main.typ` — so `typst watch .cac/main.typ` and editor extensions preview the theme outside cac. A theme project is now identified by its theme directory under `.cac/themes` (name falls back to `settings.theme`), and `theme init` no longer blocks installs inside theme projects.

- Enforce the formatter fixed point in CI: a new job formats the five Markdown examples with `cac fmt`, then asserts Prettier (pinned 3.6.2) and markdownlint (0.18.1) are no-ops over them, with a `.markdownlint.jsonc` config that treats labeled content as data. `scripts/tests/test_formatter_fixed_point.sh` runs the same check locally (`npm ci && npm run format:check`).

- Add overflow detection from frame inspection: pages whose content extends beyond the page edge print a warning naming the page, and `RenderedPdf` carries the overflow page list so build output and checks can act on it.

- Add the `document_date` setting (YYYY-MM-DD) for deterministic builds: Typst code referencing `today()` sees the fixed date instead of the wall clock, and builds with the same date stay byte-identical. Non-ISO values are rejected at settings validation.

- Add the photo slot: `Photo:` in a CV renders in the sidebar family as a circular-cropped image. Project-relative files (photos, assets) resolve through a jailed lookup in the render world that refuses anything escaping the project root.

- Add the `sidebar` layout family: a two-column layout with skills, languages, and references in a 32% sidebar, the rest of the CV in the main column, numbered section headings (01, 02, …), and level bars for language entries. A project theme selects it with `layout: (family: "sidebar")`.

- Add field packs: a `fields.toml` declares open-field keys with labels, value kinds (text, integer, boolean, length), and requiredness; `cac check --pack path/to/fields.toml` validates open-field values against the declaration — mistyped values and missing required fields are errors, undeclared keys are advisories. Native formats keep unknown fields end to end.

- Add `language` and `reference` entry kinds with Markdown spellings (`Proficiency: C1`, reference contact lines), education `Score:` support, JSON Resume import/export for photo, headline, languages, references, and education scores, and a `rich` corpus fixture exercising all extensions. `## Languages` and `## References` section titles default their entries to the new kinds; JSON Resume samples with all seven extensions round-trip losslessly.

- Render open fields in the default layouts: entry-level `Key: value` lines appear after the typed metadata with capitalized labels, and profile-level open fields appear under the header contacts — in both PDF and HTML. Documents without open fields are unchanged (byte parity holds).

- Emit the C3 named slots in the rendered view: photo, headline, structured address, and open profile fields on the profile; structured periods, scalar dates, location, URL, open fields, and tags on every entry — beside the legacy keys, with every slot key always present (absent values are null). Themes can now read slots and open fields without a model change; the shared-content checker counts only the legacy keys the default families render.

- Ship built-in `table` and `compact` layout families alongside `classic`: a project theme exports `layout: (family: "table")` to pick one, dated sections render as tables with the period in a gutter (table) or tight flow without rules and indents (compact). Legacy component themes now print a deprecation note pointing at layout hooks and theme.toml.

- Add layout hooks for Typst themes: a theme may export `layout` (document, header, section, entry) instead of the twelve late-bound components, with the classic family shipped as the default and the section→entry hop as the only dynamic dispatch. The 0.3 component dictionary keeps working through an adapter for one release; embedded classic now renders through the hooks path.

- Render the HTML document from the resolved style sheet: CSS custom properties carry fonts, colors, text layout, heading styles, spacing, link underlines, and page geometry, so a theme accent and sheet settings now show up in the HTML instead of hard-coded CSS. Dates print through the shared locale-aware formatter (en, de, fr, vi, ja) — the same one the PDF's view uses — replacing the raw ISO strings HTML previously leaked (`2023-01–Present` becomes `Jan 2023 – Present`).

- Render from the resolved style sheet: base.typ reads `.cac/style.json` (numeric lengths, hex colors, no string evaluation) and the Typst-side settings translation (`apply-settings` with its `eval`) is deleted. Settings still override project themes through a sparse overlay, so the 0.3 precedence holds; the embedded classic theme slims to its components with its typography living in the style sheet. Determinism doubles as parity: classic builds render byte-identical PDFs against the previous pipeline, including documents with legacy settings.

- Resolve the style sheet once in Rust: defaults, then theme.toml `[style]`, then settings.json `[style]` merge into one resolved sheet with concrete lengths and hex colors that both renderers read. Every deprecated 0.3 settings key keeps working through a shim that prints a `NOTE` naming its replacement (including the inverted `allow_entry_page_break`), the new `[style]` groups are accepted beside them, and the resolved sheet is served to themes as `.cac/style.json`.

- Add `cac fmt --style explicit|compact|preserve` for the two FieldMark spellings: `explicit` spells every field, `compact` uses heading and date sugar and omits implied section kinds and slug ids, and `preserve` (the default) keeps whatever the source used. Formatted output is formatter-stable: a blank line after every heading, directives (`cac:summary`, `cac:entry`) emitted only when a plain paragraph would be misread, and all five Markdown examples pass `prettier --check` after formatting.
- Rebuild the Markdown parser on the CommonMark AST: meaning now depends only on block structure, so a file reformatted by Prettier or markdownlint builds to the same CV. Field blocks (`Email:`, `Organization:`, dates) are recognized by content rather than by adjacency to a heading, blank lines between them carry no meaning, unknown `Key:` values become open fields instead of errors, YAML front matter is accepted for `locale`, `packs`, and profile fields, heading sugar (`### Role, Organization`) fills the subtitle per field with no per-entry mode, and prose that merely starts with a number is never mistaken for a date. `Summary:` and `Contact:` spelling joins `Email`/`Phone`/`Location`/`Website` as profile fields. Checked Markdown conversion now carries open profile and entry fields.
- Fix `file://` theme registry URLs on Windows: drive-letter paths (`file:///C:/...`) and percent-escaped directory names now resolve, and the full workspace test suite runs in CI on Windows.
- Add `cac build --tags a,b` for job-specific CV variants: keep the sections and entries tagged with any of the given tags, drop the rest, name the artifact after the variant (`cv-backend.pdf`), and fail with a diagnostic when nothing matches. The profile block always stays; the `job-specific-cv` example demonstrates one source file producing one CV per posting.
- Add the core model extension fields from the 0.4 roadmap contracts: `photo`, `headline`, and structured `address` on the profile, `score` on education entries, and an ordered open `fields` map on every entry for `Key: value` data the model does not type. Documents carrying these keys now parse instead of failing with unknown-field errors.
- Build and smoke-test the Apple Silicon CI jobs on macOS 15 because the macOS 14 hosted runners were retired.
- Drop macOS Intel support; releases and CI now target Apple Silicon (`aarch64-apple-darwin`) only. Existing Intel Mac installs keep working through already-published 0.3.0 artifacts.
- Check Cargo dependencies and the pinned Rust toolchain weekly with Dependabot, and use the repository toolchain in CI and local development.
- Run Homebrew tap formula validation on macOS 26 so Rust build dependencies have supported bottles.
- Use version-only GitHub release titles and verify the generated title in CI.

## [0.3.0]

Released on 2026-09-12.

- Test WinGet installation, command discovery, PDF generation, and uninstall on Windows; use an explicit output directory when testing older releases.

- Add opt-in WinGet update PR automation for `voxvanhieu.code-a-cv`, with Windows release smoke tests and initial submission instructions.
- Rewrite the contribution guide as a first-PR quick start and document branch naming prefixes and issue/PR label usage.

- Automatically start npm publication and Homebrew formula PR creation after stable releases; support the first npm publish in CI with a temporary token.

- Add npm packaging for `npx @voxvanhieu/code-a-cv`, native macOS release archives
  for Apple Silicon and Intel, and a cross-platform tested npm publication workflow.

- Restore the PR check requiring contributions under `themes/` to exclude files outside that directory.

- Rename `cac fmt --check` to `--dry-run`; formatting now writes files by default, removing the need for `--write`. Stdin continues to produce stdout.

- Make `cac fmt` preserve the source type and Markdown style, support JSON/YAML/TOML, and resolve the project root from settings. YAML/TOML formatting may remove comments.

### Section metadata and theme extensibility

- Accept `Kind:` and `Id:` below Markdown section headings; Markdown conversion
  emits these readable fields while continuing to accept legacy annotations.
- Preserve theme-defined section kinds in Markdown, JSON, YAML, TOML, and the
  shared theme view. Unrecognized categories use custom entries and generic
  layouts; themes can specialize by kind or ID without rebuilding the CLI.
- Rust API: `SectionKind` now includes `Named(String)` and no longer implements
  `Copy`. Unknown valid section identifiers are accepted rather than rejected.

### Added

* Add explicit Markdown profile/entry fields, mixed entry kinds, repeated section titles, multiline prose, nested and ordered lists, labeled contacts, single event dates, and unknown date endpoints
* Add `cac fmt` with content-preserving output, default in-place writing and `--dry-run`, plus `cac check --explain`
* Document Markdown authoring and provide editor snippets and a richer shared-theme fixture

* Enriched the default CV used by `cac init` and `cac theme init` with full contacts, a summary, multiple roles, and detailed project, publication, skill, and advisory examples

* Added a shared theme contract with complete CV fields, structured linked contacts, reusable flow/table/label helpers, and shared classic composition at `/.cac/base.typ`
* Added `cac theme test --shared` with content, order, and link checks across a bundled reusable-theme corpus
* Added optional Markdown section annotations for explicit IDs and semantic kinds, including semantic export and line diagnostics

* Added `cac theme init`, `cac theme test`, and `cac theme pack` for scaffolding, validating, previewing, hashing, and reproducibly packaging project-local themes
* Added the `themeProject` project marker, settings validation, and protections for the active development theme

* Added configurable build artifact naming through the `naming` setting
* Restored `cac schema` to synchronize the project-local settings schema and validate `settings.json`
* Made `cac init` and `cac build` keep `.cac/settings.schema.json` synchronized with the running binary

### Fixed

* Preserve summaries, punctuation, entry metadata, tags, and kinds in Markdown conversion; reject unsupported content and lossy conversions before writing
* Render locations, project/publication URLs, and publication dates in HTML as well as PDF
* Reject silently discarded skill dates and publication ranges, and malformed or unsupported JSON Resume input fields

* Limit classic theme date columns to entry titles so descriptions and highlights use the full width below

* Preserve locations and standalone project/publication URLs in PDFs, and use complete fallback for mixed table sections
* Allow oversized shared entries to continue across pages

* Reject unsafe theme paths, symlinked development directories, and manifests that hash themselves
* Escape theme README metadata as literal text and verify the complete packaged file inventory
* Validate theme initialization arguments before prompting and protect dangling targets from writes
* Stage generated theme artifacts without rewriting sources and clean up temporary ZIPs on failure
* Require a selected theme in the settings schema when `themeProject` is present

### Changed

* Make missing contact details advisory; allow optional organizations and institutions to be omitted in all native formats
* Infer Markdown section kinds only from documented conventional titles; use an explicit section annotation for other titles previously matched by English substrings
* Preserve omitted JSON Resume period endpoints as unknown instead of inferring Present; reject JSON Resume exports that cannot preserve the complete CV
* Extend shared rich nodes with paragraphs, breaks, and lists, add prose entries, and project ordered entry content into description metadata; shared themes must pass the expanded corpus

* Remove theme version declarations and checks; themes use the shared CV data and styling contract, with defaults for omitted override dictionaries

* Renamed the `cac themes` command to `cac theme`
* Made generated settings reference `.cac/settings.schema.json` for editor validation and completion
* Grouped rendering settings under `page`, `typography`, `style`, `spacing`, and `pagination` while keeping `root` and `theme` at the top level
* Generate the project-local settings schema from the internal Rust model instead of tracking a standalone schema file
* Rename the default generated-document directory from `dist/` to `offering/`
* Move the runnable example projects under `docs/examples/`

## [0.2.0]

### Added

* Added standard input support to `build`, `check`, and `convert`
* Added explicit standard output support with `convert -o -`
* Added `cac init --format` with Markdown, YAML, JSON, and TOML output
* Added complete command help, examples, and input format selection
* Added layered Typst themes with an embedded `classic` design, validated `settings.json` overrides, and project and user theme directories
* Added a checksummed downloadable theme registry with `list`, `search`, `info`, `install`, and `remove` commands
* Added the downloadable `classic-blue` and `classic-left` themes

### Changed

* Made `check` print a final `PASS` or `FAIL`
* Made `cac init` create `settings.json` with the root CV and embedded `classic` theme selected
* Made `cac build` resolve an omitted input from the `root` settings property
* Split theme definitions into design tokens, semantic styles, and page configuration
* Reserved the system theme names `classic`, `base`, and `main` from installation
* Documented output replacement rules and exit statuses

### Removed

* Removed the `cac schema` command and JSON Schema generation
* Removed JSON from `cac build --format`; use `cac convert --to json` instead

## [0.1.1]

### Changed

* Made `cac` the package, binary, and command name
* Added source-building Homebrew tap publication before Homebrew Core eligibility

## [0.1.0]

### Added

* Five-crate `cac` workspace with a filesystem-free core model
* Markdown, YAML, JSON, TOML, and JSON Resume input
* Embedded Typst PDF/A-2b rendering and self-contained HTML rendering
* `init`, `build`, `check`, `convert`, `schema`, and `theme` commands
* Generated JSON Schema and deterministic PDF identifiers and timestamps
* Content checks for the seven planned source rules
* Cross-platform CI and `cargo-dist` release packaging for Linux, macOS, and Windows
* Homebrew Core source formula preparation workflow
* Equivalent example CVs for every supported input format and JSON Resume import
* MIT License

### Changed

* Replaced the initial release-action scaffold with the first usable `cac` milestone
* Expanded the quick start with installation, initialization, and build output details
