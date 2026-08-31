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

Winget support is planned but not published yet. For now, download `cac-x86_64-pc-windows-msvc.zip` from the [latest release](https://github.com/voxvanhieu/code-a-cv/releases/latest), extract `cac.exe`, and add its directory to `PATH`.

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
