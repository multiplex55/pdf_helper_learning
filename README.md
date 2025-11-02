# PDF Helper Learning

`pdf_helper` provides a high-level wrapper around
[`genpdf`](https://crates.io/crates/genpdf) for assembling richly formatted PDF
reports.  The crate exposes a serialisation-friendly content model together
with a fluent [`PdfBuilder`](crates/pdf_helper/src/builder.rs) that wires the pieces together and
renders final documents.

## Quick start

```rust
use pdf_helper::builder::PdfBuilder;
use pdf_helper::model::{Block, Cover, Section};
use pdf_helper::richtext::Span;

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

For a deeper tour of the API surface and advanced configuration patterns, see
the [pdf_helper guide](crates/pdf_helper/GUIDE.md).

## Builder workflow

1. **Describe the content** using [`Cover`](crates/pdf_helper/src/model.rs) and [`Section`](crates/pdf_helper/src/model.rs)
   values populated with [`Block`](crates/pdf_helper/src/model.rs) instances for paragraphs,
   captioned images, and manual page breaks.
2. **Configure presentation** with [`PdfBuilder`](crates/pdf_helper/src/builder.rs) methods to toggle
   headers, footers, table of contents, paper size, margins, hyphenation, and
   alignment defaults.
3. **Render the document** by calling [`PdfBuilder::render`](crates/pdf_helper/src/builder.rs) (or
   [`render_with_bookmarks`](crates/pdf_helper/src/builder.rs) when the `bookmarks` feature is
   enabled).  The returned [`PdfRenderResult`](crates/pdf_helper/src/builder.rs) exposes the PDF
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

Lower-level configuration is available through [`DocumentBuilder`](crates/pdf_helper/src/builder.rs),
which can be extended with custom headers, footers, and page decorators for more
bespoke layouts.

## Extension points

* **Custom elements** – [`elements`](crates/pdf_helper/src/elements.rs) hosts reusable building
  blocks like captioned images that can be combined with the core model or used
  independently with `genpdf` documents.
* **Rich text parsing** – [`richtext`](crates/pdf_helper/src/richtext.rs) offers utilities to
  convert inline formatting (bold, italic, underline, colours) into `genpdf`
  styled strings, making it straightforward to plug in markdown or custom
  markup pipelines.
* **Bookmarks integration** – enabling the `bookmarks` feature pulls in
  [`lopdf`](https://crates.io/crates/lopdf) and activates
  [`PdfBuilder::render_with_bookmarks`](crates/pdf_helper/src/builder.rs) for post-processing the
  rendered bytes with hierarchical outlines.
* **Hyphenation** – the optional `hyphenation` feature embeds a US-English
  dictionary and wires it into the underlying `genpdf::Document`.

## Examples

* `cargo run --example report` writes `report.pdf` showcasing covers, rich
  paragraphs, images, and a printed table of contents.
* `cargo run --example report_bookmarks --features bookmarks` runs the same
  workflow but augments the output with navigable section bookmarks.
* `cargo run --example run_all` renders both variants in
  `target/run_all_examples/`, skipping the bookmarks pass when the feature is
  disabled.
* `cargo run --example run_all --features bookmarks` generates the same
  directory but includes the bookmarks-enhanced PDF alongside the standard
  render.

Alternatively invoke the workspace CLI: `cargo run -p main -- report`,
`cargo run -p main -- report-bookmarks --features bookmarks`, or
`cargo run -p main -- run-all`.

## Fonts

The helper API expects the Roboto font family to be available at runtime.  The
search order is:

1. The directory pointed to by the `PDF_HELPER_FONTS_DIR` environment variable.
2. `assets/fonts` next to the compiled binary (copy this directory when
   packaging your application).
3. `assets/fonts` in the crate source tree (`crates/pdf_helper/assets/fonts`,
   convenient for `cargo run` during development).

The repository ships without the actual font files; add
`Roboto-Regular.ttf`, `Roboto-Bold.ttf`, `Roboto-Italic.ttf`, and
`Roboto-BoldItalic.ttf` to one of those locations before running the examples or
integration tests.  See `crates/pdf_helper/assets/fonts/README.md` for a quick reminder when
setting up a local checkout or bundling the fonts alongside your binaries.

If none of the Roboto assets can be found the library attempts to load the
Windows 11 Arial family instead (`arial.ttf`, `arialbd.ttf`, `ariali.ttf`,
`arialbi.ttf`).  The helper checks the `PDF_HELPER_WINDOWS_FONTS_DIR`
environment variable first and, when running on Windows, falls back to the
standard `%WINDIR%\Fonts` directory.  A warning is emitted through the `log`
facade whenever the fallback activates so consumers can provision the preferred
Roboto family when desired.

## Testing

Run `cargo test` to execute unit tests, integration tests, and documentation
examples.  The integration tests render small PDFs, verify that the output is
non-empty, and confirm deterministic rendering by hashing the produced bytes;
they isolate the font search paths during setup so the fallback path is
exercised when the Roboto assets are missing.  When neither Roboto nor the
configured Windows fonts are available (e.g., on non-Windows CI) the tests skip
the rendering assertions with a note explaining the missing fonts.
