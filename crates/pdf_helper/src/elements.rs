//! Extended element implementations built on top of `genpdf` primitives.
//!
//! This module adds convenience wrappers for rendering images with captions, helpers for decoding
//! image data, and custom text elements that the upstream crate does not ship with.

use std::path::Path;

use image::GenericImageView;

use genpdf::elements::{Image, Paragraph};
use genpdf::error::{Context as _, Error};
use genpdf::style::{Style, StyledString};
use genpdf::{render, Alignment, Element, Mm, Position, RenderResult, Scale, Size};

use crate::richtext::StyledSpan;

const DEFAULT_IMAGE_DPI: f64 = 300.0;
const MM_PER_INCH: f64 = 25.4;
const DEFAULT_CAPTION_SPACING_MM: f64 = 2.0;
const DEFAULT_UNDERLINE_OFFSET_MM: f64 = 0.4;

fn mm_from_f64(value: f64) -> Mm {
    Mm::from(printpdf::Mm(value))
}

fn mm_to_f64(value: Mm) -> f64 {
    let mm: printpdf::Mm = value.into();
    mm.0
}

fn estimated_image_size(image: &image::DynamicImage, dpi: f64) -> Size {
    let (px_width, px_height) = image.dimensions();
    let width_mm = MM_PER_INCH * (px_width as f64) / dpi;
    let height_mm = MM_PER_INCH * (px_height as f64) / dpi;
    Size::new(mm_from_f64(width_mm), mm_from_f64(height_mm))
}

/// Loads an image from in-memory bytes using the [`image`] crate with descriptive errors.
pub fn decode_image_from_bytes(bytes: impl AsRef<[u8]>) -> Result<image::DynamicImage, Error> {
    image::load_from_memory(bytes.as_ref()).context("Failed to decode image from provided bytes")
}

/// Loads an image from the given path using the [`image`] crate with descriptive errors.
pub fn decode_image_from_path(path: impl AsRef<Path>) -> Result<image::DynamicImage, Error> {
    let path = path.as_ref();
    let reader = image::io::Reader::open(path)
        .with_context(|| format!("Failed to open image file {}", path.display()))?;
    reader
        .with_guessed_format()
        .context("Unable to determine image format")?
        .decode()
        .with_context(|| format!("Failed to decode image file {}", path.display()))
}

fn image_from_dynamic(image: image::DynamicImage) -> Result<(Image, Size), Error> {
    let size = estimated_image_size(&image, DEFAULT_IMAGE_DPI);
    let image = Image::from_dynamic_image(image)?;
    Ok((image, size))
}

/// Converts the provided image bytes into a `genpdf` image together with its estimated size.
pub fn image_from_bytes(bytes: impl AsRef<[u8]>) -> Result<(Image, Size), Error> {
    let dynamic = decode_image_from_bytes(bytes)?;
    image_from_dynamic(dynamic)
}

/// Converts the image at `path` into a `genpdf` image together with its estimated size.
pub fn image_from_path(path: impl AsRef<Path>) -> Result<(Image, Size), Error> {
    let dynamic = decode_image_from_path(path)?;
    image_from_dynamic(dynamic)
}

fn default_caption_spacing() -> Mm {
    mm_from_f64(DEFAULT_CAPTION_SPACING_MM)
}

fn default_underline_offset() -> Mm {
    mm_from_f64(DEFAULT_UNDERLINE_OFFSET_MM)
}

/// A convenience element that renders an image with an optional caption stacked underneath.
///
/// The image and the caption share the same alignment and the image can be rescaled to a specific
/// width while keeping the aspect ratio.  The element supports creating the image from raw bytes or
/// file paths, delegating the decoding to the [`image`] crate to provide friendly error messages.
pub struct CaptionedImage {
    image: Image,
    caption: Paragraph,
    alignment: Alignment,
    natural_size: Size,
    requested_width: Option<Mm>,
    spacing: Mm,
}

