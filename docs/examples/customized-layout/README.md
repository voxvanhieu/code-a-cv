# Customized table layout

This project uses the local `table-layout` theme to give each kind of CV
content a suitable table structure:

- Experience and education use a fixed date column beside the main details.
- Projects and publications use a flexible details column and compact date column.
- Skill groups use the group name as a label beside its skills.
- Custom and free-text entries use a simple fallback layout.

Build the CV from this directory:

```sh
cac build
```

The theme is committed at `.cac/themes/table-layout/theme.typ`, so the layout
does not depend on a theme installed in the user's home directory.

This is a reusable theme with English column labels. Every row must have a
compatible type before a table is selected. Mixed sections fall back as a whole,
empty sections retain their headings, and sections without dates collapse the
date column. Locations, linked URLs, and all generic fields remain visible.
The bundled helpers apply common settings and permit oversized rows to continue.
See the [shared-theme contract](../../development/shared-theme-contract.md).
