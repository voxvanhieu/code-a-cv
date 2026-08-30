# Development

`cac` is a Cargo workspace with five crates:

* `cac-core` contains the CV model, dates, and inline rich-text AST
* `cac-io` parses and serializes Markdown and structured formats
* `cac-render` owns the Typst API boundary and HTML renderer
* `cac-check` contains read-only content rules
* `cac` owns filesystem access and command dispatch

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
$ cargo run -p cac -- init
$ cargo run -p cac -- build
```

## Releases

`cargo-dist` builds the `cac` binary on native Linux, macOS, and Windows runners. It publishes source and binary archives, SHA-256 checksums, and shell and PowerShell installers.

Before Homebrew Core acceptance, run the `Publish Homebrew tap formula` workflow after each stable release. It validates the source-building formula on Linux and macOS, then opens a pull request in `voxvanhieu/homebrew-tap`. The workflow requires a `HOMEBREW_TAP_TOKEN` Actions secret scoped to that repository. See `HOMEBREW_TAP.md` for setup and migration instructions.

After the project meets Homebrew's acceptance requirements, run the `Prepare initial Homebrew Core formula` workflow. It requires a `HOMEBREW_CORE_TOKEN` Actions secret that can push to `voxvanhieu/homebrew-core`. Review the generated branch and open the pull request to `Homebrew/homebrew-core` manually. Coordinate its merge with the migration in the project tap. After acceptance, use Homebrew's standard `brew bump-formula-pr` process for version updates.

Prepare a release by updating the workspace version and `CHANGELOG.md`, then validate the release plan:

```console
$ dist plan
```

Commit the release changes, create a `vX.Y.Z` tag that matches the workspace version, and push the tag. The generated `.github/workflows/release.yml` workflow publishes the release. Run `dist generate` after changing the dist configuration. Run `dist init` when upgrading `cargo-dist`.