impl CaptionedImage {
    fn new(image: Image, caption: Paragraph, natural_size: Size) -> Self {
        let mut element = Self {
            image,
            caption,
            alignment: Alignment::Left,
            natural_size,
            requested_width: None,
            spacing: default_caption_spacing(),
        };
        element.apply_alignment();
        element
    }

    /// Creates a captioned image from an existing [`DynamicImage`][image::DynamicImage].
    pub fn from_dynamic_image(
        image: image::DynamicImage,
        caption: Paragraph,
    ) -> Result<Self, Error> {
        let (image, size) = image_from_dynamic(image)?;
        Ok(Self::new(image, caption, size))
    }

    /// Creates a captioned image from the contents of `bytes`.
    pub fn from_bytes(bytes: impl AsRef<[u8]>, caption: Paragraph) -> Result<Self, Error> {
        let (image, size) = image_from_bytes(bytes)?;
        Ok(Self::new(image, caption, size))
    }

    /// Creates a captioned image from the file located at `path`.
    pub fn from_path(path: impl AsRef<Path>, caption: Paragraph) -> Result<Self, Error> {
        let (image, size) = image_from_path(path)?;
        Ok(Self::new(image, caption, size))
    }

    /// Returns a mutable reference to the caption paragraph for additional customization.
    pub fn caption_mut(&mut self) -> &mut Paragraph {
        &mut self.caption
    }

    /// Returns a mutable reference to the underlying image for additional configuration.
    pub fn image_mut(&mut self) -> &mut Image {
        &mut self.image
    }

    /// Sets the horizontal alignment used by both the image and the caption.
    pub fn set_alignment(&mut self, alignment: Alignment) {
        self.alignment = alignment;
        self.apply_alignment();
    }

    /// Sets the horizontal alignment and returns the updated element.
    pub fn with_alignment(mut self, alignment: Alignment) -> Self {
        self.set_alignment(alignment);
        self
    }

    /// Sets the spacing between the image and the caption.
    pub fn set_spacing(&mut self, spacing: Mm) {
        self.spacing = spacing;
    }

    /// Sets the spacing and returns the updated element.
    pub fn with_spacing(mut self, spacing: Mm) -> Self {
        self.set_spacing(spacing);
        self
    }

    /// Constrains the rendered width of the image while preserving the aspect ratio.
    pub fn set_width(&mut self, width: Option<Mm>) {
        self.requested_width = width;
        self.apply_width();
    }

    /// Constrains the rendered width and returns the updated element.
    pub fn with_width(mut self, width: impl Into<Option<Mm>>) -> Self {
        self.set_width(width.into());
        self
    }

    fn apply_alignment(&mut self) {
        self.image.set_alignment(self.alignment);
        self.caption.set_alignment(self.alignment);
    }

    fn apply_width(&mut self) {
        if let Some(width) = self.requested_width {
            let natural = mm_to_f64(self.natural_size.width);
            if natural > f64::EPSILON {
                let desired = mm_to_f64(width);
                let scale = desired / natural;
                self.image.set_scale(Scale::new(scale, scale));
            }
        } else {
            self.image.set_scale(Scale::new(1.0, 1.0));
        }
    }
}

impl Element for CaptionedImage {
    fn render(
        &mut self,
        context: &genpdf::Context,
        mut area: render::Area<'_>,
        style: Style,
    ) -> Result<RenderResult, Error> {
        self.apply_alignment();
        self.apply_width();

        let mut result = RenderResult::default();
        let image_result = self.image.render(context, area.clone(), style)?;
        result.size = result.size.stack_vertical(image_result.size);
        result.has_more |= image_result.has_more;

        let spacing = self.spacing;
        area.add_offset(Position::new(0, image_result.size.height + spacing));
        if mm_to_f64(spacing) > 0.0 {
            result.size = result.size.stack_vertical(Size::new(0, spacing));
        }

        let caption_result = self.caption.render(context, area, style)?;
        result.size = result.size.stack_vertical(caption_result.size);
        result.has_more |= caption_result.has_more;

        Ok(result)
    }
}

