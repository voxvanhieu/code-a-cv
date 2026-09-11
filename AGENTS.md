# Repository Guidelines

## Project Structure & Module Organization

Code a CV (`cac`) is a Rust workspace that converts CV data into PDF and HTML. Under `crates/`, `cac` owns CLI commands and filesystem access; `cac-core` defines document types; `cac-io` parses and serializes formats; `cac-render` produces output; and `cac-check` provides diagnostics. Keep core types independent of filesystem and rendering concerns.

Each crate has `src/` and `tests/`. Embedded Typst templates live in `crates/cac-render/src/typst/`, shared themes in `themes/`, rendering fixtures in `crates/cac-render/fixtures/shared/`, and guides and examples in `docs/`.

## Build, Test, and Development Commands

Use Rust 1.92 or newer with `rustfmt` and Clippy; no separate Typst installation is required.

- `cargo build -p cac --locked`: build the local CLI.
- `cargo run -p cac -- --help`: explore development commands.
- `cargo test -p cac-io --locked`: run an affected crate's tests.
- `cargo fmt --all`: format Rust code.
- `cargo fmt --all --check`: verify formatting.
- `cargo test --workspace --all-features --locked`: run the full suite.
- `cargo clippy --workspace --all-features --all-targets --locked -- -D warnings`: lint with warnings treated as errors.

Run the last three checks before submitting code changes. Prose-only edits require Markdown, link, and example review.

## Coding Style & Naming Conventions

Follow Rust 2024 conventions and rustfmt's four-space indentation. Use `snake_case` for modules and functions, `PascalCase` for types, and `SCREAMING_SNAKE_CASE` for constants. Keep implementation modules private unless another crate needs their API. Put `use` declarations at module scope.

## Testing Guidelines

Use Rust `#[test]` tests; CLI integration tests use `assert_cmd` and `predicates`. Add integration tests for behavior changes in the affected crate's `tests/`, using descriptive names such as `period_rejects_reverse_dates`. Cover relevant Unicode, escaping, minimal-input, and round-trip cases. Update help snapshots only for intentional changes. No numeric coverage threshold is documented.

## Commit & Pull Request Guidelines

Use short imperative commits, matching history: `Add configurable build artifact names`. Name focused branches like `fix/escape-html-links` and target `main`. Describe the problem, resulting behavior, related issues, compatibility impact, and verification results. Include previews for visual changes and update `CHANGELOG.md` under `Unreleased` for user-visible changes.

PRs touching `themes/` must contain only theme files; submit supporting code, documentation, and changelog changes separately. Follow `CONTRIBUTING.md` and wait for review and `CI complete` before merging.

## Rendering Safety

Escape rich text in each renderer. Never interpolate CV strings into Typst source or grant themes filesystem or shell access. Keep `cac check` read-only.
