# Code a CV

`cac` builds CVs from Markdown, YAML, JSON, or TOML. It ships as one binary with an embedded Typst renderer and embedded fonts, so it does not require Python, LaTeX, or a separate typesetting program.

## What it provides today

* PDF/A-2b and self-contained HTML output from one CV source file
* Markdown source that mirrors the visible CV sections
* Editor completion and validation for structured formats through a JSON Schema
* Rich text for links, emphasis, and code without renderer-specific escaping
* JSON Resume import and export
* Deterministic PDF output
* Seven source checks for contact details, empty sections, highlights, first-person pronouns, and date consistency

## Quick start

### Install `cac`

After the formula is accepted into Homebrew Core, install it on macOS or Linux:

```console
$ brew install code-a-cv
```

Homebrew builds bottles for its supported platforms after the formula is accepted.

To install from source, first install the Rust toolchain, then run:

```console
$ cargo install --locked --git https://github.com/voxvanhieu/code-a-cv cac-cli
```

### Create your first CV

Create a directory for the CV and initialize it:

```console
$ mkdir my-cv
$ cd my-cv
$ cac init
```

`cac init` creates `cv.md`. Its headings match the visible CV:

```markdown
# Ada Lovelace

ada@example.com · London, United Kingdom

## Experience

### Software Engineer, Analytical Engines Ltd
Jan 2023–Present

- Reduced build time by **35%**
```

Edit `cv.md`, then build it:

```console
$ cac build
BUILT dist/cv.pdf
```

The PDF is at `dist/cv.pdf`, relative to the directory where you ran `cac build`.
The output name follows the input file name, so `cac build resume.md` creates
`dist/resume.pdf`.

Build more formats or choose another output directory with:

```console
$ cac build --format pdf,html,json --output dist
```

## Checks and conversion

Check the source. `--strict` makes warnings fail the command in CI:

```console
$ cac check
PASS
```

Convert between native formats or export JSON Resume:

```console
$ cac convert cv.md --to yaml --output cv.yaml
$ cac convert cv.md --to jsonresume --output resume.json
```

Print the generated JSON Schema with `cac schema`.

## Examples

The same CV is available in every native input format, plus JSON Resume for import:

| Format | Example | Command |
|---|---|---|
| Markdown | [`examples/cv.md`](examples/cv.md) | `cac build examples/cv.md` |
| YAML | [`examples/cv.yaml`](examples/cv.yaml) | `cac build examples/cv.yaml` |
| JSON | [`examples/cv.json`](examples/cv.json) | `cac build examples/cv.json` |
| TOML | [`examples/cv.toml`](examples/cv.toml) | `cac build examples/cv.toml` |
| JSON Resume | [`examples/resume.json`](examples/resume.json) | `cac init --from examples/resume.json --output imported.md` |

## Commands

| Command | Description |
|---|---|
| `cac init [--from resume.json]` | Create a starter CV or import JSON Resume data |
| `cac build [INPUT]` | Validate and render a CV |
| `cac check [INPUT]` | Check source content |
| `cac convert <IN> --to <FORMAT>` | Convert supported input formats |
| `cac schema` | Print the generated JSON Schema |
| `cac themes` | List embedded themes |

Commands support `--json` for automation.

Tagged selections, page fitting, file watching, rendered PDF checks, additional themes, and WebAssembly builds remain roadmap work.

## Development

Run:

```console
$ cargo fmt --all --check
$ cargo test --workspace
$ cargo clippy --all-features --all-targets -- -D warnings
```

See [the development guide](docs/development/README.md) for architecture details.

## License

Code a CV is available under the [MIT License](LICENSE).
