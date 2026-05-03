#import "@preview/in-dexter:0.7.2": *
#import "notes.typ"

#set text(
  font: "XB Niloofar",
  lang: "fa",
  size: 14pt,
)

#let to-persian(n) = numbering("۱", int(n))

// 2. Apply to all citations in the text
#show ref: it => {
  show regex("[0-9]+"): n => to-persian(n.text)
  it
}

#show figure.where(
  kind: table,
): set figure(supplement: "جدول")

#show figure.where(
  kind: image,
): set figure(supplement: "تصویر")

#show figure: set figure(numbering: "۱")

#show figure: it => block(width: 100%)[#align(center)[
  #it.body
  #text(size: .8em, [
    #strong([#it.supplement #it.counter.display(it.numbering):])
    #it.caption.body
  ])
]]

#show heading: set block(above: 36pt, below: 32pt)
#show heading.where(level: 1): set text(size: 24pt)

#include "heading.typ"

#include "besmellah.typ"
#pagebreak()

#include "signature.typ"
#pagebreak()

#set par(
  first-line-indent: (amount: 1.5em, all: true),
  leading: 15pt,
  justify: true,
)

#include "thanks.typ"
#pagebreak()

#include "summary.typ"
#pagebreak()

#outline(title: [فهرست مطالب])

#pagebreak()

#outline(
  title: [فهرست تصاویر],
  target: figure.where(kind: image),
)

#pagebreak()

#outline(
  title: [فهرست جداول],
  target: figure.where(kind: table),
)


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
// #include "chapters/prev.typ"
// #pagebreak()
#include "chapters/prevent_halucination.typ"
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

#set heading(numbering: none)

#show heading.where(level: 1): it => [
  #set text()
  #linebreak() #linebreak() #it.body
]


= مراجع

#text(lang: "en", font: "Times New Roman", size: 9pt)[

  #bibliography("ref.bib", title: none)

]

#pagebreak()

= واژه‌نامه

// ===================================================================
//  GLOSSARY DATA
//  --- EDIT THIS LIST ---
// ===================================================================
// Add your words here. The code will handle sorting and grouping.
#context {
  let glossary-data = notes.glossary-state.get()
  // ===================================================================
  //  AUTOMATIC GROUPING & FORMATTING
  //  (No need to edit below this line)
  // ===================================================================

  // --- 1. Processing and Grouping Data ---
  // Sort data alphabetically by the Persian term
  let sorted-data = glossary-data.sorted(key: x => x.fa)

  // Group sorted data into a dictionary by the first letter
  let groups = (:)
  for item in sorted-data {
    let first-letter = item.fa.at(0)
    let existing = groups.at(first-letter, default: ())
    groups.insert(first-letter, existing + (item,))
  }

  // --- 2. Formatting Function ---
  // This function formats a single letter-group (e.g., all "الف" words)
  let format-group(letter, items) = {
    // Letter heading (e.g., "الف")
    align(center)[
      #strong[#text(size: 24pt, letter)]
    ]
    v(1em)

    // Table for the entries
    table(
      columns: (5fr, 1fr, 5fr),
      stroke: none,
      align: (left, center, right),

      // Loop through all items for this letter
      ..for item in items {
        (
          item.fa, // Persian part (RTL)
          [......], // Dotted line
          text(dir: ltr)[#item.en], // English part (LTR)
        )
      }
    )
  }

  let persian-order = (
    "ا": 1, "ب": 2, "پ": 3, "ت": 4, "ث": 5,
    "ج": 6, "چ": 7, "ح": 8, "خ": 9, "د": 10,
    "ذ": 11, "ر": 12, "ز": 13, "ژ": 14, "س": 15,
    "ش": 16, "ص": 17, "ض": 18, "ط": 19, "ظ": 20,
    "ع": 21, "غ": 22, "ف": 23, "ق": 24, "ک": 25,
    "گ": 26, "ل": 27, "م": 28, "ن": 29, "و": 30,
    "ه": 31, "ی": 32,
  )

  let sort-key(word) = (
    word.codepoints().map(c => persian-order.at(c, default: 1))
  )

  let all-letters = groups.keys().sorted(key: sort-key)
  // let mid-point = calc.ceil(all-letters.len() / 2)
  // let col1-letters = all-letters.slice(0, mid-point)
  // let col2-letters = all-letters.slice(mid-point)

  // Create the 2-column grid
  grid(
    columns: (1fr),
    gutter: 3em,

    // RIGHT COLUMN (First half of the alphabet)
    // (This is the first grid item, so it appears right in RTL)
    [
      #for letter in all-letters   {
        format-group(letter, groups.at(letter))
        v(2em) // Space between letter sections
      }
    ],

    // LEFT COLUMN (Second half of the alphabet)
    // [
    //   #for letter in col2-letters {
    //     format-group(letter, groups.at(letter))
    //     v(2em) // Space between letter sections
    //   }
    // ],
  )
}

#pagebreak()

#set page(numbering: none)
#set text(lang: "en")

#include "summary_en.typ"

#pagebreak()

#include "heading_en.typ"

