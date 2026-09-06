# Building and settings

[Usage guide](README.md)

## Render PDF or HTML

Run commands from the CV project directory:

```sh
cac build
cac build cv.md --format pdf,html
cac build cv.yaml --format pdf
cac build cv.md --output previews
```

`cac build` selects the `root` in `settings.json`, falling back to `cv.md`.
An explicit filename selects that source. Output defaults to `offering/`;
existing artifacts with the same names are replaced. HTML uses the HTML renderer;
Typst theme layouts and PDF typography settings apply to PDF output.

## Project settings

The starter creates `settings.json`. For example:

```json
{
  "$schema": ".cac/settings.schema.json",
  "root": "cv.md",
  "naming": "Ada_Lovelace_Engineering",
  "theme": "classic",
  "page": { "paper": "a4", "margin": "16mm" },
  "typography": { "font_size": "9.5pt" }
}
```

`root` selects the input; `naming` selects the artifact basename. This example
produces `offering/Ada_Lovelace_Engineering.pdf`. Without `naming`, the source
filename supplies the basename.

`cac init` and `cac build` refresh `.cac/settings.schema.json` for editor
completion and validation. Use that schema to explore supported settings.
Pass `--settings path/to/settings.json` to select a settings file explicitly.
See [using themes](themes.md) for theme selection and the
[customized-layout example](../examples/customized-layout/README.md) for layout settings.
