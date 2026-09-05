#import "/.cac/base.typ" as base
#import "/.cac/theme.typ": theme
#let settings = json("/.cac/settings.json")
#base.render(json("/.cac/cv.json"), theme, settings)
