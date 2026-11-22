// SPDX-License-Identifier: GPL-3.0-or-later
// Cruncha's calc-package report template (spike S6). It reads the report from
// /report.json (crates/cruncha-report/src/model.rs) and lays out: cover with title block,
// revisions and check summary; contents; the calculation; a provenance appendix.

#let r = json("/report.json")
#let t = r.title
#let s = r.style
#let p = r.provenance
#let accent = rgb(s.accent)
#let muted = luma(105)
#let rule = luma(200)
#let ok-color = rgb("#1b7f3b")
#let fail-color = rgb("#b42318")
#let warn-color = rgb("#b54708")
#let echo-color = rgb("#0000ff")

#let title-text = if t.title != "" { t.title } else { p.documentName }
#let short-hash = if p.contentHash.len() >= 12 { p.contentHash.slice(0, 12) } else { p.contentHash }
#let current-rev = if r.revisions.len() > 0 { r.revisions.last().label } else { "—" }
#let or-dash(x) = if x == none or x == "" { [—] } else { x }

// ---------------------------------------------------------------------------------------
// Page, header and footer

#set document(title: title-text, author: if t.preparedBy != "" { t.preparedBy } else { () })
#set text(font: s.fonts, size: s.fontSize * 1pt, lang: "en")
#set par(justify: false, leading: 0.6em, spacing: 0.9em)
#show math.equation: set text(font: ("New Computer Modern Math",))
#show raw: set text(font: ("DejaVu Sans Mono",), size: 0.85em)

#let header() = {
  set text(size: 0.8em)
  grid(
    columns: (1fr, 2fr, 1fr),
    align: (left + horizon, center + horizon, right + horizon),
    if s.logo != none { image(s.logo, height: 1.1cm) } else { strong(s.company) },
    [#strong(title-text) \ #text(fill: muted)[#t.project]],
    [#text(fill: muted)[Project No.] #or-dash(t.projectNumber) \
     #text(fill: muted)[Sheet] #context counter(page).display("1 of 1", both: true)],
  )
  v(-0.35em)
  line(length: 100%, stroke: 0.8pt + accent)
}

#let footer() = {
  set text(size: 0.72em, fill: muted)
  line(length: 100%, stroke: 0.4pt + rule)
  v(-0.45em)
  grid(
    columns: (1fr, auto),
    [By #or-dash(t.preparedBy) · Checked #or-dash(t.checkedBy) · Rev #current-rev · #p.committedAt.slice(0, 10)],
    [#p.documentName · version #p.version · #short-hash · Cruncha #p.crunchaVersion],
  )
}

#set page(
  paper: s.paper,
  margin: (top: 2.9cm, bottom: 2.2cm, x: 2cm),
  header: header(),
  header-ascent: 18%,
  footer: footer(),
)

#set heading(numbering: "1.1")
#show heading: it => {
  set text(fill: accent)
  block(above: 1.3em, below: 0.7em, sticky: true, it)
}
#show heading.where(level: 1): set text(size: 1.35em)
#show heading.where(level: 2): set text(size: 1.15em)
#show heading.where(level: 3): set text(size: 1.0em)
#show quote.where(block: true): it => block(inset: (left: 10pt, y: 2pt), stroke: (left: 1.5pt + muted), it.body)
#show link: it => text(fill: accent, underline(it))

// ---------------------------------------------------------------------------------------
// Pieces

#let runs(rs) = for x in rs {
  let c = x.text
  if x.code { c = raw(c) }
  if x.bold { c = strong(c) }
  if x.italic { c = emph(c) }
  if x.at("sub", default: false) { c = sub(c) }
  if x.at("sup", default: false) { c = super(c) }
  if x.at("link", default: none) != none { c = link(x.link, c) }
  c
}

// Prose blocks: paragraphs, lists and quotes nest (a list item holds paragraphs, lists and
// quotes); headings and rules are top-level only.
#let prose(b) = {
  if b.type == "paragraph" {
    par(runs(b.text))
  } else if b.type == "list" {
    let items = b.items.map(item => item.map(prose).join())
    if b.ordered { enum(start: b.start, ..items) } else { list(..items) }
  } else if b.type == "quote" {
    quote(block: true, b.blocks.map(prose).join())
  } else if b.type == "rule" {
    line(length: 100%, stroke: 0.5pt + muted)
  }
}

#let badge(color, body) = box(
  inset: (x: 4pt, y: 2pt), radius: 2pt, fill: color.lighten(88%), stroke: 0.5pt + color,
  text(size: 0.8em, weight: "bold", fill: color, body),
)

#let status-badge(st) = {
  if st.state == "pass" { badge(ok-color)[OK ✓] }
  else if st.state == "fail" { badge(fail-color)[NOT OK ✗] }
  else if st.state == "error" { badge(fail-color)[ERROR] }
  else if st.state == "notCalculated" { badge(warn-color)[NOT CALCULATED] }
}

