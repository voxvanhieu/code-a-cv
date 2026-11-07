# Job-specific CV variants

One source file, one CV per posting. Sections and entries carry `<!-- tags: -->
` comments; `cac build --tags` keeps what a posting cares about, drops the
rest, and names the artifact after the variant.

- `cac build --tags backend` writes `offering/Ada_Lovelace_JobSpecific-backend.pdf`
  with the Platform section (and its leadership-tagged role) but no Web section.
- `cac build --tags frontend` writes the Web-only variant.
- `cac build --tags backend,leadership` keeps both tag sets.
- A tag that matches nothing fails with a diagnostic instead of writing an
  empty CV.

The committed PDFs in `offering/` were produced with the embedded `classic`
theme from this directory:

```sh
cd job-specific-cv
cac build --tags backend
cac build --tags frontend
cac build --tags backend,leadership
```

Profile data (name, contacts, summary) always stays; sections and entries are
what the tags filter. Untagged sections drop whenever `--tags` is set, so a
variant is exactly what you asked for and nothing else.
