//! Data structures describing the logical content of a PDF document.
//!
//! The types in this module form a serialization-friendly model that mirrors the
//! building blocks expected by `genpdf`.  They intentionally avoid referencing
//! the rendering crate directly so the values can be produced by frontends,
//! persisted, or exchanged over the network without pulling in heavy
//! dependencies.

use crate::richtext::Span;

/// Metadata that controls how textual and visual elements are aligned once
/// they are converted into [`genpdf::elements`].
///
/// The variants map directly to [`genpdf::Alignment`] and are stored as a small
/// enum so that serialized representations stay compact and easy to
/// interoperate with.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum HorizontalAlignment {
    /// Left aligned content.
    #[default]
    Left,
    /// Center aligned content.
    Center,
    /// Right aligned content.
    Right,
    /// Fully justified paragraphs.
    Justified,
}

/// Rich text paragraph carrying inline styling information and alignment
/// metadata.
///
/// The paragraph stores a vector of [`Span`] values, which already capture
/// inline decorations such as bold, italic, underline and custom colors.  When
/// the paragraph is eventually rendered the alignment can be mapped to
/// [`genpdf::Alignment`] while the spans are turned into styled strings via the
/// helpers in [`crate::richtext`].
#[derive(Clone, Debug, Default, PartialEq)]
pub struct RichParagraph {
    spans: Vec<Span>,
    alignment: HorizontalAlignment,
}

impl RichParagraph {
    /// Creates a paragraph from the provided spans using left alignment.
    pub fn new(spans: impl Into<Vec<Span>>) -> Self {
        Self {
            spans: spans.into(),
            ..Self::default()
        }
    }

    /// Returns the spans that make up the paragraph.
    pub fn spans(&self) -> &[Span] {
        &self.spans
    }

    /// Returns the configured alignment.
    pub fn alignment(&self) -> HorizontalAlignment {
        self.alignment
    }

    /// Sets the alignment and returns the updated paragraph.
    pub fn with_alignment(mut self, alignment: HorizontalAlignment) -> Self {
        self.alignment = alignment;
        self
    }
}

/// Representation of image sources supported by the content model.
#[derive(Clone, Debug, PartialEq)]
pub enum ImageSource {
    /// Image loaded from raw bytes.
    Bytes(Vec<u8>),
    /// Image referenced by a file path.
    Path(String),
}

impl ImageSource {
    /// Creates a new in-memory image from raw bytes.
    pub fn from_bytes(bytes: impl Into<Vec<u8>>) -> Self {
        Self::Bytes(bytes.into())
    }

    /// Creates an image sourced from a file path.
    pub fn from_path(path: impl Into<String>) -> Self {
        Self::Path(path.into())
    }
}

/// Additional metadata for image blocks.
///
/// The width is stored as millimetres to make it straightforward to map into
/// the [`genpdf::elements::Image`] scaling API.  The alignment and caption reuse
/// the same primitives as text paragraphs, allowing callers to build captions
/// with the same styling affordances.
#[derive(Clone, Debug, PartialEq)]
pub struct ImageBlock {
    source: ImageSource,
    caption: Option<RichParagraph>,
    alignment: HorizontalAlignment,
    width_mm: Option<f64>,
}

impl ImageBlock {
    /// Creates a new image block using the provided source.
    pub fn new(source: ImageSource) -> Self {
        Self {
            source,
            caption: None,
            alignment: HorizontalAlignment::Left,
            width_mm: None,
        }
    }

    /// Returns the image source.
    pub fn source(&self) -> &ImageSource {
        &self.source
    }

    /// Returns the caption paragraph, if any.
    pub fn caption(&self) -> Option<&RichParagraph> {
        self.caption.as_ref()
    }

    /// Returns the configured alignment.
    pub fn alignment(&self) -> HorizontalAlignment {
        self.alignment
    }

    /// Returns the requested rendered width in millimetres, if any.
    pub fn width_mm(&self) -> Option<f64> {
        self.width_mm
    }

    /// Sets the caption and returns the updated image block.
    pub fn with_caption(mut self, caption: impl Into<Option<RichParagraph>>) -> Self {
        self.caption = caption.into();
        self
    }

    /// Sets the alignment and returns the updated image block.
    pub fn with_alignment(mut self, alignment: HorizontalAlignment) -> Self {
        self.alignment = alignment;
        self
    }

    /// Constrains the rendered width (in millimetres) and returns the updated block.
    pub fn with_width_mm(mut self, width_mm: impl Into<Option<f64>>) -> Self {
        self.width_mm = width_mm.into();
        self
    }
}

/// Individual content blocks that make up sections and the cover.
#[derive(Clone, Debug, PartialEq)]
pub enum Block {
    /// Styled paragraph content.
    Paragraph(RichParagraph),
    /// Captioned image content.
    Image(ImageBlock),
    /// Explicit page break request.
    PageBreak,
}

impl Block {
    /// Convenience helper for building a paragraph block.
    pub fn paragraph(spans: impl Into<Vec<Span>>) -> Self {
        Self::Paragraph(RichParagraph::new(spans))
    }

    /// Convenience helper for building an image block.
    pub fn image(source: ImageSource) -> Self {
        Self::Image(ImageBlock::new(source))
    }

    /// Convenience helper that yields an explicit page break block.
    pub fn page_break() -> Self {
        Self::PageBreak
    }
}

