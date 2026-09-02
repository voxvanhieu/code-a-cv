# Examples

Each directory is a self-contained Code a CV project. Change into a project
directory before running its commands so `cac` can find the project's
`settings.json` and write its output to that project's `offering/` directory.
Running `cac build` also creates or refreshes `.cac/settings.schema.json`, which
is referenced by each implemented project's `settings.json`.

PDF results are committed in each implemented example's `offering/` directory
so they can be inspected without building the project locally.

| Project | PDF result | Purpose | Commands |
|---|---|---|---|
| [`basic-markdown`](basic-markdown/) | [`Ada_Lovelace_BasicMarkdown.pdf`](basic-markdown/offering/Ada_Lovelace_BasicMarkdown.pdf) | The recommended starting point: author a CV in Markdown and render it with the embedded `classic` theme. | `cd basic-markdown`<br>`cac check`<br>`cac build` |
| [`structured-formats`](structured-formats/) | [`Ada_Lovelace_StructuredFormats.pdf`](structured-formats/offering/Ada_Lovelace_StructuredFormats.pdf) | Keep the same complete CV in YAML, JSON, and TOML. YAML is the configured root; the other sources can be selected explicitly. | `cd structured-formats`<br>`cac build`<br>`cac build cv.json`<br>`cac build cv.toml` |
| [`json-resume-import`](json-resume-import/) | [`Ada_Lovelace_JsonResumeImport.pdf`](json-resume-import/offering/Ada_Lovelace_JsonResumeImport.pdf) | Import an existing JSON Resume document into a regular Markdown-based `cac` project. | `cd json-resume-import`<br>`cac build`<br>`cac build resume.json --input-format jsonresume` |
| [`format-conversion`](format-conversion/) | [`Ada_Lovelace_FormatConversion.pdf`](format-conversion/offering/Ada_Lovelace_FormatConversion.pdf) | Convert a YAML CV to Markdown, then build the converted document. | `cd format-conversion`<br>`cac convert cv.yaml --to markdown -o Ada_Lovelace.md`<br>`cac build` |
| [`customized-layout`](customized-layout/) | [`Ada_Lovelace_CustomizedLayout.pdf`](customized-layout/offering/Ada_Lovelace_CustomizedLayout.pdf) | Render each section with a table suited to its content using a project-local theme. | `cd customized-layout`<br>`cac build` |
| [`local-theme`](local-theme/) |  | Find and download themes into a project's `.cac/themes` directory. | `cd local-theme`<br>`cac theme list`<br>`cac theme search`<br>`cac theme install THEME --local` |
| [`content-checking`](content-checking/) |  | Demonstrate normal and strict content diagnostics. | To be added |
| [`job-specific-cv`](job-specific-cv/) |  | Maintain a CV tailored to a particular role. | To be added |
| [`complete-cv`](complete-cv/) | [`Ada_Lovelace_CompleteCV.pdf`](complete-cv/offering/Ada_Lovelace_CompleteCV.pdf) | Exercise a long, complex Markdown CV with every supported section and entry shape. | `cd complete-cv`<br>`cac check`<br>`cac build --format pdf,html` |

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

The `json-resume-import/Ada_Lovelace.md` file is the imported result committed for easy
inspection. To repeat the import without overwriting the example, run this in
an empty directory:

```sh
cac init --from ../path/to/resume.json
```
