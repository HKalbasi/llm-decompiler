#import "@preview/in-dexter:0.7.2": *
#import "notes.typ"

#set text(
  font: "XB Niloofar",
  lang: "fa",
  size: 13.5pt,
)

#set par(
  first-line-indent: (amount: 1.5em, all: true),
  leading: 15pt,
  justify: true,
)

#show heading: set block(above: 36pt, below: 32pt)
#show heading.where(level: 1): set text(size: 24pt)

#include "heading.typ"
#pagebreak()

#include "summary.typ"
#pagebreak()

#heading(outlined: false)[فهرست مطالب]

#outline(title: none)

#pagebreak()

#show heading.where(level: 1): it => [
  #set text()
  #linebreak() #linebreak() #linebreak() فصل #counter(heading).display(it.numbering) #linebreak() #linebreak() #it.body
]

#set heading(numbering: (..n) => numbering("۱-۱", ..n.pos().rev()))

#set page(numbering: "۱")

#counter(page).update(1)

#include "chapters/intro.typ"
#pagebreak()
#include "chapters/prev.typ"
#pagebreak()
#include "chapters/prevent_halucination.typ"
#pagebreak()
#include "chapters/works.typ"
#pagebreak()
#include "chapters/layer.typ"
#pagebreak()
#include "chapters/ir_passes.typ"
#pagebreak()
#include "chapters/ir_to_c.typ"
#pagebreak()
#include "chapters/evaluation.typ"
#pagebreak()
#include "chapters/future.typ"
#pagebreak()


= مراجع

#text(dir: ltr, font: "Times New Roman", size: 9pt)[

  #bibliography("ref.bib", title: none)

]
