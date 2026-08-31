# Changelog

All notable changes to Code a CV are documented in this file.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and releases use [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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
