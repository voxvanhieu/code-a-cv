# Repository Guidelines

## Project Structure and Module Organization

This repository is a Rust 2024 workspace for the `cac` command-line application. The crates have distinct responsibilities:

* `crates/cac-core`: CV data types, dates, and rich text
* `crates/cac-io`: Markdown, YAML, JSON, TOML, and JSON Resume codecs
* `crates/cac-render`: HTML and Typst-based PDF rendering
* `crates/cac-check`: read-only content diagnostics
* `crates/cac`: CLI parsing, command dispatch, and filesystem access

Integration tests live in each crate's `tests/` directory. Example CV projects are in `docs/examples/`, downloadable themes are in `themes/`, and architecture notes are in `docs/development/`. Keep public APIs in each crate's `src/lib.rs` and implementation details private where possible.

## Build, Test, and Development Commands

Use Rust 1.92 or newer with `rustfmt` and Clippy installed.

```console
$ cargo run -p cac -- --help
$ cargo test --workspace --all-features --locked
$ cargo fmt --all --check
$ cargo clippy --workspace --all-features --all-targets --locked -- -D warnings
$ cargo build --release --locked --package cac
```

The first command runs the CLI locally. The next three match CI quality checks. The final command creates an optimized binary at `target/release/cac`.

## Coding Style and Naming Conventions

Run `cargo fmt --all` before submitting changes. Follow standard Rust naming: `snake_case` for modules, functions, and tests; `CamelCase` for types and traits; `SCREAMING_SNAKE_CASE` for constants. Put `use` declarations at module scope. Prefer clear types and focused modules over fully qualified paths or explanatory comments.

Preserve rendering safety: escape rich text in each renderer, never interpolate CV strings into Typst source, and do not grant themes filesystem or shell access.

## Testing Guidelines

Add integration tests for every behavior change. Place them in the affected crate and name test functions after observable behavior, such as `rejects_invalid_date_range`. Cover format round trips, Unicode, special characters, and minimal and complete documents. Update CLI snapshots in `crates/cac/tests/snapshots/` only when help output changes intentionally.

## Commit and Pull Request Guidelines

Recent commits use short imperative subjects, often with a pull request number, for example `Improve CI path checks (#5)`. Keep commits and pull requests focused. PR descriptions must state the user-visible result, compatibility impact, and verification commands. Link relevant issues, update documentation for changed workflows, and add user-visible changes under `Unreleased` in `CHANGELOG.md`.
