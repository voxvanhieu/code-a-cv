# Contributing to Code a CV

Welcome! Help make `cac` a better way to turn CV data into PDF and HTML. Documentation fixes, bug reports, tests, themes, and code improvements all count. You do not need to know the whole codebase to get started.

## 1. Pick a small change

Browse [open issues](https://github.com/voxvanhieu/code-a-cv/issues) and [pull requests](https://github.com/voxvanhieu/code-a-cv/pulls) to see what is already underway. Good first contributions include clarifying an example, reproducing a bug, or adding a test for an edge case. Comment on an issue if you want to work on it or need help finding the right file.

For a small fix, feel free to open a PR directly. For a new feature or a large change, open an issue first so we can agree on the approach before you invest time. Keep the default `cac init` → `cac build` workflow simple, local, and self-contained, with configuration optional. Discuss additions such as LaTeX output, a theme registry, an LSP, a web playground, or AI writing features first.

Reporting a bug? Include your OS, `cac --version`, the command you ran, expected and actual results, and a small CV that reproduces the problem. Use fictional personal details in shared CVs. For suspected security vulnerabilities, contact the maintainers privately instead of opening a public issue.

### Label issues and pull requests

Apply the relevant labels when opening or triaging an issue or PR. Use the **Labels** menu in GitHub's sidebar; if you cannot edit labels, suggest them in the description so a maintainer can apply them. Label a related issue and PR separately, choosing labels that describe each item.

Use the repository's [available labels](https://github.com/voxvanhieu/code-a-cv/labels):

| Label | Apply to | When to use it |
| --- | --- | --- |
| `bug` | Issues and PRs | Report broken behavior or submit a fix, such as incorrect HTML escaping. |
| `documentation` | Issues and PRs | Request or make improvements to guides, examples, or other documentation. |
| `enhancement` | Issues and PRs | Propose or implement new functionality or an improvement, including a new theme. |
| `accessibility` | Issues and PRs | Identify or remove a barrier affecting people with disabilities. Combine with `bug` or `enhancement` when appropriate. |
| `dependencies` | PRs | Update dependency files, such as Cargo manifests and lockfiles. |
| `github_actions` | PRs | Change GitHub Actions workflows. Combine with `dependencies` for action-version updates. |
| `question` | Issues and PRs | Ask a usage question or flag that further information is needed to proceed. |
| `good first issue` | Issues | Maintainers identify a clearly scoped task suitable for newcomers, with enough guidance to start. |
| `help wanted` | Issues and PRs | Maintainers invite extra help with implementation, reproduction, testing, or review. |
| `duplicate` | Issues and PRs | Maintainers identify an existing item covering the same work; link to that item. |
| `invalid` | Issues and PRs | Maintainers determine that an item is not a valid report or request; explain why. |
| `wontfix` | Issues and PRs | Maintainers decide the work will not be pursued; record the reason. |

Choose all labels that apply, without forcing a match. For example, a guide correction uses `documentation`, while a request to fix inaccessible output may use both `bug` and `accessibility`. Branch prefixes and labels serve different purposes: a `chore/` branch updating an action can use `github_actions` and `dependencies`. Labels are not assigned automatically by branch names.

## 2. Get a local checkout

For documentation edits, you can work directly in GitHub's editor without installing Rust. For code or theme work, install Git and [Rust 1.92 or newer](https://rustup.rs/), then add the formatter and linter:

```console
$ rustup update stable
$ rustup component add rustfmt clippy --toolchain stable
```

Fork [code-a-cv](https://github.com/voxvanhieu/code-a-cv/fork) on GitHub. Replace `YOUR-USERNAME` below with your GitHub username:

```console
$ git clone https://github.com/YOUR-USERNAME/code-a-cv.git
$ cd code-a-cv
$ git remote add upstream https://github.com/voxvanhieu/code-a-cv.git
$ rustup override set stable
$ git fetch upstream
$ git switch --create docs/improve-quickstart upstream/main
$ cargo run -p cac -- --help
```

Choose a branch name for your own change using the rules below. The first build downloads and compiles dependencies, so allow a few minutes. You do not need a separate Typst or LaTeX installation.

### Branch names

Start each contribution from an up-to-date `main` and open its PR against `main`. Use `<prefix>/<short-description>` with a lowercase, hyphen-separated description. Keep one purpose per branch.

| Prefix | Use for | Example |
| --- | --- | --- |
| `feat/` | New functionality | `feat/add-export-option` |
| `fix/` | Bug fixes | `fix/escape-html-links` |
| `docs/` | Documentation | `docs/improve-quickstart` |
| `test/` | Tests and fixtures | `test/unicode-dates` |
| `refactor/` | Code cleanup without behavior changes | `refactor/split-date-parser` |
| `chore/` | Dependencies, tooling, and CI | `chore/update-ci-cache` |
| `theme/` | Shared themes under `themes/` | `theme/add-compact-layout` |
| `release/` | Release preparation | `release/0.2.0` |

An issue number is optional, for example `fix/123-escape-html-links`. These prefixes are a contribution convention, not a CI-enforced naming check.

## 3. Make and try your change

Use this map to find a starting point:

| What you want to change | Where to look |
| --- | --- |
| Commands and CLI behavior | `crates/cac/` |
| CV data types, dates, and rich text | `crates/cac-core/` |
| Markdown, YAML, JSON, TOML, and JSON Resume | `crates/cac-io/` |
| HTML and PDF rendering | `crates/cac-render/` |
| Content diagnostics | `crates/cac-check/` |
| Guides and sample CVs | `docs/` and `docs/examples/` |
| Downloadable themes | `themes/` |

Run the affected crate's tests while you work, for example:

```console
$ cargo test -p cac-io --locked
```

For an end-to-end preview, run these commands from the repository root in a POSIX shell (such as Bash or Git Bash on Windows):

```console
$ cargo build -p cac --locked
$ mkdir -p target/contribution-preview
$ cd target/contribution-preview
$ ../debug/cac init
$ ../debug/cac build
$ cd ../..
```

Open `target/contribution-preview/offering/cv.pdf` to inspect the result. On Windows, the executable is `cac.exe`. The scratch directory keeps sample CVs, settings, and generated output out of your contribution.

Add an integration test in the affected crate's `tests/` directory for each behavior change. Include relevant Unicode, special-character, minimal-document, or round-trip cases. Update CLI help snapshots only when the help change is intentional. For changes affecting rendering or checking performance, include a measurement.

Keep rendering safe: escape rich text in each renderer, never interpolate CV strings into Typst source, and never give themes filesystem or shell access. Keep `cac check` read-only and never let `--fit` remove CV content. Preserve explicit Markdown section semantics when headings are translated or renamed.

**Contributing a shared theme?** Follow the [theme author contract](docs/development/shared-theme-contract.md), run `cac theme test --shared` in the theme project, and include a preview for visual review. A PR touching `themes/` must contain only files under `themes/`; CI rejects mixed changes. Put supporting code or documentation changes, including any root changelog update, in a separate PR.

See the [development guide](docs/development/README.md) for architecture, detailed testing, and release workflows. Edit release configuration in `dist-workspace.toml` and regenerate with `dist generate`; do not hand-edit the generated release workflow.

## 4. Check your work

For code or theme changes, run the same quality checks as CI:

```console
$ cargo fmt --all
$ cargo fmt --all --check
$ cargo test --workspace --all-features --locked
$ cargo clippy --workspace --all-features --all-targets --locked -- -D warnings
```

For prose-only edits, preview the Markdown and check links and examples. Rust checks are not needed unless you also change code or executable examples. If you cannot run a relevant check, say so in the PR and include any error output; you can still ask for help.

Update the relevant documentation when commands, formats, installation, or workflows change. Add user-visible changes under `Unreleased` in [CHANGELOG.md](CHANGELOG.md). Before committing, review `git diff` and `git status` so generated files and personal CV data stay out of the PR.

## 5. Open a pull request

Commit your change with a short imperative message, such as `Clarify the contributor quick start`, then push your branch to your fork:

```console
$ git push --set-upstream origin HEAD
```

Open a PR against `voxvanhieu/code-a-cv:main` and apply the [relevant labels](#label-issues-and-pull-requests). A useful description includes:

- What problem you solved and what users will see afterward.
- A related issue, if there is one, and any compatibility impact.
- The verification commands you ran and their results; screenshots or PDF previews for visual changes.

Draft PRs are welcome when you want feedback or are stuck. Point out the part you need help with. Keep discussion constructive, respond to review comments, and push follow-up commits to the same branch. Wait for review and the `CI complete` check before merging.

Thank you for helping improve `cac`, whether your contribution is a single sentence or a new feature.
