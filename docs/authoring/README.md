# Usage guide

Create, write, check, and render a CV with Code a CV. Install `cac` using the
[repository instructions](../../README.md), then start a project:

```sh
mkdir my-cv
cd my-cv
cac init
```

This creates `cv.md`, `settings.json`, and an editor schema. Edit the starter with
your own details using [FieldMark](fieldmark.md), our Markdown style with readable
fields such as `Kind:`, `Organization:`, and `Period:`. Optional fields can be removed.

```sh
cac check cv.md
cac build
```

The default PDF is `offering/cv.pdf`. Content suggestions from `cac check` are
advisory; invalid source data must be corrected before building.

Formatting is optional: `cac fmt` formats the project source in place, preserving
its format. Use `cac fmt --dry-run` to check formatting without changing the file.
See [checking and formatting](checking-and-formatting.md) for output and exit statuses.

## Contents

| Feature | Guide |
|---|---|
| Create a project or import an existing CV | [Starting a CV](getting-started.md) |
| Write headings, fields, dates, prose, and theme-defined sections | [FieldMark syntax](fieldmark.md) |
| Inspect parsed data and check content | [Checking and formatting](checking-and-formatting.md#check-content) |
| Format a source file or enforce its formatting | [Formatting source files](checking-and-formatting.md#format-source-files) |
| Export PDF/HTML and configure output | [Building and settings](building.md) |
| Convert Markdown, JSON, YAML, TOML, and JSON Resume | [Format conversion](conversion.md) |
| Select an installed or downloaded theme | [Using themes](themes.md) |
| Insert profile, section, and entry templates | [Editor snippets](checking-and-formatting.md#editor-snippets) |
| Explore complete source files and rendered results | [Example projects](../examples/README.md) |

Run `cac --help` to list commands or `cac build --help` for a command's options.
For theme development, see the [theme contribution guide](../../themes/README.md)
and [shared theme contract](../development/shared-theme-contract.md).
