#import "/.cac/base.typ" as base
#import "/.cac/theme.typ": theme
#let style = json("/.cac/style.json")
#let settings-style = json("/.cac/settings-style.json")
#base.render(json("/.cac/cv.json"), theme, style, settings-style)
