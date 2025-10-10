
#let cntrs_id = "note_cntr"
#let notes = state("dict", (:))
#let note(it, note) = context {
	let pg = 0
  let cntr = counter(cntrs_id + str(pg))
  cntr.step()
  notes.update(dict => {
    let vec = ()
    if str(pg) in dict.keys() {
      vec = dict.at(str(pg))
    }
    vec.push(note)
    dict.insert(str(pg), vec)
    dict
  })
  it + super(numbering("۱", cntr.get().at(0)))
}

#let display(k) = context {
	set text(size: 8pt)
	let pg = 0
  let dict = notes.final()
  let vec = ()
  if str(pg) in dict.keys() {
    vec = dict.at(str(pg))
  }
  
  if vec.len() > 0 {
      let t = calc.ceil(vec.len() / 3)
      for i in range(k * t, calc.min((k + 1) * t, vec.len())) {
          super[#numbering("۱", i + 1)] + h(1pt) + vec.at(i)
          linebreak()
      }
  }
}
