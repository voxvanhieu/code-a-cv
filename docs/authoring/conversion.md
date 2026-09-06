# Format conversion

[Usage guide](README.md)

```sh
cac convert cv.md --to json -o cv.json
cac convert cv.yaml --to markdown -o cv.md
cac convert cv.json --to toml -o cv.toml
cac convert resume.json --input-format jsonresume --to markdown -o imported.md
```

Supported format names are `markdown`, `json`, `yaml`, `toml`, and `jsonresume`.
Without `-o`, conversion writes to standard output. With `-o`, it replaces the
specified file after parsing and any required preservation checks succeed.
Use `--input-format jsonresume` to distinguish JSON Resume from native CV JSON.

Markdown, native JSON, YAML, and TOML share the core CV model. Native structured
formats preserve theme-defined section kinds and the complete model. Markdown
export verifies that reparsing preserves the supplied document and fails if it
cannot represent the data without loss.

JSON Resume supports a smaller subset: basics, work, education, and skills.
Import rejects unsupported fields. Export fails if it would lose content,
formatting, custom sections, order, or metadata. An omitted end date stays unknown;
it does not implicitly mean an ongoing role. Prefer a native format when you need
to preserve the complete CV.

See the [conversion example](../examples/format-conversion/README.md) and
[structured source examples](../examples/structured-formats/).
