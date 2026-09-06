# Using themes

[Usage guide](README.md)

The embedded `classic` theme works immediately. Discover and install another:

```sh
cac theme list
cac theme search blue
cac theme install classic-blue --local
```

`--local` installs into this project's `.cac/themes/`. Omit it to install for
all your projects. Set the theme in the existing `settings.json`:

```json
{
  "theme": "classic-blue"
}
```

Keep your other settings, then run `cac build` to render the PDF.

FieldMark's section `Kind:` and `Id:` values are passed to themes. A theme can
recognize categories such as `certifications` without rebuilding the CLI;
unrecognized categories use the normal layout. Section categories do not add
new structured entry fields. See [theme-defined sections](fieldmark.md#theme-defined-sections).

To create a theme, follow the [theme contribution guide](../../themes/README.md).
Its implementation must follow the [shared theme contract](../development/shared-theme-contract.md).
