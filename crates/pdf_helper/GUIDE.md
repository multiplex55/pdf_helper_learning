# `pdf_helper` Guide

This guide walks through installing the crate, producing your first document, and
exploring the richer building blocks that power the bundled examples.

## Installation

Add `pdf_helper` to your project and opt into optional features as needed:

```toml
[dependencies]
pdf_helper = { version = "0.1", features = ["hyphenation", "bookmarks"] }
```

The `hyphenation` feature embeds a US-English dictionary to smooth paragraph
layout, while `bookmarks` enables post-processing that injects hierarchical
outlines into the rendered PDF.

## Quick start

The fastest way to produce a PDF is to reuse the ready-made sample builder from
the examples crate:

```rust
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    pdf_helper::examples::report::run()
}
```

This mirrors `cargo run --example report`, writing `report.pdf` to the current
working directory by combining a cover page, multiple sections, a table of
contents, and generated placeholder imagery. Ensure that the Roboto font family
is discoverable (see the caveats section) before running the example.

## Advanced topics

### Crafting covers and sections

`pdf_helper::examples::shared::build_sample_report_builder` demonstrates how to
assemble richly formatted covers and narrative sections. The following excerpt
illustrates paragraph construction with inline styling and colour accents:

```rust
use pdf_helper::model::{Block, Cover, Section};
use pdf_helper::richtext::Span;
use genpdf::style::Color;

let cover = Cover::new("Engineering Highlights")
    .with_subtitle(Some("Spring Edition".to_string()))
    .with_block(Block::paragraph(vec![
        Span::new("Prepared for the ").italic(),
        Span::new("Architecture Guild").bold(),
        Span::new(" to summarise quarterly progress across shared platforms."),
    ]))
    .with_block(Block::paragraph(vec![
        Span::new("This briefing blends narrative summaries, quantitative dashboards, and roadmap context so stakeholders can absorb the full story before diving into team-level detail."),
    ]));

let highlights = Section::new("Executive Highlights")
    .with_block(Block::paragraph(vec![
        Span::new("Over the last quarter, the "),
        Span::new("Platform Engineering").bold(),
        Span::new(" guild accelerated delivery on migration initiatives, reducing mean rollout time by "),
        Span::new("38%")
            .bold()
            .colored(Color::Rgb(40, 120, 90)),
        Span::new(", while maintaining a "),
        Span::new("99.95%")
            .bold()
            .colored(Color::Rgb(32, 102, 148)),
        Span::new(" service uptime across customer-facing surfaces."),
    ]));
```

Additional blocks can be chained onto each section to introduce bullet lists,
call-outs, or further narrative paragraphs.

### Centered and aligned imagery

The same sample builder shows how to embed centered hero imagery and
right-aligned dashboards by combining `ImageBlock` with byte-backed assets. The
snippet below includes the helper that generates PNG bytes on the fly so it can
compile standalone:

```rust
use std::io::Cursor;

use image::{DynamicImage, ImageBuffer, ImageOutputFormat, Rgb};
use pdf_helper::model::{HorizontalAlignment, ImageBlock, ImageSource, RichParagraph};
use pdf_helper::richtext::Span;

const HERO_IMAGE_WIDTH_MM: f64 = 120.0;

let hero_image = ImageBlock::new(ImageSource::from_bytes(generate_placeholder_image()?))
    .with_caption(Some(RichParagraph::new(vec![
        Span::new("Figure 1: ").bold(),
        Span::new("Narrative montage of delivery milestones across the quarter."),
    ])))
    .with_alignment(HorizontalAlignment::Center)
    .with_width_mm(Some(HERO_IMAGE_WIDTH_MM));

fn generate_placeholder_image() -> Result<Vec<u8>, image::ImageError> {
    generate_gradient_image(240, 140, [78, 102, 148], [228, 188, 152])
}

fn generate_gradient_image(
    width: u32,
    height: u32,
    start: [u8; 3],
    end: [u8; 3],
) -> Result<Vec<u8>, image::ImageError> {
    let width_f = (width.saturating_sub(1)) as f32;
    let height_f = (height.saturating_sub(1)) as f32;
    let buffer = ImageBuffer::from_fn(width, height, |x, y| {
        let xf = if width_f > 0.0 {
            x as f32 / width_f
        } else {
            0.0
        };
        let yf = if height_f > 0.0 {
            y as f32 / height_f
        } else {
            0.0
        };
        let mix = (0.65 * xf + 0.35 * yf).clamp(0.0, 1.0);
        let mut channels = [0u8; 3];
        for (index, channel) in channels.iter_mut().enumerate() {
            let start = start[index] as f32;
            let end = end[index] as f32;
            *channel = (start + (end - start) * mix).round().clamp(0.0, 255.0) as u8;
        }
        let accent = ((xf - yf).abs() * 40.0).round().clamp(0.0, 40.0) as u8;
        channels[0] = channels[0].saturating_add(accent);
        channels[2] = channels[2].saturating_sub(accent.saturating_div(2));
        Rgb(channels)
    });

    let mut bytes = Vec::new();
    DynamicImage::ImageRgb8(buffer)
        .write_to(&mut Cursor::new(&mut bytes), ImageOutputFormat::Png)?;
    Ok(bytes)
}
```