/// Metadata that describes the cover page of a document.
///
/// The cover stores a title and optional identifier/summary blocks.  Blocks can
/// mix paragraphs, images with captions, and explicit page breaks to provide a
/// flexible layout while remaining easy to serialize.
#[derive(Clone, Debug, PartialEq)]
pub struct Cover {
    title: String,
    subtitle: Option<String>,
    identifier: Option<String>,
    blocks: Vec<Block>,
}

impl Cover {
    /// Creates a new cover with the given title.
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            subtitle: None,
            identifier: None,
            blocks: Vec::new(),
        }
    }

    /// Returns the title shown on the cover page.
    pub fn title(&self) -> &str {
        &self.title
    }

    /// Returns the subtitle, if any.
    pub fn subtitle(&self) -> Option<&str> {
        self.subtitle.as_deref()
    }

    /// Returns the identifier, if any.
    pub fn identifier(&self) -> Option<&str> {
        self.identifier.as_deref()
    }

    /// Returns the content blocks rendered on the cover page.
    pub fn blocks(&self) -> &[Block] {
        &self.blocks
    }

    /// Sets the subtitle and returns the updated cover.
    pub fn with_subtitle(mut self, subtitle: impl Into<Option<String>>) -> Self {
        self.subtitle = subtitle.into();
        self
    }

    /// Sets the identifier and returns the updated cover.
    pub fn with_identifier(mut self, identifier: impl Into<Option<String>>) -> Self {
        self.identifier = identifier.into();
        self
    }

    /// Appends a block to the cover and returns the updated instance.
    pub fn with_block(mut self, block: Block) -> Self {
        self.blocks.push(block);
        self
    }

    /// Extends the cover with multiple blocks and returns the updated instance.
    pub fn with_blocks<I>(mut self, blocks: I) -> Self
    where
        I: IntoIterator<Item = Block>,
    {
        self.blocks.extend(blocks);
        self
    }
}

/// Logical representation of a document section.
#[derive(Clone, Debug, PartialEq)]
pub struct Section {
    identifier: Option<String>,
    title: String,
    blocks: Vec<Block>,
}

impl Section {
    /// Creates a new section with the provided title.
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            identifier: None,
            title: title.into(),
            blocks: Vec::new(),
        }
    }

    /// Returns the section identifier used for bookmarks or cross references.
    pub fn identifier(&self) -> Option<&str> {
        self.identifier.as_deref()
    }

    /// Returns the title of the section.
    pub fn title(&self) -> &str {
        &self.title
    }

    /// Returns the blocks contained in the section.
    pub fn blocks(&self) -> &[Block] {
        &self.blocks
    }

    /// Sets the identifier and returns the updated section.
    pub fn with_identifier(mut self, identifier: impl Into<Option<String>>) -> Self {
        self.identifier = identifier.into();
        self
    }

    /// Appends a block and returns the updated section.
    pub fn with_block(mut self, block: Block) -> Self {
        self.blocks.push(block);
        self
    }

    /// Extends the section with additional blocks and returns the updated instance.
    pub fn with_blocks<I>(mut self, blocks: I) -> Self
    where
        I: IntoIterator<Item = Block>,
    {
        self.blocks.extend(blocks);
        self
    }

    /// Creates a builder that can inject an initial page break.
    pub fn builder(title: impl Into<String>) -> SectionBuilder {
        SectionBuilder::new(title)
    }
}

/// Builder for [`Section`] values.
///
/// The builder is intentionally minimal so it can be used with serde-driven
/// deserialization.  Callers can opt-in to inserting a page break at the
/// beginning of the section via [`SectionBuilder::start_on_new_page`].
#[derive(Clone, Debug, Default)]
pub struct SectionBuilder {
    identifier: Option<String>,
    title: String,
    blocks: Vec<Block>,
    start_on_new_page: bool,
}

impl SectionBuilder {
    /// Creates a builder for a section with the given title.
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            ..Self::default()
        }
    }

    /// Marks the section to start on a new page.
    pub fn start_on_new_page(mut self, start_on_new_page: bool) -> Self {
        self.start_on_new_page = start_on_new_page;
        self
    }

    /// Sets the identifier for the section.
    pub fn identifier(mut self, identifier: impl Into<Option<String>>) -> Self {
        self.identifier = identifier.into();
        self
    }

    /// Pushes an additional block into the section.
    pub fn push_block(mut self, block: Block) -> Self {
        self.blocks.push(block);
        self
    }

    /// Extends the builder with multiple blocks.
    pub fn extend_blocks<I>(mut self, blocks: I) -> Self
    where
        I: IntoIterator<Item = Block>,
    {
        self.blocks.extend(blocks);
        self
    }

    /// Builds the final section, injecting a leading page break when requested.
    pub fn build(mut self) -> Section {
        if self.start_on_new_page {
            match self.blocks.first() {
                Some(Block::PageBreak) => {}
                _ => self.blocks.insert(0, Block::PageBreak),
            }
        }

        let mut section = Section::new(self.title);
        section.identifier = self.identifier;
        section.blocks = self.blocks;
        section
    }
}

#[cfg(test)]
mod tests {
    use super::{Block, Section};

    #[test]
    fn builder_inserts_page_break() {
        let section = Section::builder("Intro")
            .start_on_new_page(true)
            .push_block(Block::paragraph(Vec::new()))
            .build();

        assert!(matches!(section.blocks().first(), Some(Block::PageBreak)));
    }

    #[test]
    fn builder_does_not_duplicate_page_break() {
        let section = Section::builder("Intro")
            .start_on_new_page(true)
            .push_block(Block::PageBreak)
            .build();

        assert!(matches!(section.blocks().first(), Some(Block::PageBreak)));
        assert_eq!(section.blocks().len(), 1);
    }
}
