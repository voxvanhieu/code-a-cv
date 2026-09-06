# Code a CV

[![CI](https://github.com/voxvanhieu/code-a-cv/actions/workflows/ci.yaml/badge.svg?branch=main)](https://github.com/voxvanhieu/code-a-cv/actions/workflows/ci.yaml)
[![Latest release](https://img.shields.io/github/v/release/voxvanhieu/code-a-cv?label=release)](https://github.com/voxvanhieu/code-a-cv/releases/latest)
[![Downloads](https://img.shields.io/github/downloads/voxvanhieu/code-a-cv/total)](https://github.com/voxvanhieu/code-a-cv/releases)
[![Platforms](https://img.shields.io/badge/platforms-macOS%20%7C%20Linux%20%7C%20Windows-blue)](https://github.com/voxvanhieu/code-a-cv/releases/latest)
[![Homebrew](https://img.shields.io/badge/Homebrew-voxvanhieu%2Ftap-yellow?logo=homebrew)](https://github.com/voxvanhieu/homebrew-tap)
[![License](https://img.shields.io/github/license/voxvanhieu/code-a-cv)](LICENSE)

Still naming it like `CV_ElonMusk_Facebook_final_v2.pdf`? Just focus on your work experiences, mark it down, and let `cac` do the rest. That’s the idea behind CV as Code.

Write your CV as code and render it as PDF or HTML. Keep your content in version
control, choose a theme, and rebuild consistently.

- Author in Markdown, or use JSON, YAML, and TOML.
- Import and export the supported [JSON Resume subset](docs/authoring/conversion.md).
- Check content and format source files without an LLM.
- Customize PDF layouts with reusable themes.

> ⭐ Your [star](https://github.com/voxvanhieu/code-a-cv) can help make Code a CV better

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

Edit the generated `cv.md`, then build your PDF:

```sh
cac build
```

Open `offering/cv.pdf`. Continue with the **[usage guide](docs/authoring/README.md)**
for authoring, settings, and output options.

## Write and maintain your CV

| Task                       | Command                       | Guide                                                                                    |
| -------------------------- | ----------------------------- | ---------------------------------------------------------------------------------------- |
| Start or import a CV       | `cac init`                    | [Getting started](docs/authoring/getting-started.md)                                     |
| Check content              | `cac check cv.md`             | [Content checks](docs/authoring/checking-and-formatting.md#check-content)                |
| Format the source in place | `cac fmt`                     | [Formatting and dry runs](docs/authoring/checking-and-formatting.md#format-source-files) |
| Render PDF or HTML         | `cac build`                   | [Building and settings](docs/authoring/building.md)                                      |
| Convert source formats     | `cac convert cv.md --to json` | [Conversion](docs/authoring/conversion.md)                                               |

See [FieldMark syntax](docs/authoring/fieldmark.md) for headings and optional fields,
and [example projects](docs/examples/README.md) for complete CVs and rendered results.

## Choose or create a theme

The `classic` theme is included. Follow [using themes](docs/authoring/themes.md)
to find, install, and select another design.

To create your own, use `cac theme init`, `cac theme test`, and `cac theme pack`.
The [theme contribution guide](themes/README.md) explains the workflow;
the [shared theme contract](docs/development/shared-theme-contract.md) defines the
rendering requirements.

## Development

[![Rust 1.92+](https://img.shields.io/badge/Rust-1.92%2B-orange?logo=rust)](Cargo.toml)

See the [development guide](docs/development/README.md) for setup, architecture, testing, pull requests, and releases.

## Roadmap

- [ ] More themes
- [ ] Rebuild automatically when files change
- [ ] Create job-specific CV versions with tags
- [ ] Fit a CV to a page limit
- [ ] Check the rendered PDF for missing content
- [ ] WebAssembly support
- [ ] Stunning local Web UI with Tauri
- [ ] VS Code extension

## License

Code a CV is available under the [MIT License](LICENSE).