/// A single line of styled text that supports underlines by drawing thin strokes underneath.
pub struct UnderlinedText {
    spans: Vec<StyledSpan>,
    alignment: Alignment,
    underline_offset: Mm,
}

impl UnderlinedText {
    /// Creates a new underlined text element from the provided spans.
    pub fn new(spans: Vec<StyledSpan>) -> Self {
        Self {
            spans,
            alignment: Alignment::Left,
            underline_offset: default_underline_offset(),
        }
    }

    /// Builds the element from any iterator over spans.
    pub fn from_spans<I>(spans: I) -> Self
    where
        I: IntoIterator<Item = StyledSpan>,
    {
        Self::new(spans.into_iter().collect())
    }

    /// Sets the alignment for the rendered line.
    pub fn set_alignment(&mut self, alignment: Alignment) {
        self.alignment = alignment;
    }

    /// Sets the alignment and returns the updated element.
    pub fn with_alignment(mut self, alignment: Alignment) -> Self {
        self.set_alignment(alignment);
        self
    }

    /// Sets the distance between the baseline and the underline stroke.
    pub fn set_underline_offset(&mut self, offset: Mm) {
        self.underline_offset = offset;
    }

    /// Sets the underline offset and returns the updated element.
    pub fn with_underline_offset(mut self, offset: Mm) -> Self {
        self.set_underline_offset(offset);
        self
    }
}

impl Element for UnderlinedText {
    fn render(
        &mut self,
        context: &genpdf::Context,
        mut area: render::Area<'_>,
        style: Style,
    ) -> Result<RenderResult, Error> {
        let mut prepared: Vec<(StyledString, bool, Mm)> = Vec::with_capacity(self.spans.len());
        let mut total_width = Mm::default();
        let mut max_line_height = style.line_height(&context.font_cache);
        let mut max_glyph_height = Mm::default();

        for span in &self.spans {
            let mut string = span.string.clone();
            string.style = style.and(string.style);
            let width = string.width(&context.font_cache);
            total_width += width;
            max_line_height = max_line_height.max(string.style.line_height(&context.font_cache));
            let glyph_height = string
                .style
                .font(&context.font_cache)
                .glyph_height(string.style.font_size());
            max_glyph_height = max_glyph_height.max(glyph_height);
            prepared.push((string, span.underline, width));
        }

        let available_width = area.size().width;
        let x_offset = match self.alignment {
            Alignment::Left => Mm::default(),
            Alignment::Center => (available_width - total_width) / 2.0,
            Alignment::Right => available_width - total_width,
        };

        if max_line_height > area.size().height {
            let mut result = RenderResult::default();
            result.has_more = true;
            return Ok(result);
        }

        let mut result = RenderResult::default();

        if let Some(mut section) =
            area.text_section(&context.font_cache, Position::new(x_offset, 0), style)
        {
            for (string, _, _) in &prepared {
                section.print_str(&string.s, string.style)?;
            }
        } else {
            result.has_more = true;
            return Ok(result);
        }

        let baseline = max_glyph_height + self.underline_offset;
        let mut cursor = x_offset;
        for (string, underline, width) in &prepared {
            if *underline {
                let mut line_style = Style::new();
                if let Some(color) = string.style.color().or(style.color()) {
                    line_style = line_style.with_color(color);
                }
                area.draw_line(
                    vec![
                        Position::new(cursor, baseline),
                        Position::new(cursor + *width, baseline),
                    ],
                    line_style,
                );
            }
            cursor += *width;
        }

        result.size = Size::new(total_width, max_line_height);
        area.add_offset(Position::new(0, max_line_height));

        Ok(result)
    }
}

impl<I> From<I> for UnderlinedText
where
    I: IntoIterator<Item = StyledSpan>,
{
    fn from(iter: I) -> Self {
        Self::from_spans(iter)
    }
}
