# PDF Helper Learning

`pdf_helper_learning` provides a high-level wrapper around
[`genpdf`](https://crates.io/crates/genpdf) for assembling richly formatted PDF
reports.  The crate exposes a serialisation-friendly content model together
with a fluent [`PdfBuilder`](src/builder.rs) that wires the pieces together and
renders final documents.

## Quick start

```rust
use pdf_helper_learning::builder::PdfBuilder;
use pdf_helper_learning::model::{Block, Cover, Section};
use pdf_helper_learning::richtext::Span;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cover = Cover::new("Quarterly Report").with_block(Block::paragraph(vec![
        Span::new("Prepared by ").italic(),
        Span::new("Data Automation Team").bold(),
    ]));

    let body = Section::new("Highlights").with_block(Block::paragraph(vec![
        Span::new("Sales exceeded expectations by 24%.").bold(),
    ]));

    let pdf = PdfBuilder::new()
        .with_cover(cover)
        .add_section(body)
        .render()?;

    std::fs::write("report.pdf", &pdf.bytes)?;
    Ok(())
}
```

## Builder workflow

1. **Describe the content** using [`Cover`](src/model.rs) and [`Section`](src/model.rs)
   values populated with [`Block`](src/model.rs) instances for paragraphs,
   captioned images, and manual page breaks.
2. **Configure presentation** with [`PdfBuilder`](src/builder.rs) methods to toggle
   headers, footers, table of contents, paper size, margins, hyphenation, and
   alignment defaults.
3. **Render the document** by calling [`PdfBuilder::render`](src/builder.rs) (or
   [`render_with_bookmarks`](src/builder.rs) when the `bookmarks` feature is
   enabled).  The returned [`PdfRenderResult`](src/builder.rs) exposes the PDF
   bytes together with per-section start pages that can feed downstream systems.

The builder runs two passes when section metadata or a printed table of contents
is requested.  The initial dry run records section start pages, while the second
pass produces the final bytes (and optionally applies bookmark annotations).
This ensures that repeated renders with the same inputs are deterministic.

## Configuration options

`PdfBuilder` exposes a fluent API for shaping the final output:

| Method | Effect |
| ------ | ------ |
| `with_paper_size(Size)` | Override the default paper size. |
| `with_margins(Margins)` | Apply custom page margins. |
| `show_header(bool)` / `show_footer(bool)` | Toggle the automatically generated title header and page-number footer. |
| `enable_hyphenation(bool)` | Use the embedded US-English hyphenation dictionary (requires the `hyphenation` feature). |
| `with_cover(Cover)` | Attach a cover page rendered before the sections. |
| `add_section(Section)` / `with_sections(Vec<Section>)` | Provide the body content. |
| `include_printed_toc(bool)` | Emit a table of contents page. |
| `with_toc_title(Option<String>)` | Customise the printed TOC heading. |
| `with_default_alignment(HorizontalAlignment)` | Pick the alignment applied when sections omit explicit preferences. |
| `render_section_headings(bool)` | Control whether section titles are promoted to headings automatically. |
| `collect_section_pages(bool)` | Record the first page of each section without affecting the rendered output. |

Lower-level configuration is available through [`DocumentBuilder`](src/builder.rs),
which can be extended with custom headers, footers, and page decorators for more
bespoke layouts.

## Extension points

* **Custom elements** – [`elements`](src/elements.rs) hosts reusable building
  blocks like captioned images that can be combined with the core model or used
  independently with `genpdf` documents.
* **Rich text parsing** – [`richtext`](src/richtext.rs) offers utilities to
  convert inline formatting (bold, italic, underline, colours) into `genpdf`
  styled strings, making it straightforward to plug in markdown or custom
  markup pipelines.
* **Bookmarks integration** – enabling the `bookmarks` feature pulls in
  [`lopdf`](https://crates.io/crates/lopdf) and activates
  [`PdfBuilder::render_with_bookmarks`](src/builder.rs) for post-processing the
  rendered bytes with hierarchical outlines.
* **Hyphenation** – the optional `hyphenation` feature embeds a US-English
  dictionary and wires it into the underlying `genpdf::Document`.

## Examples

* `cargo run --example report` writes `report.pdf` showcasing covers, rich
  paragraphs, images, and a printed table of contents.
* `cargo run --example report_bookmarks --features bookmarks` runs the same
  workflow but augments the output with navigable section bookmarks.

## Fonts

The helper API expects the Roboto font family to live in
`assets/fonts`.  The repository ships without the actual font files; add
`Roboto-Regular.ttf`, `Roboto-Bold.ttf`, `Roboto-Italic.ttf`, and
`Roboto-BoldItalic.ttf` to that directory (or adjust your configuration) before
running the examples or integration tests.  See `assets/fonts/README.md` for a
quick reminder when setting up a local checkout.

## Testing

Run `cargo test` to execute unit tests, integration tests, and documentation
examples.  The integration tests render small PDFs, verify that the output is
non-empty, and confirm deterministic rendering by hashing the produced bytes;
they skip automatically when the bundled fonts are not present.
