# Theme-defined sections

This fictional complete CV extends the Ada Lovelace starter with certifications,
consulting, and community recognition. It includes contacts, a summary, work
experience, education, projects, publications, skills, advisory work, and interests.

Build `cv.md` with any theme. The section categories `certifications` and
`consulting` are preserved without requiring a compiler rebuild. Themes can
select layouts using `section.kind` or `section.id`; unrecognized categories
use the normal section layout.

The consulting entry explicitly uses the experience data structure. Section
categories select presentation; entry kinds define the available fields.

See the [FieldMark authoring guide](../../authoring/fieldmark.md#theme-defined-sections) and
[theme contract](../../development/shared-theme-contract.md#theme-defined-section-categories).
