# Examples

Each directory is a self-contained Code a CV project. Change into a project
directory before running its commands so `cac` can find the project's
`settings.json` and write its output to that project's `offering/` directory.
Running `cac build` also creates or refreshes `.cac/settings.schema.json`, which
is referenced by each implemented project's `settings.json`.

The generated `offering/` directories, including their PDFs and HTML files, are
ignored by Git. You can build and inspect them locally without changing the
working tree.

| Project | Status | Purpose | Commands |
|---|---|---|---|
| [`basic-markdown`](basic-markdown/) | Implemented | The recommended starting point: author a CV in Markdown and render it with the embedded `classic` theme. | `cd basic-markdown`<br>`cac check`<br>`cac build` |
| [`structured-formats`](structured-formats/) | Implemented | Keep the same complete CV in YAML, JSON, and TOML. YAML is the configured root; the other sources can be selected explicitly. | `cd structured-formats`<br>`cac build`<br>`cac build cv.json`<br>`cac build cv.toml` |
| [`json-resume-import`](json-resume-import/) | Implemented | Import an existing JSON Resume document into a regular Markdown-based `cac` project. | `cd json-resume-import`<br>`cac build`<br>`cac build resume.json --input-format jsonresume` |
| [`format-conversion`](format-conversion/) | Implemented | Convert a YAML CV to Markdown, then build the converted document. | `cd format-conversion`<br>`cac convert cv.yaml --to markdown -o cv.md`<br>`cac build cv.md` |
| [`customized-layout`](customized-layout/) | Implemented | Render each section with a table suited to its content using a project-local theme. | `cd customized-layout`<br>`cac build` |
| [`local-theme`](local-theme/) | Documentation | Find and download themes into a project's `.cac/themes` directory. | `cd local-theme`<br>`cac theme list`<br>`cac theme search`<br>`cac theme install THEME --local` |
| [`content-checking`](content-checking/) | Planned | Demonstrate normal and strict content diagnostics. | To be added |
| [`job-specific-cv`](job-specific-cv/) | Planned | Maintain a CV tailored to a particular role. | To be added |
| [`complete-cv`](complete-cv/) | Implemented | Exercise a long, complex Markdown CV with every supported section and entry shape. | `cd complete-cv`<br>`cac check`<br>`cac build --format pdf,html` |

## Build every example

From the repository root, use the local development binary:

```sh
(cd docs/examples/basic-markdown && cargo run -p cac -- build)
(cd docs/examples/structured-formats && cargo run -p cac -- build)
(cd docs/examples/json-resume-import && cargo run -p cac -- build)
```

With an installed `cac` binary, replace `cargo run -p cac --` with `cac`.

The three files in `structured-formats` intentionally describe the same CV.
This lets codec tests verify that all native structured formats produce the
same document while keeping them together as one example use case.

The `json-resume-import/cv.md` file is the imported result committed for easy
inspection. To repeat the import without overwriting the example, run this in
an empty directory:

```sh
cac init --from ../path/to/resume.json
```
