# YAML to Markdown conversion

This project demonstrates converting a structured YAML CV into editable
Markdown. The YAML document remains the project root in `settings.json`, while
`Ada_Lovelace.md` is the generated conversion output.

Convert the source:

```sh
cac convert cv.yaml --to markdown -o Ada_Lovelace.md
```

Build the converted Markdown document:

```sh
cac build Ada_Lovelace.md
```

Run `cac build` without a file to build the configured YAML root instead.
