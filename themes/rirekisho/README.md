# rirekisho

A JIS-style 履歴書 form built on the form family: fixed grid, 和暦 era
dates computed against the `document_date` setting, a photo cell, and
furigana from the profile's open `Furigana` field.

Select it with `"theme": "rirekisho"` and set `"document_date"` in
settings.json for deterministic era dates (without one, the date line is
omitted). The form requires a photo and the `Furigana` profile field, and
accepts Education, Certifications, and Languages sections — anything else
fails with the section named.

The bundled font is a JIS X 0208 subset of Noto Sans JP, licensed under
the SIL Open Font License 1.1 (`fonts/OFL.txt`; the theme as a whole is
`MIT AND OFL-1.1`). Only the Regular weight ships, so headings render in
the regular weight — drop a Bold face into `fonts/` to strengthen them.
Install with `cac theme install rirekisho`; the registry gates the theme
on cac 0.4.0 or newer.
