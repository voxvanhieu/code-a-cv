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

The initialization also writes `settings.json` with `cv.md` as the root CV and the embedded `classic` theme selected. The build writes `dist/cv.pdf`. Both `cv.md` and `dist/` are ignored by Git.

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

Only maintainers with permission to push tags and run repository workflows can create a release. Releases use semantic versions such as `0.2.0` and tags such as `v0.2.0`.

### 1. Prepare the version

Choose the next version according to [Semantic Versioning](https://semver.org/). Update `workspace.package.version` in the root `Cargo.toml`, then let Cargo update the five workspace package entries in `Cargo.lock`:

```console
$ cargo check --workspace
```

Move the entries from `Unreleased` in `CHANGELOG.md` into a versioned section without a release date. Leave a new empty `Unreleased` section for later changes. `cargo-dist` uses the heading as the GitHub release title, so a heading such as `## [0.2.0]` produces the title `0.2.0`.

### 2. Verify the release candidate

Run the full locked checks and release build:

```console
$ cargo fmt --all --check
$ cargo test --workspace --all-features --locked
$ cargo clippy --workspace --all-features --all-targets --locked -- -D warnings
$ cargo build --release --locked --package cac
$ target/release/cac --version
```

The reported version must match the version in `Cargo.toml`.

Install the `cargo-dist` version configured in `dist-workspace.toml` if needed, then validate the release plan:

```console
$ cargo install cargo-dist --version 0.32.0 --locked
$ dist plan
```

### 3. Merge the release change

Open and merge a pull request containing the version, lockfile, and changelog updates. Confirm that all required CI jobs pass on the release commit.

### 4. Tag the release

Create and push a tag that exactly matches the workspace version:

```console
$ git switch main
$ git pull --ff-only
$ git tag v0.2.0
$ git push origin v0.2.0
```

Replace `0.2.0` with the prepared version. Do not reuse or move a published release tag.

The tag starts `.github/workflows/release.yml`. `cargo-dist` builds archives and installers for the five supported targets, creates checksums and build provenance attestations, and publishes the GitHub release.

### 5. Verify the GitHub release

Wait for the Release workflow to finish, then confirm:

* The GitHub release is published under the expected tag
* The GitHub release title is the version without a release date
* Linux, macOS, and Windows archives are present
* Shell and PowerShell installers are present
* Checksums and provenance attestations are present
* A downloaded binary prints the expected version

### 6. Publish the Homebrew tap formula

Run the `Publish Homebrew tap formula` workflow manually with the stable release tag. The workflow:

1. Downloads the published source archive and calculates its SHA-256 checksum
2. Generates the formula with `scripts/render-homebrew-formula.rb`
3. Builds, tests, audits, and styles the formula on Linux and macOS
4. Opens a pull request in `voxvanhieu/homebrew-tap`

The workflow requires the `HOMEBREW_TAP_TOKEN` repository secret. Review and merge the generated pull request before announcing the Homebrew update.

### 7. Prepare Homebrew Core when eligible

The `Prepare initial Homebrew Core formula` workflow is only for the first submission to Homebrew Core. It checks Homebrew's notability threshold, validates the formula on Linux and macOS, and pushes a review branch to `voxvanhieu/homebrew-core` using `HOMEBREW_CORE_TOKEN`.

Review that branch before opening the pull request to `Homebrew/homebrew-core`. After Homebrew Core accepts the formula, use Homebrew's normal version-update process and stop publishing duplicate formulas through the project tap.

## Update release tooling

The generated release workflow and `dist-workspace.toml` must use the same `cargo-dist` version.

When changing release configuration, edit `dist-workspace.toml` and run:

```console
$ dist generate
```

Review the generated `.github/workflows/release.yml` before committing it. Use `dist init` only when upgrading or reconfiguring `cargo-dist`, then review every generated change.
