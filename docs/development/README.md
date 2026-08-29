# Development

`cac` is a Cargo workspace with five crates:

* `cac-core` contains the CV model, dates, and inline rich-text AST
* `cac-io` parses and serializes Markdown and structured formats
* `cac-render` owns the Typst API boundary and HTML renderer
* `cac-check` contains read-only content rules
* `cac-cli` owns filesystem access and command dispatch

The renderer passes a normalized CV view to Typst as the virtual `cv.json` file. User text is never interpolated into Typst source. The HTML renderer walks the same `RichText` AST and escapes every text and attribute node.

Typst dependencies are pinned to `0.15.1` because its Rust API is not stable. Rust 1.92 or newer is required.

Run the required checks with:

```console
$ cargo fmt --all --check
$ cargo test --workspace
$ cargo clippy --all-features --all-targets -- -D warnings
```

Build a local sample with:

```console
$ cargo run -p cac-cli -- init
$ cargo run -p cac-cli -- build
```

## Releases

`cargo-dist` builds the `cac` binary on native Linux, macOS, and Windows runners. It publishes archives, SHA-256 checksums, and shell and PowerShell installers to GitHub Releases.

Prepare a release by updating the workspace version and `CHANGELOG.md`, then validate the release plan:

```console
$ dist plan
```

Commit the release changes, create a `vX.Y.Z` tag that matches the workspace version, and push the tag. The generated `.github/workflows/release.yml` workflow publishes the release. Run `dist generate` after changing the dist configuration. Run `dist init` when upgrading `cargo-dist`.
