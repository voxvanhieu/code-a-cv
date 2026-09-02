# Local themes

List the embedded and already installed themes:

```sh
cac theme list
```

Search all downloadable themes, or filter them with a word such as `blue`:

```sh
cac theme search
cac theme search blue
```

Inspect a theme before downloading it:

```sh
cac theme info classic-blue
```

Download a theme into this project:

```sh
cac theme install classic-blue --local
```

The `--local` option installs it under `.cac/themes/`. Select the downloaded
theme by setting `"theme": "classic-blue"` in the project's `settings.json`.