#let problem(st) = {
  if st.state == "error" { text(size: 0.85em, fill: fail-color)[#st.message] }
  else if st.state == "notCalculated" { text(size: 0.85em, fill: warn-color)[#st.reason] }
}

#let stripe(st) = {
  if st.state in ("error", "fail") { fail-color }
  else if st.state == "notCalculated" { warn-color }
  else { none }
}

#show math.equation.where(block: true): set align(left)
#show math.equation.where(block: true): set block(above: 0pt, below: 0pt)

#let calc-block(b) = {
  let c = stripe(b.status)
  let math-part = if b.math != "" {
    math.equation(block: true, eval(b.math, mode: "math"))
  } else {
    raw(b.text)
  }
  block(
    breakable: false, width: 100%, above: 0.55em, below: 0.55em,
    inset: (left: if c != none { 6pt } else { 0pt }, y: 2pt),
    stroke: if c != none { (left: 2pt + c) } else { none },
  )[
    #grid(
      columns: (1fr, auto),
      column-gutter: 12pt,
      align: (left + horizon, right + horizon),
      math-part,
      {
        if b.description != "" { text(size: 0.9em, style: "italic", fill: muted, b.description) }
        if b.status.state != "ok" { h(6pt); status-badge(b.status) }
      },
    )
    #if b.status.state == "notCalculated" { problem(b.status) }
    #label("cell-" + b.id)
  ]
}

#let code-block(b) = {
  let c = stripe(b.status)
  block(width: 100%, above: 0.9em, below: 0.9em, breakable: true,
    stroke: if c != none { (left: 2pt + c) } else { none },
    inset: (left: if c != none { 6pt } else { 0pt }))[
    // The label and the source stay together (short sources never split across pages).
    #block(breakable: b.source.split("\n").len() > 40, width: 100%)[
      #text(size: 0.75em, fill: muted, weight: "bold", upper(b.lang))
      #if b.status.state != "ok" { h(4pt); status-badge(b.status) }
      #if r.options.showCode and b.source != "" {
        block(width: 100%, fill: luma(246), inset: 7pt, radius: 3pt, above: 0.4em, raw(b.source, lang: b.lang, block: true))
      }
    ] #label("cell-" + b.id)
    #problem(b.status)
    #if b.at("provenance", default: none) != none {
      block(above: 0.4em, text(size: 0.85em, style: "italic", fill: warn-color, b.provenance))
    }
    #for o in b.outputs {
      if o.type == "image" {
        block(breakable: false, above: 0.6em, align(center, image(o.path, width: 85%)))
      } else if o.type == "figure" {
        if o.path != none {
          block(breakable: false, above: 0.6em, align(center, image(o.path, width: 85%)))
        } else {
          block(breakable: false, above: 0.6em, width: 100%, inset: 8pt, radius: 3pt, stroke: 0.7pt + warn-color, {
            badge(warn-color)[FIGURE NOT RENDERED]
            h(6pt)
            if o.reason != none { text(size: 0.85em, fill: warn-color, o.reason) }
            if o.summary != "" { linebreak(); text(size: 0.85em, fill: muted, o.summary) }
          })
        }
      } else if o.type == "table" {
        let n = o.columns.len()
        block(above: 0.6em, {
          set text(size: 0.82em)
          table(
            columns: n,
            stroke: (x, y) => (bottom: if y == 0 { 0.7pt + rule } else { 0.3pt + luma(225) }),
            fill: (x, y) => if y == 0 { luma(240) },
            align: right,
            table.header(..o.columns.map(strong)),
            ..o.rows.flatten(),
          )
          if o.totalRows > o.rows.len() {
            text(fill: muted)[First #o.rows.len() of #o.totalRows rows.]
          }
        })
      } else if o.type == "text" {
        block(above: 0.6em, text(fill: if o.stream == "stderr" { warn-color } else { black }, raw(o.text, block: true)))
      }
    }
    #if b.defines.len() > 0 {
      set text(size: 0.82em, fill: muted)
      [Defines: ]
      b.defines.map(d => [#text(fill: black, raw(d.at(0), lang: none)) = #d.at(1)]).join([ · ])
    }
  ]
}

// ---------------------------------------------------------------------------------------
// Cover

#let field(name, value) = (text(fill: muted, size: 0.85em, name), or-dash(value))

