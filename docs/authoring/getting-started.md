# Starting a CV

[Usage guide](README.md)

Run `cac init` in a new project directory to create a Markdown starter and
`settings.json`. Replace the fictional Ada Lovelace content with your own facts.
See [FieldMark](fieldmark.md) for the writing rules.

## Structured sources

Choose one of these commands in a new directory:

```sh
cac init --format json
cac init --format yaml
cac init --format toml
```

Each creates a starter in the selected format and configures it as the project
root. Use named fields in structured sources; rich-text values support the same
inline formatting as FieldMark.

## Import JSON Resume

```sh
cac init --from ../resume.json
```

This imports the supported JSON Resume subset into a new project. Supported
sections are basics, work, education, and skills. Unsupported fields produce an
error rather than disappearing. See [conversion](conversion.md) for limitations.

## Next steps

Run `cac check` with the source filename, then `cac build` from your project.
Optional formatting uses `cac fmt` to write the project source or
`cac fmt --dry-run` to check it without writing. See
[checking and formatting](checking-and-formatting.md) and [building](building.md).
