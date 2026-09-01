#import "/.cac/base.typ" as base
#import "/.cac/theme.typ": theme

#let cv = json("/.cac/cv.json")
#let settings = json("/.cac/settings.json")

#if "api_version" not in theme {
  panic("theme does not declare an API version; export `theme` with `base.extend`")
}

#if theme.api_version != base.api_version {
  panic("unsupported theme API version " + str(theme.api_version) + "; cac supports version " + str(base.api_version))
}

#base.render(cv, theme, settings)
