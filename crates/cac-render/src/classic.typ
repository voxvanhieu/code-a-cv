#let cv = json("cv.json")
#let rich(nodes) = {
  for node in nodes {
    if node.kind == "text" { node.text }
    else if node.kind == "code" { raw(node.text) }
    else if node.kind == "emph" { emph(rich(node.body)) }
    else if node.kind == "strong" { strong(rich(node.body)) }
    else if node.kind == "link" { link(node.href, rich(node.body)) }
  }
}
#set document(title: cv.profile.name + " CV", author: cv.profile.name)
#set page(paper: "a4", margin: (x: 1.8cm, y: 1.5cm))
#set text(font: "Libertinus Serif", size: 10pt)
#set par(justify: true, leading: 0.55em)
#show heading.where(level: 2): it => block(above: 1.0em, below: 0.45em)[
  #set text(size: 12pt, weight: "bold")
  #it.body
  #line(length: 100%, stroke: 0.5pt)
]

#align(center)[
  #text(size: 20pt, weight: "bold")[#cv.profile.name]
  #v(0.25em)
  #cv.profile.contacts.join("  ·  ")
]

#if cv.profile.summary != none [
  #v(0.8em)
  #rich(cv.profile.summary)
]

#for section in cv.sections [
  == #section.title
  #for entry in section.entries [
    #grid(
      columns: (1fr, auto),
      gutter: 0.5em,
      [#strong(rich(entry.primary))#if entry.secondary != none [, #rich(entry.secondary)]],
      [#if entry.period != none [#entry.period]],
    )
    #if entry.highlights.len() > 0 [
      #set list(indent: 1em, body-indent: 0.45em, spacing: 0.15em)
      #for item in entry.highlights [
        - #rich(item)
      ]
    ]
    #v(0.35em)
  ]
]
