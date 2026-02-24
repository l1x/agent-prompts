<!--file:create-typst-template.md-->

# Create Typst Template

## Objective

Generate a Typst template for Pandoc (`--pdf-engine=typst --template=<file>`) that renders Markdown → PDF aligned to a company's design system. The template must feel slick, modern, and high‑tech while remaining highly readable and professional. It applies typesetting best practices for long‑form reading material and uses color sparingly — brand accent for emphasis only, never for large fills or dense data.

## Inputs & References (Required)

- **Design system tokens & assets**: use the provided design system folder or ask for if it was not provided.
- **Visual language doc** (optional): if provided, follow its color-usage rules

## Process

1. **Analyze Design System** — Read and extract: brand primary, hover, light variant; neutral/surface palette; spacing scale; border-radius scale
2. **Map Tokens to Typst** — Define `#let` variables for every brand color and semantic alias. Use `rgb()` for hex values.
3. **Build Template Structure** — Implement every section listed in Template Architecture below
4. **Wire Pandoc Variables** — Support all variables listed in Pandoc Compatibility
5. **Verify** — Build a test markdown file with all element types (H1–H4, lists, tables, code, blockquotes, links, images) and confirm the PDF renders correctly with `pandoc --pdf-engine=typst --template=<output>`

## Requirements

1. **Design Alignment**
   - Reflect the company design system (colors, typography, spacing, and hierarchy).
   - Use color lightly: brand accent for heading accents, list markers, link color, and thin rules only. Never for backgrounds, large fills, or chart data.
   - Neutral palette (warm grays) for surfaces, borders, muted text.
   - Typography colors must be fixed neutrals (black/gray from the design system). No headings should use the primary brand color.

2. **Readability & Typesetting Best Practices**
   - Justified text with proper leading (≥1.3× body size)
   - Generous margins (2.5cm recommended for A4)
   - Font size hierarchy: body 11pt, H3 13pt, H2 16pt, H1 24pt, cover title 32pt
   - Section numbering for H2 and H3; H1 is document-level title, unnumbered
   - Tables: 9pt body text, bold header row, alternating row fills using neutral palette
   - Page numbers in footer (centered, "1 / N" format); document title + date in header (from page 2 onward)

3. **Template Features**
   - Cover page: toggled by `$coverpage$` pandoc variable, not forced
   - Auto-TOC: always on its own page right after the title/cover. If a cover page exists, it already ends with `pagebreak()`, so the TOC naturally starts next. If no cover page, the TOC becomes page 1 (no pagebreak before). Styled with black text (not link color).
   - H1 excluded from TOC; only H2 and H3 entries shown
   - Blockquotes: left border in brand accent, card feel (subtle border + white fill), italic text
   - Code: inline gets light background pill; blocks get light background with radius
   - Lists: custom markers using brand accent color (e.g., `▸` chevron). Marker size should scale with body text; avoid oversized markers in dense lists.
   - Term lists: bold term, indented description

4. **Output Style**
   - Modern, minimal, and precise. Avoid ornamental or overly stylized elements.
   - Default to grayscale with warm neutrals. Brand accent appears only as thin lines, small markers, and link color.

## Template Architecture

Every generated template must contain these sections in order:

1. **Brand token definitions** — `#let` variables for all colors + semantic aliases
2. **Global show/set rules** — term lists, table defaults, figure captions, horizontal rule
3. **Cover page function** — takes title, subtitle, date, authors; uses brand accent bars (top/bottom), document type badge, metadata grid
4. **Main `conf()` function** — accepts all pandoc-wired parameters; sets up:
   - Document metadata (`set document`)
   - Page setup (`set page` with header/footer)
   - Text defaults (`set text`)
   - Paragraph settings (`set par` with justify + leading)
   - Heading numbering (skip H1 level, number H2 and H3 only)
   - Outline (TOC) show rule — own page, black text, styled entries
   - Heading show rules — H1 (large, bold), H2 (with underline rule), H3 (with accent number/chevron), H4 (muted, smaller)
   - Link, blockquote, table, code, list show/set rules
   - Title block (when no cover page)
   - Abstract block (optional)
   - Content output (conditional columns wrapping)
5. **Pandoc variable wiring** — `#show: doc => conf(...)` block mapping all pandoc variables

## Pandoc Compatibility

The template must support these pandoc template variables:

| Variable                     | Type    | Maps to                                         |
| ---------------------------- | ------- | ----------------------------------------------- |
| `$title$`                    | content | Document title                                  |
| `$subtitle$`                 | content | Subtitle                                        |
| `$author$`                   | list    | Author names (with optional affiliation, email) |
| `$date$`                     | content | Date string                                     |
| `$abstract$`                 | content | Abstract body                                   |
| `$abstract-title$`           | content | Abstract heading                                |
| `$lang$`, `$region$`         | string  | Text language settings                          |
| `$papersize$`                | string  | Paper size (default: a4)                        |
| `$fontsize$`                 | length  | Override body font size                         |
| `$coverpage$`                | boolean | Enable cover page                               |
| `$columns$`                  | integer | Multi-column layout                             |
| `$highlighting-definitions$` | raw     | Syntax highlighting                             |

Use pandoc's `$if(var)$...$endif$` conditionals for all optional variables.

Enable TOC via CLI (example): `pandoc --toc --toc-depth=2 --pdf-engine=typst --template=...`

## Known Pitfalls

These are hard-won lessons. Follow them exactly:

1. **`pagebreak()` inside `columns()`**: Typst errors with "pagebreaks are not allowed inside containers." Fix: wrap content conditionally — `if cols > 1 { columns(cols, doc) } else { doc }` — so single-column mode (the common case) avoids the container entirely
2. **`outline.entry` API (Typst 0.14+)**: Fields like `body`, `page`, `prefix` are _methods_, not fields. Use `entry.body()`, `entry.page()`, `entry.prefix()` — not `entry.body`
3. **Double TOC from state**: Using `state("toc-placed", false)` to track whether TOC has been inserted causes all `context` blocks to see the initial value in the same layout pass, producing duplicate TOCs and "layout did not converge" warnings. Fix: use `query(heading.where(level: 2))` and compare `.location()` to detect the first H2 — no state needed
4. **TOC link color bleeding**: Links inside `outline` inherit the document's `linkcolor`. Fix: add both `set text(fill: black)` and `show link: set text(fill: black)` inside the outline show rule
5. **Pandoc `--toc` and default TOC rendering**: When the user passes `--toc`, pandoc sets the `$toc$` variable and provides a `$table-of-contents$` block. Do not render `$table-of-contents$` — it is unstyled and positioned wrong. Instead, check `$if(toc)$` and render a custom Typst `outline()` with brand styling

## Output

- A single `.template.typ` file compatible with `pandoc --pdf-engine=typst --template=<file>`
- Include a brief summary of the design decisions and how they map to the design system.

## Constraints

- Do not introduce new brand colors; use existing design system tokens only.
- Do not require frontmatter or raw Typst blocks in Markdown files. The template must work with plain Markdown + pandoc variables passed via CLI or defaults file.
- Prioritize clarity and editorial quality over visual flair.
- Respect the project's existing tech stack and architecture.
- Test with real content: the template must handle documents from 2 pages to 30+ pages gracefully.
- All features (TOC, cover page, columns, etc.) are controlled via pandoc CLI flags (`--toc`, `-V coverpage=true`, etc.), never via markdown frontmatter or raw Typst blocks. The template reads these flags and handles rendering and styling.
- Do not write implementation code outside the template itself.
