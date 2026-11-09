# Style keys

The style sheet every renderer reads: defaults, then the theme's `[style]`, then the settings' `[style]`. Generated from `crates/cac-render/schema/style.schema.json` — do not edit by hand;

```sh
CAC_WRITE_STYLE_KEYS=1 cargo test -p cac-render --lib
```

regenerates it.

## `colors`

| Key | Type | Description |
| --- | --- | --- |
| `accent` | ColorSpec or null |  |
| `muted` | ColorSpec or null |  |
| `rule` | ColorSpec or null | Color of section heading rules and the header rule; defaults to `accent`. |
| `text` | ColorSpec or null |  |

## `entry`

Presentation of dated entries; the layout families read these.

| Key | Type | Description |
| --- | --- | --- |
| `date_position` | DatePosition or null |  |
| `keep_together` | value | Keep each entry on one page when it fits. The legacy `pagination.allow_entry_page_break` is the inverse of this key. |
| `subtitle_style` | SubtitleStyle or null |  |

## `fonts`

| Key | Type | Description |
| --- | --- | --- |
| `body` | value |  |
| `heading` | value |  |
| `mono` | value |  |

## `header`

| Key | Type | Description |
| --- | --- | --- |
| `align` | HeaderAlign or null |  |
| `rule` | RuleStyle or null |  |

## `headings`

| Key | Type | Description |
| --- | --- | --- |
| `entry` | HeadingSlot or null | Entry titles (heading level 3 today). |
| `name` | HeadingSlot or null | The profile name heading (heading level 1 today). |
| `section` | SectionHeadingSlot or null | Section titles (heading level 2 today). |

## `links`

| Key | Type | Description |
| --- | --- | --- |
| `underline` | value |  |

## `lists`

| Key | Type | Description |
| --- | --- | --- |
| `bullet` | value |  |

## `page`

| Key | Type | Description |
| --- | --- | --- |
| `margin` | MarginSpecs or null | Per-side margins; the legacy single `page.margin` applies to all sides. |
| `paper` | Paper or null |  |

## `spacing`

| Key | Type | Description |
| --- | --- | --- |
| `entry` | LengthSpec or null | Space after each entry. |
| `heading_gap` | LengthSpec or null | Space between a section heading and its entries (the 0.3 `spacing.section_heading`). |
| `item` | LengthSpec or null | Space between list items; the default follows the text leading. |
| `section` | LengthSpec or null | Space above each section block. |

## `text`

| Key | Type | Description |
| --- | --- | --- |
| `align` | TextAlign or null |  |
| `leading` | LengthSpec or null |  |
| `paragraph_gap` | LengthSpec or null |  |
| `size` | LengthSpec or null |  |
