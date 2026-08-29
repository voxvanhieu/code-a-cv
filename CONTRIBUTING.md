# Contributing to Code a CV

Thank you for contributing to `cac`. The project creates PDF and HTML CVs from structured data without requiring a TeX installation or a runtime dependency.

## Scope

Keep the core workflow simple:

```console
$ cac init
$ cac build
```

The installed tool must work locally without required configuration or external programs. The first usable milestone supports Markdown input by default, plus YAML, JSON, and TOML; PDF and HTML output; JSON Resume conversion; one embedded theme; and source checks. File watching, tagged selections, page fitting, rendered checks, and additional themes remain roadmap work.

Do not add LaTeX output, a theme registry, an LSP, a web playground, or AI writing features without prior discussion.

## Development quick start

Install stable Rust with [rustup](https://rustup.rs/), then run:

```console
$ cargo test
$ cargo fmt --all --check
$ cargo clippy --all-features --all-targets -- -D warnings
```

See [the development guide](docs/development/README.md) for the full setup and architecture notes.

## Making changes

Keep pull requests focused. Include tests for changed behaviour and describe the user-visible result, compatibility impact, and verification commands.

Run the formatter, test suite, and Clippy before opening a pull request:

```console
$ cargo fmt --all
$ cargo test
$ cargo clippy --all-features --all-targets -- -D warnings
```

Do not commit generated build output. Update `README.md` when commands, formats, workflows, or installation methods change. Add user-visible changes to `CHANGELOG.md` under `Unreleased`.

## Testing

Add integration tests and focused fixtures for parsing, conversion, rendering, and checks. Cover minimal and complete CVs, Unicode text, long careers, and special characters such as `C#`, `100%`, paths, and emoji.

Use property tests for format round trips and snapshots for stable text and HTML output. Changes that may affect rendering or checking performance should include a measurement.

## Design rules

* Keep the binary self-contained
* Keep configuration optional
* Keep Markdown headings aligned with the visible CV sections
* Escape rich text in each renderer
* Do not interpolate CV strings into Typst source
* Do not let `--fit` remove CV content
* Keep `cac check` read-only
* Do not give themes file-system or shell access

## Pull requests

Use a specific imperative title. Do not combine unrelated refactors with a behaviour change. Maintainers may request narrower scope, additional tests, a benchmark, or documentation before merging.

## Security

Report suspected security issues privately to the maintainers. Do not file public issues for problems involving CV data, theme isolation, PDF output, dependencies, or release artifacts.
