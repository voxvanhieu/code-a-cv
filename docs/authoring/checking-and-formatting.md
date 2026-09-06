# Checking and formatting

[Usage guide](README.md)

## Check content

```sh
cac check cv.md
cac check cv.md --explain
cac check cv.md --strict
cac check cv.yaml
```

`--explain` prints the normalized CV as JSON before content diagnostics, showing
how headings and fields were interpreted. Rich text appears as canonical Markdown
inside JSON strings.

Ordinary content suggestions do not stop `cac build`. `--strict` makes any
reported diagnostic fail the check, which is useful when enforcing editorial
rules. Invalid syntax, conflicting fields, and invalid dates must be corrected.
The check command defaults to `cv.md`; specify the filename for other sources.

## Format source files

```sh
cac fmt
cac fmt cv.md
cac fmt cv.json
cac fmt cv.yaml --dry-run
cac fmt cv.toml
cat cv.json | cac fmt - --input-format json
```

With no filename, `cac fmt` uses `settings.json`'s `root`, falling back to `cv.md`.
The filename extension selects the format; `--input-format` overrides it. Standard
input defaults to Markdown. JSON Resume can use `--input-format jsonresume`.

The default replaces the source file after verifying that parsing preserves its
CV data. Invalid input or a failed preservation check leaves the file unchanged.

| Command | Output | Writes the file? | Exit status |
|---|---|---|---|
| `cac fmt cv.md` | `FORMATTED cv.md` | Yes | Zero on success |
| `cac fmt cv.md --dry-run` (already formatted) | `PASS` | No | Zero |
| `cac fmt cv.md --dry-run` (needs formatting) | Error explaining that formatting is needed | No | Nonzero |
| `cac fmt -` | Formatted source on stdout | No | Zero on success |

`--dry-run` reports status, not a diff or a formatted preview. Parse errors also
exit nonzero. With stdin, `--dry-run` checks the supplied text instead of printing
formatted source. Without `--dry-run`, stdin produces formatted stdout because
there is no source file to replace.

### Check formatting without writing

```sh
cac fmt --dry-run
```

Use this in automated checks to fail when the configured project source needs
formatting. To apply the changes locally, run `cac fmt`, then repeat the dry run.
This checks source formatting; `cac check` separately checks CV content.

### Migrating older commands

Replace `cac fmt FILE --check` with `cac fmt FILE --dry-run`. Replace
`cac fmt FILE --write` with `cac fmt FILE`; both old options have been removed.
Previously, plain `cac fmt FILE` printed the formatted source. It now writes the
file. To inspect formatted output without writing, use stdin:

```sh
cat cv.md | cac fmt -
cat cv.json | cac fmt - --input-format json
```


Formatting preserves the source format. Bare Markdown stays bare Markdown;
FieldMark keeps its existing fields. Markdown formatting tidies trailing whitespace
and final blank lines while retaining hard breaks, comments, metadata boundaries,
headings, links, and date notation. It does not insert `Kind:` fields or directives.
JSON is pretty-printed; YAML and TOML are serialized in their own syntax, retaining
supplied fields rather than adding normalized CV defaults. YAML/TOML comments and
original quoting may not survive serialization.

Use [conversion](conversion.md) when you want another representation. Formatting
never rewrites achievements or invents facts.

## Editor snippets

Copy [cac.code-snippets](../editor/cac.code-snippets) into your project's
`.vscode/` directory, or add its entries to your editor's user snippets.
In VS Code Markdown files, use `cac-profile`, `cac-section`, `cac-experience`,
or `cac-project` to insert a template. Fill the placeholders and remove optional
fields you do not need.