`ImageSource` also accepts filesystem paths via `ImageSource::from_path` when
shipping production assets; prefer relative paths that resolve alongside your
binary or embed bytes via `include_bytes!` for fully self-contained builds.

### Custom headers, footers, and tables of contents

Header and footer closures grant direct access to `genpdf` primitives for
bespoke layouts, while the printed table of contents uses a `TableLayout` to
render two-column entries:

```rust
use genpdf::elements::{LinearLayout, Paragraph};
use genpdf::Alignment;
use pdf_helper::builder::PdfBuilder;

let header_title = "Engineering Highlights".to_string();
let header_edition = "Spring Edition • April 2024".to_string();
let footer_contact = "https://intranet.example.com/reports".to_string();
let footer_note = "Automation & Insights • Reach out for dataset refreshes".to_string();

let builder = PdfBuilder::new()
    .with_header({
        let title = header_title.clone();
        let edition = header_edition.clone();
        move |_| {
            let mut layout = LinearLayout::vertical();
            let mut title_line = Paragraph::new(title.clone());
            title_line.set_alignment(Alignment::Center);
            layout.push(title_line);

            let mut edition_line = Paragraph::new(edition.clone());
            edition_line.set_alignment(Alignment::Center);
            layout.push(edition_line);

            layout
        }
    })
    .with_footer(18.0, {
        let contact = footer_contact.clone();
        let note = footer_note.clone();
        move |page| {
            let mut layout = LinearLayout::vertical();

            let mut contact_line = Paragraph::new(format!("Contact: {}", contact));
            contact_line.set_alignment(Alignment::Right);
            layout.push(contact_line);

            let mut page_line = Paragraph::new(format!("Page {} • {}", page, note));
            page_line.set_alignment(Alignment::Right);
            layout.push(page_line);

            layout
        }
    })
    .include_printed_toc(true)
    .with_toc_title(Some("Contents".to_string()));
```

Enable the `bookmarks` feature and switch to `render_with_bookmarks` to add
hierarchical outlines that mirror the printed table of contents.

### Rendering multiple variants programmatically

`pdf_helper::examples::run_all::run` demonstrates how to orchestrate multiple
renders into a shared output directory, optionally producing a bookmarks-enabled
variant when the feature flag is available:

```rust
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    pdf_helper::examples::run_all::run()
}
```

Adopt the same pattern for integration tests or CLI entry points that need to
manage multiple output artefacts.

## Caveats and integration notes

* **Font discovery** – The library looks for Roboto in the directory pointed to
  by `PDF_HELPER_FONTS_DIR`, then an `assets/fonts` folder next to the compiled
  binary, and finally the crate's own `assets/fonts` directory. When unavailable,
  it falls back to the Windows Arial family if present.
* **Image assets** – Examples generate PNG bytes at runtime, but production
  callers typically point `ImageSource::from_path` to files shipped alongside the
  binary. Confirm that relative paths resolve correctly in release builds and
  bundle assets accordingly.
* **Page sizing and margins** – `PdfBuilder` exposes `.with_paper_size`,
  `.with_margins`, and other layout toggles that can be composed before calling
  `render`. Leverage these hooks to align with corporate templates or printer
  requirements.
* **Styling APIs** – Inline styling relies on `Span` helpers for bold, italic,
  underline, colour, and link decoration. Combine these with structured blocks
  (`RichParagraph`, `ImageBlock`, manual page breaks) to keep content expressive
  yet serialisation-friendly.
* **Embedding in larger applications** – Share the `PdfBuilder` configuration
  across services or CLIs by returning it from helper functions (as shown in the
  examples) and deferring the final call to `render` or
  `render_with_bookmarks`. This keeps asset loading, header/footer branding, and
  layout rules centralised.
