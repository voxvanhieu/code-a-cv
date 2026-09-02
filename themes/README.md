# Contributing themes

The `themes/` directory is the official downloadable theme registry. The default `classic` theme remains embedded in every `cac` installation and is maintained under `crates/cac-render/src/typst/themes/`.

## Add a theme

1. Create `themes/<theme-name>/theme.typ`
2. Create `themes/<theme-name>/preview.jpg` with a representative rendered CV
3. Create `themes/<theme-name>/README.md` using the manifest information and display the preview directly below the description
4. Create `themes/<theme-name>/theme.json`
5. Add the theme name and description to `themes/index.json`
6. Run `shasum -a 256` on every downloadable file and record each digest in `theme.json`
7. Run the repository test and lint commands
8. Open a pull request with the theme, manifest, registry entry, README, preview, and a short description of the design

Theme names use lowercase ASCII letters, numbers, hyphens, and underscores. Every theme must use Theme API 1, include an explicit license, provide a `README.md` and preview image, export `theme` from `theme.typ`, and inherit from the bundled base:

```typst
#import "/.cac/base.typ" as base

#let theme = base.extend(
  tokens: (...),
  styles: (...),
  page: (...),
  components: (...),
)
```

The manifest format is:

```json
{
  "name": "example",
  "description": "A short searchable description",
  "author": "Contributor name",
  "license": "MIT",
  "theme_api": 1,
  "preview": "preview.jpg",
  "files": [
    {
      "path": "theme.typ",
      "sha256": "64 lowercase hexadecimal characters"
    },
    {
      "path": "README.md",
      "sha256": "64 lowercase hexadecimal characters"
    },
    {
      "path": "preview.jpg",
      "sha256": "64 lowercase hexadecimal characters"
    }
  ]
}
```

`preview` names the image shown in the theme README. Use a repository-relative path inside the theme directory and include that image and `README.md` in `files`. The README must show the manifest's name, description, author, license, Theme API version, and entrypoint without duplicating file checksums.

Files may be placed under the theme directory, including `assets/` and `fonts/`. Absolute paths and parent-directory traversal are rejected. `cac` verifies every checksum before writing an installed theme.

## Test a registry checkout

Point `cac` at a local registry while developing:

```console
$ CAC_THEME_REGISTRY="file:///path/to/code-a-cv/themes" cac theme search
$ CAC_THEME_REGISTRY="file:///path/to/code-a-cv/themes" cac theme install example --local
$ cac build
```

After a contribution is merged, users install it from the default GitHub registry without setting the environment variable.
