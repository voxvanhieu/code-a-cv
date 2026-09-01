# Code a CV

[![CI](https://github.com/voxvanhieu/code-a-cv/actions/workflows/ci.yaml/badge.svg?branch=main)](https://github.com/voxvanhieu/code-a-cv/actions/workflows/ci.yaml)
[![Latest release](https://img.shields.io/github/v/release/voxvanhieu/code-a-cv?label=release)](https://github.com/voxvanhieu/code-a-cv/releases/latest)
[![Downloads](https://img.shields.io/github/downloads/voxvanhieu/code-a-cv/total)](https://github.com/voxvanhieu/code-a-cv/releases)
[![Platforms](https://img.shields.io/badge/platforms-macOS%20%7C%20Linux%20%7C%20Windows-blue)](https://github.com/voxvanhieu/code-a-cv/releases/latest)
[![Homebrew](https://img.shields.io/badge/Homebrew-voxvanhieu%2Ftap-yellow?logo=homebrew)](https://github.com/voxvanhieu/homebrew-tap)
[![License](https://img.shields.io/github/license/voxvanhieu/code-a-cv)](LICENSE)

Still naming it like `CV_ElonMusk_Facebook_final_v2.pdf`? Just focus on your work experiences, mark it down, and let `cac` do the rest. That’s the idea behind CV as Code.

Code a CV turns structured files like Markdown into polished CVs, making it easy to manage versions, customize content for different roles, and export consistently without manual reformatting.

> ⭐ Your [star](https://github.com/voxvanhieu/code-a-cv) can help make Code a CV better

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

<details>
<summary>Other installations</summary>

### Windows

Winget installation will be available in a future release. For now, download `cac-x86_64-pc-windows-msvc.zip` from the [latest release](https://github.com/voxvanhieu/code-a-cv/releases/latest), extract `cac.exe`, and add its directory to `PATH`.

### Linux

The Homebrew command above also works on Linux. You can also build `cac` from source.

### Install from source

Install [Rust 1.92 or newer](https://rustup.rs/), then run:

```sh
git clone https://github.com/voxvanhieu/code-a-cv.git
cd code-a-cv
cargo install --path crates/cac --locked
```

</details>

Create a CV:

```sh
mkdir my-cv
cd my-cv
cac init
```

This creates `cv.md` and `settings.json`. The settings identify `cv.md` as the
root CV and select the embedded `classic` theme. Edit `cv.md`, then build it:

```sh
cac build
```

The finished PDF is `offering/cv.pdf`—your carefully prepared offering to the hiring gods.

## Common commands

| Command | Description |
|---|---|
| `cac init [--format FORMAT] [--from resume.json]` | Create a starter CV and classic theme settings or import JSON Resume data |
| `cac build [FILE]` | Validate and render a CV as PDF or HTML |
| `cac check [FILE] [--strict]` | Check source content |
| `cac convert <FILE> --to <FORMAT>` | Convert supported input formats |
| `cac themes list` | List embedded and installed themes |
| `cac themes search [QUERY]` | Search downloadable themes |
| `cac themes info <THEME>` | Show downloadable theme metadata |
| `cac themes install <THEME> [--local]` | Install a downloadable theme |
| `cac themes remove <THEME> [--local]` | Remove an installed theme |

The `root` property in `settings.json` selects the CV source used by `cac build` when no file is provided. This avoids ambiguity when a directory contains several supported CV files. Put common PDF formatting overrides in the same file. Internally, `cac init` and `cac build` keep the project-local settings schema current for editor completion and validation. Use a project-local theme under `.cac/themes/<name>/theme.typ` for structural customization. See the [theme architecture](docs/development/theme-architecture.md) for the supported settings and theme API, and [`themes/`](themes/) for the contribution process.

The theme names `classic`, `base`, and `main` are reserved by `cac` and cannot be installed. The default `classic` theme is embedded and available without installation. Install `classic-left` from the downloadable theme registry when needed.

## Contribute your theme

Add a theme directory with `theme.typ` and `theme.json`, register it in `themes/index.json`, and include SHA-256 checksums for every downloadable file. See the [theme contribution guide](themes/README.md) for the Theme API contract, manifest format, validation rules, and local testing commands.

## Examples

See the runnable [`docs/examples/`](docs/examples/) projects for Markdown, structured formats,
and JSON Resume import workflows.

## Development

[![Rust 1.92+](https://img.shields.io/badge/Rust-1.92%2B-orange?logo=rust)](Cargo.toml)

See the [development guide](docs/development/README.md) for setup, architecture, testing, pull requests, and releases.

## Todo list

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
* [ ] Stunning local Web UI with Tauri
* [ ] VS Code extension

## License

Code a CV is available under the [MIT License](LICENSE).
