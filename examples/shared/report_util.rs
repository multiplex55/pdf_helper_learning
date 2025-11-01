use std::error::Error;
use std::io::Cursor;

use genpdf::style::Color;
use image::{DynamicImage, ImageBuffer, ImageOutputFormat, Rgb};
use pdf_helper_learning::builder::PdfBuilder;
use pdf_helper_learning::model::{
    Block, Cover, HorizontalAlignment, ImageBlock, ImageSource, RichParagraph, Section,
};
use pdf_helper_learning::richtext::Span;

pub fn build_sample_report_builder() -> Result<PdfBuilder, Box<dyn Error>> {
    let hero_image = ImageBlock::new(ImageSource::from_bytes(generate_placeholder_image()?))
        .with_caption(Some(RichParagraph::new(vec![
            Span::new("Figure 1: ").bold(),
            Span::new("Generated gradient rendered through CaptionedImage."),
        ])))
        .with_alignment(HorizontalAlignment::Center)
        .with_width_mm(Some(120.0));

    let cover = Cover::new("Engineering Highlights")
        .with_subtitle(Some("Spring Edition".to_string()))
        .with_block(Block::paragraph(vec![
            Span::new("Prepared for the ").italic(),
            Span::new("Architecture Guild").bold(),
        ]))
        .with_block(Block::paragraph(vec![Span::new(
            "The data points and imagery below are generated for demonstration purposes.",
        )]));

    let summary = Section::new("Executive Summary")
        .with_block(Block::paragraph(vec![Span::new(
            "This multi-section PDF is produced with pdf_helper_learning.",
        )]))
        .with_block(Block::paragraph(vec![
            Span::new("Paragraphs can mix styles like "),
            Span::new("bold").bold(),
            Span::new(", "),
            Span::new("italic").italic(),
            Span::new(" and "),
            Span::new("colour accents").colored(Color::Rgb(200, 90, 140)),
            Span::new(" within the same block."),
        ]));

    let visuals = Section::new("Visual Walkthrough")
        .with_block(Block::paragraph(vec![Span::new(
            "Captioned images align with surrounding content and support custom widths.",
        )]))
        .with_block(Block::Image(hero_image))
        .with_block(Block::paragraph(vec![Span::new(
            "Sections can also insert explicit page breaks when longer narratives are required.",
        )]))
        .with_block(Block::page_break())
        .with_block(Block::paragraph(vec![Span::new(
            "After a break, additional paragraphs continue the story.",
        )]));

    let builder = PdfBuilder::new()
        .show_header(true)
        .show_footer(true)
        .include_printed_toc(true)
        .with_toc_title(Some("Contents".to_string()))
        .with_cover(cover)
        .add_section(summary)
        .add_section(visuals);

    Ok(builder)
}

fn generate_placeholder_image() -> Result<Vec<u8>, image::ImageError> {
    let width: u32 = 240;
    let height: u32 = 140;
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
        let r = (xf * 155.0 + 80.0).round().clamp(0.0, 255.0) as u8;
        let g = (yf * 120.0 + 60.0).round().clamp(0.0, 255.0) as u8;
        let b = ((xf + yf) * 100.0 + 55.0).round().clamp(0.0, 255.0) as u8;
        Rgb([r, g, b])
    });

    let mut bytes = Vec::new();
    DynamicImage::ImageRgb8(buffer)
        .write_to(&mut Cursor::new(&mut bytes), ImageOutputFormat::Png)?;
    Ok(bytes)
}
