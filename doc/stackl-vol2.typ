/* -----------------------------------------------------------------------
 * DOCUMENT SETUP
 * ----------------------------------------------------------------------- */

#set page(
  paper: "a4",
  margin: 1in, // Standard 1in margin
)
#set text(
  size: 11pt,
  font: "Liberation Sans", // Standard Typst font
)

// Enable standard numbering for chapters and sections (for later use)
#set heading(numbering: "1")

---

/* -----------------------------------------------------------------------
 * TITLE PAGE
 * ----------------------------------------------------------------------- */

#align(center)[
  #v(20%) // Vertical space before the title

  // Title Part 1
  #text(30pt, weight: "bold")[STACKL Architecture]

  #v(-1.2em)

  // Title Part 2
  #text(30pt, weight: "bold")[Programmer's Manual]

  #v(3em)

  // Title Part 3
  #text(30pt, weight: "bold")[Volume 2: Instruction Set Reference]

  #v(1fr) // Push content to the bottom
]
