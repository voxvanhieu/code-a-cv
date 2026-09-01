# Changelog

All notable changes to Code a CV are documented in this file.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and releases use [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.0]

### Added

* Added standard input support to `build`, `check`, and `convert`
* Added standard output support to `init` and explicit `convert -o -`
* Added complete command help, examples, and input format selection
* Added layered Typst themes with embedded `classic` and `classic-left` designs, validated `settings.json` overrides, and project and user theme directories
* Added `cac themes list`, `install`, and `remove` commands

### Changed

* Made `check` print a final `PASS` or `FAIL`
* Made `cac init` create `settings.json` with the embedded `classic` theme selected
* Split theme definitions into design tokens, semantic styles, and page configuration
* Documented output replacement rules and exit statuses

### Removed

* Removed the `cac schema` command and JSON Schema generation
* Removed JSON from `cac build --format`; use `cac convert --to json` instead
* Removed the global `--json` command-output option

## [0.1.1] - 2026-08-31

### Changed

* Made `cac` the package, binary, and command name
* Added source-building Homebrew tap publication before Homebrew Core eligibility

## [0.1.0] - 2026-08-30

### Added

* Five-crate `cac` workspace with a filesystem-free core model
* Markdown, YAML, JSON, TOML, and JSON Resume input
* Embedded Typst PDF/A-2b rendering and self-contained HTML rendering
* `init`, `build`, `check`, `convert`, `schema`, and `themes` commands
* Generated JSON Schema and deterministic PDF identifiers and timestamps
* Content checks for the seven planned source rules
* Cross-platform CI and `cargo-dist` release packaging for Linux, macOS, and Windows
* Homebrew Core source formula preparation workflow
* Equivalent example CVs for every supported input format and JSON Resume import
* MIT License

### Changed

* Replaced the initial release-action scaffold with the first usable `cac` milestone
* Expanded the quick start with installation, initialization, and build output details
