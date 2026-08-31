# Code a CV

Still naming it like `CV_ElonMusk_Facebook_final_v2.pdf`? Just focus on your work experience, mark it down, and let `cac` do the rest. That’s the idea behind CV as Code.

Code a CV turns structured files like Markdown into polished CVs, making it easy to manage versions, customize content for different roles, and export consistently without manual reformatting.

## What it supports

* Write your CV in Markdown, YAML, JSON, TOML, or [JSON Resume](https://jsonresume.org/)
* Export your CV as PDF or HTML
* CV variants based on the job requirements
* Auto format with theme
* Check for missing contact details, empty sections, weak highlights, first-person wording, and inconsistent dates *(no LLM)*
* Rebuild the same PDF consistently

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

## Examples

See [`examples/`](examples/) for Markdown, YAML, JSON, TOML, and JSON Resume files.

## Development

See the [development guide](docs/development/README.md) for setup, architecture, testing, pull requests, and releases.

## Roadmap

* [x] Markdown and structured data input
* [x] PDF and HTML output
* [x] JSON Resume import and export
* [x] CV content checks
* [x] Cross-platform releases
* [ ] More themes
* [ ] Rebuild automatically when files change
* [ ] Create job-specific CV versions with tags
* [ ] Fit a CV to a page limit
* [ ] Check the rendered PDF for missing content
* [ ] WebAssembly support

## License

Code a CV is available under the [MIT License](LICENSE).
