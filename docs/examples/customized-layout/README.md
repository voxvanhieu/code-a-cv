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
