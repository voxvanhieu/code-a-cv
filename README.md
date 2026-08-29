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

Build and install the binary from the workspace:

```console
$ cargo install --path crates/cac-cli
$ cac init
$ cac build
```

Tagged releases publish prebuilt archives and installers for Linux, macOS, and Windows on GitHub Releases.

`cac init` creates `cv.md`. Its headings match the visible CV:

```markdown
# Ada Lovelace

ada@example.com · London, United Kingdom

## Experience

### Software Engineer, Analytical Engines Ltd
Jan 2023–Present

- Reduced build time by **35%**
```

`cac build` creates `dist/cv.pdf`. Build more formats with:

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

The license has not been selected yet.
