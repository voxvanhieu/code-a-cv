# Instructions for AI Agents

## Project Overview

`rust-build-package-release-action` is an opinionated GitHub Action that automates release workflows for Rust projects,
built as a single Rust binary crate with clap subcommands (edition 2024, rust-version 1.85).

This is a conventions-based, opinionated release process extracted from:

 * [rabbitmq/rabbitmqadmin-ng](https://github.com/rabbitmq/rabbitmqadmin-ng)
 * [michaelklishin/rabbitmq-lqt](https://github.com/michaelklishin/rabbitmq-lqt)
 * [michaelklishin/frm](https://github.com/michaelklishin/frm)

## Testing

Run all tests:

```bash
cargo test
```

Run clippy:

```bash
cargo clippy --all-targets -- -D warnings
```

See `CONTRIBUTING.md` as well.

For verifying YAML file syntax, use `yq`, Ruby or Python YAML modules (whichever is available).

## Key Files

 * `action.yml`: GitHub Action definition
 * `Cargo.toml`: crate manifest
 * `src/main.rs`: clap CLI and command dispatcher (routes subcommands to modules)
 * `src/lib.rs`: shared utilities (`env_or`, `parse_comma_list`) and module re-exports
 * `src/error.rs`: error type and `Result` alias
 * `src/output.rs`: GitHub Actions output helpers (`output`, `output_multiline`, `print_hr`)
 * `src/platform.rs`: target triple parsing and platform detection
 * `src/metadata/`: Cargo package metadata, changelog, and version handling
 * `src/packaging/`: nfpm configuration and Homebrew, AUR, and Winget manifest generation
 * `src/release/`: release build, archive, collection, download, formatting, publishing, and package testing
 * `src/release/security/`: checksum, SBOM, and Sigstore artifact security operations
 * `src/tools.rs`: external tool installation (cross, cargo-zigbuild, nfpm, etc.)
 * `tests/test_helpers.rs`: shared test utilities (`create_temp_file`, `create_temp_text_file`)
 * `tests/*_tests.rs`: unit tests for each module
 * `tests/*_proptests.rs`: property-based tests (proptest) for version, changelog, platform
 * `examples/*.yml`: workflow examples for common scenarios

## macOS Support

Building macOS binaries requires a native runner:

 * Intel Mac (`x86_64-apple-darwin`): use `macos-15-intel` or `macos-26-intel` runners (`macos-13` was removed in December 2025)
 * Apple Silicon (`aarch64-apple-darwin`): use `macos-14` or newer runners for native compilation
 * Building for Intel on Apple Silicon runners is technically possible but not recommended (cross-compilation adds complexity)
 * Each architecture needs a separate build job with its corresponding runner
 * The action generates Homebrew formulas that auto-select the correct binary per architecture
 * Platform detection in `src/platform.rs` already handles both: `macos-x64` and `macos-arm64`

For workflows, use a matrix strategy to build both in parallel. See `examples/macos-multi-arch.yml` for a complete example.

## Rust Style

 * Use `use` statements at the top module level, never in function scope
 * Avoid fully qualified type paths (e.g. `std::fmt::Display`) unless needed for disambiguation
 * All tests go in `tests/` as integration tests, not inline `#[cfg(test)]` modules
 * Use `LazyLock<Mutex<()>>` to serialise tests that modify process-wide state (env vars, CWD)
 * Mark `env::set_var` and `env::remove_var` calls as `unsafe` with a safety comment
 * Format with `cargo fmt --all`, lint with `cargo clippy --all-features --all`

## Git and GitHub Conventions

 * Do not commit changes automatically without an explicit permission to do so
 * Never add yourself to commit co-authors list
 * Never mention yourself in commit messages
 * If a `gh` operation fails with an authentication failure, unset `GITHUB_TOKEN` and retry once, as `gh` may be configured to fetch the token differently

## Comments, Writing Style and Voice

Only add very important comments to the tests and the implementation.

### Voice

Write like an engineer who values clarity and simplicity. This applies
to all prose: design docs, analyses, notes, and commit messages.

 * Plain and factual: state the why in one line, never narrate the what
 * Literal mechanism over metaphor: name the actual thing, not an image of it
 * Prefer the plainest word. No coined verbs, no jargon for its own sake
 * No flourish, no editorializing, no imagery. Real domain terms are fine
 * If a sentence needs a second clause to justify itself, it is probably too clever
 * Plain full sentences over compressed clever noun phrases: "a helper
   crate", not "a `tower`-shaped convenience"
 * State guarantees and behavior explicitly; do not leave them implied
   by jargon
 * Name tools and platforms precisely: `rustc` 1.92, edition 2024,
   crates.io, WebAssembly
 * No bold for emphasis; bold is for structural labels only, and sparingly
 * No "term — explanation" em-dash glosses: use ": " or parentheses
 * These vocabulary rules apply to identifiers too: test function names,
   helper modules, and fixture names use the same plain words as prose

### Writing and Markdown Style

 * Never add full stops to Markdown list items
 * Use "X and Y" in prose, not "X / Y" slash-shorthand. Exceptions: unit
   fractions (`bytes/sec`), single-concept abbreviations (`I/O`), and paths
   or code (`tests/unit/`, `src/lib.rs`)
 * Wrap code identifiers in backticks in prose: types like `Vec<T>`, traits
   like `Display`, functions like `Iterator::next`, modules, file names, and paths
 * Avoid robotic labels such as `**Thing / other:**`; write a plain sentence or a simple label
 * Match the existing conventions of the file and subdirectory you are
   editing: bullet character, heading depth, ID schemes, and table shape
   vary by project, and the local choice wins

## Releases

### How to Roll (Produce) a New Major Release

GitHub Actions use major version tags (`@v1`, `@v2`, `@v3`) as their public API.
Consumers pin to `@vN` and automatically receive all non-breaking updates.

Suppose the current development version in `Cargo.toml` is `N.0.0` and `CHANGELOG.md` has
a `## vN.0.0 (in development)` section at the top.

To produce a new major release:

 1. Update the changelog: replace `(in development)` with today's date, e.g. `(Mar 22, 2026)`. Make sure all notable changes since the previous release are listed
 2. Commit with the message `N.0.0` (just the version number, nothing else)
 3. Tag the commit: `git tag vN.0.0`
 4. Move the floating major tag to the same commit: `git tag -f vN`
 5. Bump the dev version: set `Cargo.toml` version to `(N+1).0.0`
 6. Add a new `## v(N+1).0.0 (in development)` section to `CHANGELOG.md` with `No changes yet.` underneath
 7. Commit with the message `Bump dev version`
 8. Push: `git push && git push --tags --force`

The `--force` on the tag push is required because the floating `vN` tag already exists
on the remote and must be moved forward.

### Notes

 * This crate is `publish = false`: there is no crates.io publishing step
 * The floating major tag (e.g. `v2`) is what consumers reference in their workflows
 * The precise tag (e.g. `v2.0.0`) is for changelog traceability

## Iterative Post-Implementation Review (IPIR)

Review the changes very carefully and holistically for correctness and safety,
opportunities to meaningfully simplify the implementation without losing
fidelity and effectiveness, the use of Rust idioms, the rich type system
patterns, meaningful test coverage, API usability and whether the changes are
worth adopting to begin with.

Look hard for ways to meaningfully improve both the tests and the implementation.

Perform 5 such iterations (holistic analysis runs).
