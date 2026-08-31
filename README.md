# Code a CV

`cac` builds CVs from Markdown, YAML, JSON, or TOML. It ships as one binary with an embedded Typst renderer and embedded fonts, so it does not require Python, LaTeX, or a separate typesetting program.

## What it provides today

* PDF/A-2b and self-contained HTML output from one CV source file
* Markdown source that mirrors the visible CV sections
* Rich text for links, emphasis, and code without renderer-specific escaping
* JSON Resume import and export
* Deterministic PDF output
* Seven source checks for contact details, empty sections, highlights, first-person pronouns, and date consistency

## Quick start

Install `cac` on macOS or Linux:

```sh
brew install voxvanhieu/tap/code-a-cv
```

Create a CV:

```sh
mkdir my-cv
cd my-cv
cac init
```

Edit `cv.md`, then build it:

```sh
cac build
```

The finished PDF is `dist/cv.pdf`.

## Common commands

| Command | Description |
|---|---|
| `cac init [--from resume.json]` | Create a starter CV or import JSON Resume data |
| `cac build [INPUT]` | Validate and render a CV as PDF or HTML |
| `cac check [INPUT] [--strict]` | Check source content |
| `cac convert <IN> --to <FORMAT>` | Convert supported input formats |
| `cac themes` | List embedded themes |

## Examples

See [`examples/`](examples/) for Markdown, YAML, JSON, TOML, and JSON Resume files.

## Development

```console
$ cargo fmt --all --check
$ cargo test --workspace
$ cargo clippy --all-features --all-targets -- -D warnings
```

See the [development guide](docs/development/README.md) for details.

## License

Code a CV is available under the [MIT License](LICENSE).
