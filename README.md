# Code a CV

Code a CV helps you manage your CV(s) as Code. Leverage source control to track every change and never lose a version again, tailor your CV to match different job requirements without duplicating work, and let automated formatting keep every version polished and consistent — no manual reformatting, ever.

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
| `cac build [FILE]` | Validate and render a CV as PDF or HTML |
| `cac check [FILE] [--strict]` | Check source content |
| `cac convert <FILE> --to <FORMAT>` | Convert supported input formats |
| `cac themes` | List embedded themes |

## Command-line behavior

`cac` writes operational messages for people. JSON is a CV document format, not a command-output protocol. Use `cac convert cv.md --to json` to write the native JSON representation or `--to jsonresume` to export JSON Resume data.

`build`, `check`, and `convert` accept `-` as standard input. Standard input is treated as Markdown unless `--input-format markdown|yaml|json|toml|jsonresume` is supplied. `convert` writes to standard output when `--output` is omitted or set to `-`. `init -o -` writes the starter or imported Markdown to standard output.

`init` refuses to replace a file unless `--force` is supplied. `build` replaces artifacts with the same names in its output directory, and `convert -o FILE` replaces `FILE`.

Exit status `0` means success, `1` means a runtime, parsing, rendering, or check failure, and `2` means invalid command syntax. `check` prints a final `PASS` or `FAIL`; `--strict` makes any diagnostic fail the command.

The current interface does not include verbosity or color controls. They should only be added when commands produce enough variable detail to require them.

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
