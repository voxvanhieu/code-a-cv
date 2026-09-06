# Changelog

All notable changes to Code a CV are documented in this file.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and releases use [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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

* Limit cargo-dist release binaries to Linux (x64 and ARM64) and Windows (x64); use Homebrew for macOS installation
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
