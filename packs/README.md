# Field packs

A pack declares the open-field keys a layout or locale expects: labels,
types, and which are required. Load one with `cac check`:

```sh
cac check cv.md --pack packs/jp-rirekisho/fields.toml
```

Declared keys validate by type (a mistyped value is an error) and required
keys must be present where they apply; undeclared keys in the CV stay open
fields with an advisory. Pack files live beside the themes that read them;
`themes/rirekisho` documents its pack in its own README.