#{
  v(1.5cm)
  text(size: 0.9em, fill: accent, weight: "bold", upper[Calculation])
  v(-0.4em)
  text(size: 2.1em, weight: "bold", title-text)
  if t.subject != "" { linebreak(); text(size: 1.2em, fill: muted, t.subject) }
  v(0.6cm)
  grid(
    columns: (auto, 1fr, auto, 1fr),
    column-gutter: 10pt, row-gutter: 8pt,
    ..field("Project", t.project), ..field("Project No.", t.projectNumber),
    ..field("Client", t.client), ..field("Date", p.committedAt.slice(0, 10)),
    ..field("Prepared by", t.preparedBy), ..field("Checked by", t.checkedBy),
    ..field("Approved by", t.approvedBy), ..field("Revision", current-rev),
  )
  v(0.8cm)
  text(weight: "bold", fill: accent)[Revisions]
  v(-0.3em)
  table(
    columns: (auto, auto, 1fr, auto),
    stroke: (x, y) => (bottom: 0.4pt + rule),
    fill: (x, y) => if y == 0 { luma(242) },
    inset: 5pt,
    table.header([*Rev*], [*Date*], [*Description*], [*By*]),
    ..if r.revisions.len() == 0 { ([—], [—], [No snapshots yet], [—]) } else {
      r.revisions.map(v => (strong(v.label), v.date, v.description, v.by)).flatten()
    },
  )
}

// The check summary: every check and every problem, with the sheet it is on.
#let checks = r.blocks.filter(b => b.type == "calc" and b.check)
#let problems = r.blocks.filter(b => (b.type == "calc" or b.type == "code") and b.status.state in ("error", "notCalculated"))
#let elsewhere = r.blocks.filter(b => b.type == "code" and b.at("provenance", default: none) != none)
#let sheet-of(id) = context {
  let found = query(label("cell-" + id))
  if found.len() > 0 { str(counter(page).at(found.first().location()).first()) } else { "?" }
}

#if r.options.checkSummary {
  let passed = checks.filter(b => b.status.state == "pass").len()
  let failed = checks.filter(b => b.status.state == "fail").len()
  let open = checks.len() - passed - failed
  v(0.8cm)
  text(weight: "bold", fill: accent)[Summary]
  v(-0.3em)
  block[
    #badge(ok-color)[#passed of #checks.len() checks OK]
    #if failed > 0 { badge(fail-color)[#failed NOT OK] }
    #if open > 0 { badge(warn-color)[#open not calculated] }
    #if problems.len() > 0 { badge(warn-color)[#problems.len() cells with problems] }
    #if elsewhere.len() > 0 { badge(warn-color)[#elsewhere.len() code cells not reproduced here] }
  ]
  if elsewhere.len() > 0 {
    text(size: 0.85em, fill: muted)[Output of #elsewhere.len() code #if elsewhere.len() == 1 [cell was] else [cells were] computed on another computer and read from the document's stored results; checks that use it were evaluated here.]
  }
  if checks.len() + problems.len() > 0 {
    set text(size: 0.85em)
    table(
      columns: (1fr, auto, auto),
      stroke: (x, y) => (bottom: 0.3pt + rule),
      inset: 4pt,
      align: (left + horizon, center + horizon, right + horizon),
      table.header([*Check*], [*Result*], [*Sheet*]),
      ..checks.map(b => (
        if b.math != "" { math.equation(block: false, eval(b.math, mode: "math")) } else { raw(b.text) },
        status-badge(b.status), sheet-of(b.id),
      )).flatten(),
      ..problems.filter(b => not b.at("check", default: false)).map(b => (
        text(fill: muted)[#if b.type == "code" [#upper(b.lang) cell] else [Calc] — #problem(b.status)],
        status-badge(b.status), sheet-of(b.id),
      )).flatten(),
    )
  }
}

// Contents follow the summary on the cover, and flow onto the next sheet when long.
#if r.options.tableOfContents {
  v(0.8cm)
  outline(title: text(size: 1em, fill: accent, weight: "bold")[Contents], indent: auto, depth: 3)
}

#pagebreak()

// ---------------------------------------------------------------------------------------
// The calculation

#for b in r.blocks {
  if b.type == "heading" {
    heading(level: calc.min(b.level, 6), runs(b.text))
  } else if b.type in ("paragraph", "list", "quote", "rule") {
    prose(b)
  } else if b.type == "calc" {
    calc-block(b)
  } else if b.type == "code" {
    code-block(b)
  } else {
    block(fill: luma(246), inset: 6pt, width: 100%)[#text(size: 0.8em, fill: muted, b.kind) \ #raw(b.source)]
  }
}

// ---------------------------------------------------------------------------------------
// Provenance

#pagebreak(weak: true)
#heading(numbering: none)[Provenance]
#text(fill: muted, size: 0.9em)[This report was produced from one version of the document. The content hash covers every cell; a document with the same hash gives the same results.]
#v(0.4em)
#table(
  columns: (auto, 1fr),
  stroke: (x, y) => (bottom: 0.3pt + rule),
  inset: 5pt,
  [Document], p.documentName,
  [Document id], raw(p.documentId),
  [Version], [#p.version #if p.snapshot != none [(snapshot #strong(p.snapshot))]],
  [Committed], p.committedAt,
  [Content hash (SHA-256)], raw(p.contentHash),
  [Cruncha], p.crunchaVersion,
  [Python environment], if p.pythonEnv != none { raw(p.pythonEnv) } else { [—] },
)
