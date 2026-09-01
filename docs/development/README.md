# Development

This guide covers the path from a new checkout to a published release. See
[`CONTRIBUTING.md`](../../CONTRIBUTING.md) for project scope, design rules, and pull request expectations.

## Prerequisites

Install these tools before working on the project:

* Git
* Rust 1.92 or newer through [rustup](https://rustup.rs/)
* The `rustfmt` and Clippy components

`cac` does not require Python, LaTeX, Typst, or another runtime program. The binary embeds its Typst template and fonts.

Install the Rust components with:

```console
$ rustup toolchain install 1.92 --component rustfmt,clippy
```

## Set up the repository

Clone the repository and enter its directory:

```console
$ git clone https://github.com/voxvanhieu/code-a-cv.git
$ cd code-a-cv
```

Build the workspace and run every test:

```console
$ cargo test --workspace --all-features --locked
```

Run the development binary without installing it:

```console
$ cargo run -p cac -- --help
```

To exercise the default workflow in the repository directory, create and build the ignored `cv.md` file:

```console
$ cargo run -p cac -- init
$ cargo run -p cac -- build
```

The initialization also writes `settings.json` with `cv.md` as the root CV and
the embedded `classic` theme selected. It synchronizes the bundled settings
schema to `.cac/settings.schema.json`; builds refresh that file when needed.
The build writes `offering/cv.pdf`. Both `cv.md` and `offering/` are ignored by Git.

## Architecture

The repository is a Cargo workspace with five crates. Dependencies point inward toward `cac-core`; the core model does not access the filesystem or invoke renderers.

| Crate | Responsibility |
|---|---|
| `cac-core` | CV document model, dates, periods, and inline rich text |
| `cac-io` | Markdown and structured format parsing, validation, serialization, and JSON Resume conversion |
| `cac-render` | HTML generation and the Typst-to-PDF boundary |
| `cac-check` | Read-only content diagnostics |
| `cac` | Command-line interface, filesystem access, standard streams, and command dispatch |

A command follows this data flow:

1. `clap` parses arguments in `crates/cac/src/cli.rs` and the selected command module
2. `cac` reads a file or standard input and asks `cac-io` to produce a `CvDocument`
3. `cac-io` validates the document after parsing
4. `cac-check` inspects the document or `cac-render` converts it to PDF or HTML
5. `cac` writes document content, diagnostics, or artifacts to the requested destination

### Source layout

| Path | Purpose |
|---|---|
| `crates/cac/src/main.rs` | Process entry point and exit status handling |
| `crates/cac/src/cli.rs` | Root `clap` parser, shared format values, and dispatch |
| `crates/cac/src/commands/` | Arguments, help text, and execution for each command |
| `crates/cac/src/source.rs` | File, stdin, stdout, and input format handling |
| `crates/cac/src/error.rs` | Errors reported by the executable |
| `crates/cac-core/src/document.rs` | CV, section, and entry types |
| `crates/cac-core/src/date.rs` | Dates, periods, ordering, and serialization |
| `crates/cac-core/src/rich_text.rs` | Inline Markdown AST and rendering helpers |
| `crates/cac-io/src/codec.rs` | Native format dispatch and Markdown conversion |
| `crates/cac-io/src/json_resume.rs` | JSON Resume import and export |
| `crates/cac-render/src/html.rs` | Escaped HTML output |
| `crates/cac-render/src/pdf.rs` | Typst world, normalized render view, and PDF output |
| `crates/cac-render/src/settings.rs` | Rendering settings parsing and validation |
| `crates/cac-render/src/typst/base.typ` | Stable Typst rendering and component API |
| `crates/cac-render/src/typst/themes/classic.typ` | Embedded default theme |
| `crates/cac-check/src/lib.rs` | Diagnostic definitions and content rules |

Each library crate exposes its supported API through `src/lib.rs`. Keep implementation modules private unless downstream crates need the type or function.

### Rendering safety

The PDF renderer serializes a normalized view of `CvDocument` as a virtual `cv.json` file. It never interpolates CV text into Typst source.

The HTML renderer walks the same `RichText` tree and escapes text and attribute values at the point where it writes them. Preserve these boundaries when adding fields or rich-text nodes.

Typst dependencies are pinned to `0.15.1` because their Rust API is not stable. Review renderer changes carefully when updating that version.

## Make a change

Create a focused branch from an up-to-date `main` branch. Keep behavior changes separate from unrelated cleanup.

Use this map to find the first place to edit:

| Change | Start here |
|---|---|
| Add or change a command | `crates/cac/src/commands/` and `crates/cac/src/cli.rs` |
| Change the CV data model | `crates/cac-core/src/document.rs` |
| Change date handling | `crates/cac-core/src/date.rs` |
| Change Markdown parsing or output | `crates/cac-io/src/codec.rs` |
| Change JSON Resume conversion | `crates/cac-io/src/json_resume.rs` |
| Change HTML output | `crates/cac-render/src/html.rs` |
| Change PDF layout | `crates/cac-render/src/typst/` and `crates/cac-render/src/pdf.rs` |
| Add a content check | `crates/cac-check/src/lib.rs` |

When behavior visible to users changes:

* Add or update an integration test
* Update the relevant help snapshot when CLI help changes intentionally
* Update the root `README.md` when commands, formats, installation, or workflows change
* Add an entry under `Unreleased` in `CHANGELOG.md`

Do not hand-edit generated files such as `.github/workflows/release.yml`. Change `dist-workspace.toml` and regenerate the workflow with the configured `cargo-dist` version.

## Test and verify

Tests live in each crate's `tests/` directory. The executable tests run the compiled `cac` binary and verify help text, standard streams, files, and exit statuses.

| Test path | Coverage |
|---|---|
| `crates/cac/tests/` | Commands, help snapshots, stdin, stdout, files, and exit statuses |
| `crates/cac-core/tests/` | Rich text, dates, and periods |
| `crates/cac-io/tests/` | Examples, parsing, validation, and format round trips |
| `crates/cac-render/tests/` | HTML escaping and reproducible PDF output |
| `crates/cac-check/tests/` | Diagnostic rules and stable diagnostic codes |

Format code during development with:

```console
$ cargo fmt --all
```

Before opening a pull request, run the same quality checks as CI:

```console
$ cargo fmt --all --check
$ cargo test --workspace --all-features --locked
$ cargo clippy --workspace --all-features --all-targets --locked -- -D warnings
```

Build and smoke-test an optimized local binary when changing dependencies, rendering, packaging, or startup behavior:

```console
$ cargo build --release --locked --package cac
$ target/release/cac --version
```

The release profile uses fat link-time optimization, so a clean release build can take several minutes.
On Windows, run `target\release\cac.exe --version` for the smoke test.

## Open a pull request

Push the branch and open a pull request against `main`. Describe the user-visible result, compatibility impact, and commands used for verification.

For code changes, CI performs these checks:

* Formatting, workspace tests, and Clippy on Linux
* Release builds and version smoke tests on Linux x64, Linux ARM64, macOS x64, macOS ARM64, and Windows x64
* A `cargo-dist` release plan

Wait for the aggregate `CI complete` job before merging.

## Create a release

See the [release and Homebrew tap guide](release.md) for the release architecture, one-time tap setup, GitHub release steps, bottle publication, and Homebrew Core migration.

## Update release tooling

The generated release workflow and `dist-workspace.toml` must use the same `cargo-dist` version.

When changing release configuration, edit `dist-workspace.toml` and run:

```console
$ dist generate
```

Review the generated `.github/workflows/release.yml` before committing it. Use `dist init` only when upgrading or reconfiguring `cargo-dist`, then review every generated change.
