# Contributing themes

The `themes/` directory is the official downloadable theme registry. The default `classic` theme remains embedded in every `cac` installation and is maintained under `crates/cac-render/src/typst/themes/`.

## Add a theme

1. Create `themes/<theme-name>/theme.typ`
2. Create `themes/<theme-name>/preview.jpg` with a representative rendered CV
3. Create `themes/<theme-name>/README.md` using the manifest information and display the preview directly below the description
4. Create `themes/<theme-name>/theme.json`
5. Add the theme name and description to `themes/index.json`
6. Run `shasum -a 256` on every downloadable file and record each digest in `theme.json`
7. Run `cac theme test --shared` in the theme-development project, review the fixture PDFs, and run the repository test and lint commands
8. Open a pull request with the theme, manifest, registry entry, README, preview, and a short description of the design

Theme names use lowercase ASCII letters, numbers, hyphens, and underscores. New shared themes follow the [data and styling contract](../docs/development/shared-theme-contract.md), include an explicit license, provide a `README.md` and preview image, export `theme` from `theme.typ`, and inherit from the bundled base:

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
  "author_url": "https://example.com/contributor",
  "license": "MIT",
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

`author_url` is optional and, when provided, must be an HTTP or HTTPS URL. Link the author name to it in the theme README. `preview` names the image shown in the theme README. Use a repository-relative path inside the theme directory and include that image and `README.md` in `files`. Display it with an HTML `img` element that sets `width` and `height` to its intrinsic dimensions. The README must show the manifest's name, description, author, license, and entrypoint without duplicating file checksums.

Files may be placed under the theme directory, including `assets/` and `fonts/`. Absolute paths and parent-directory traversal are rejected. `cac` verifies every checksum before writing an installed theme.

## Test a registry checkout

Point `cac` at a local registry while developing:

```console
$ CAC_THEME_REGISTRY="file:///path/to/code-a-cv/themes" cac theme search
$ CAC_THEME_REGISTRY="file:///path/to/code-a-cv/themes" cac theme install example --local
$ cac build
```

After a contribution is merged, users install it from the default GitHub registry without setting the environment variable.

See the [shared theme contract](../docs/development/shared-theme-contract.md) for strict acceptance requirements, regression checks, and visual review before publishing.

## Shared-theme admission

Registry themes must follow the [shared-theme contract](../docs/development/shared-theme-contract.md): preserve every visible field and source order, handle mixed and optional content through generic fallback, and honor applicable common settings. Keep presentation labels in a theme-owned dictionary and document the language. Literal title comparisons must not select layouts. Installation location is independent of shared-theme suitability.

Theme projects start with the same rich Ada Lovelace sample as `cac init`. Edit the generated `cv.md` to choose the content shown in `preview.jpg`; shared acceptance fixtures are separate.

Start with `cac theme init`, complete the manifest description, then run `cac theme test --shared` before `cac theme pack`. The shared fixture PDFs stay under `offering/` and outside the package. Conformance depends on preserving the documented CV data and honoring styling settings, without a version declaration.

CI renders the corpus for every registry theme, including theme-only changes. Review the complete, mixed, long, and settings PDFs for clipping, glyphs, order, page breaks, and readable repeated headers. A passing finite corpus supplements source review.
