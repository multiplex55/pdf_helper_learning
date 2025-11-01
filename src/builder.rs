//! Document construction helpers for the pdf_helper_learning crate.

use std::cell::RefCell;
use std::fmt;
use std::rc::Rc;

use crate::elements::CaptionedImage;
use crate::fonts;
use crate::model::{
    Block, Cover, HorizontalAlignment, ImageBlock, ImageSource, RichParagraph, Section,
};
use genpdf::elements::{Break as LineBreak, PageBreak, Paragraph, TableLayout};
use genpdf::error::{Error, ErrorKind};
use genpdf::style::{Style, StyledString};
use genpdf::{self, Alignment, Element, Margins, Mm, PageDecorator, Position, Size};

#[cfg(feature = "hyphenation")]
use hyphenation::{Language, Load as _, Standard as HyphenationStandard};

/// Tracks the page numbers observed during a render pass so that section metadata can be derived.
#[derive(Debug, Default)]
struct PageTracker {
    current_page: usize,
    section_pages: Vec<Option<usize>>,
}

type PageTrackerHandle = Rc<RefCell<PageTracker>>;

impl PageTracker {
    fn new(section_count: usize) -> Self {
        Self {
            current_page: 0,
            section_pages: vec![None; section_count],
        }
    }

    fn set_current_page(&mut self, page: usize) {
        self.current_page = page;
    }

    fn mark_section(&mut self, index: usize) {
        if let Some(slot) = self.section_pages.get_mut(index) {
            if slot.is_none() {
                *slot = Some(self.current_page);
            }
        }
    }

    fn pages(&self) -> &[Option<usize>] {
        &self.section_pages
    }
}

/// Builder for `genpdf::Document` instances pre-configured with the crate defaults.
#[derive(Default)]
pub struct DocumentBuilder {
    paper_size: Option<Size>,
    margins: Option<Margins>,
    header: Option<Box<HeaderFactory>>,
    footer: Option<FooterSpec>,
    page_tracker: Option<PageTrackerHandle>,
    #[cfg(feature = "hyphenation")]
    hyphenator: Option<HyphenationStandard>,
}

type HeaderFactory = dyn Fn(usize) -> Box<dyn Element>;

type FooterFactory = dyn Fn(usize) -> Box<dyn Element>;

impl DocumentBuilder {
    /// Creates a new builder instance with default settings.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the paper size used for newly created documents.
    pub fn with_paper_size(mut self, paper_size: impl Into<Size>) -> Self {
        self.paper_size = Some(paper_size.into());
        self
    }

    /// Sets the margins applied through the page decorator.
    pub fn with_margins(mut self, margins: impl Into<Margins>) -> Self {
        self.margins = Some(margins.into());
        self
    }

    /// Configures a header callback that is invoked for every page.
    pub fn with_header<F, E>(mut self, header: F) -> Self
    where
        F: Fn(usize) -> E + 'static,
        E: Element + 'static,
    {
        self.header = Some(Box::new(move |page| {
            Box::new(header(page)) as Box<dyn Element>
        }));
        self
    }

    /// Configures a footer callback with a fixed height that is invoked for every page.
    pub fn with_footer<F, E>(mut self, height: impl Into<Mm>, footer: F) -> Self
    where
        F: Fn(usize) -> E + 'static,
        E: Element + 'static,
    {
        self.footer = Some(FooterSpec::new(height, footer));
        self
    }

    /// Enables hyphenation using the provided hyphenation dictionary.
    #[cfg(feature = "hyphenation")]
    pub fn with_hyphenator(mut self, hyphenator: HyphenationStandard) -> Self {
        self.hyphenator = Some(hyphenator);
        self
    }

    /// Installs a page tracker that is notified whenever a new page is decorated.
    fn with_page_tracker(mut self, tracker: PageTrackerHandle) -> Self {
        self.page_tracker = Some(tracker);
        self
    }

    /// Builds a fully configured `genpdf::Document` instance.
    pub fn build(self) -> Result<genpdf::Document, Error> {
        let font_family = fonts::default_font_family()?;
        let mut document = genpdf::Document::new(font_family);

        if let Some(paper_size) = self.paper_size {
            document.set_paper_size(paper_size);
        }

        let decorator =
            ConfiguredPageDecorator::new(self.margins, self.header, self.footer, self.page_tracker);
        document.set_page_decorator(decorator);

        #[cfg(feature = "hyphenation")]
        if let Some(hyphenator) = self.hyphenator {
            document.set_hyphenator(hyphenator);
        }

        Ok(document)
    }
}

/// Definition of a footer rendered through the page decorator.
pub struct FooterSpec {
    height: Mm,
    factory: Box<FooterFactory>,
}

impl FooterSpec {
    /// Creates a new footer specification.
    pub fn new<F, E>(height: impl Into<Mm>, factory: F) -> Self
    where
        F: Fn(usize) -> E + 'static,
        E: Element + 'static,
    {
        Self {
            height: height.into(),
            factory: Box::new(move |page| Box::new(factory(page)) as Box<dyn Element>),
        }
    }
}

struct ConfiguredPageDecorator {
    page: usize,
    margins: Option<Margins>,
    header: Option<Box<HeaderFactory>>,
    footer: Option<FooterSpec>,
    tracker: Option<PageTrackerHandle>,
}

impl ConfiguredPageDecorator {
    fn new(
        margins: Option<Margins>,
        header: Option<Box<HeaderFactory>>,
        footer: Option<FooterSpec>,
        tracker: Option<PageTrackerHandle>,
    ) -> Self {
        Self {
            page: 0,
            margins,
            header,
            footer,
            tracker,
        }
    }
}

impl PageDecorator for ConfiguredPageDecorator {
    fn decorate_page<'a>(
        &mut self,
        context: &genpdf::Context,
        mut area: genpdf::render::Area<'a>,
        style: Style,
    ) -> Result<genpdf::render::Area<'a>, Error> {
        self.page += 1;

        if let Some(tracker) = &self.tracker {
            tracker.borrow_mut().set_current_page(self.page);
        }

        if let Some(margins) = self.margins {
            area.add_margins(margins);
        }

        if let Some(header_cb) = &self.header {
            let mut element = header_cb(self.page);
            let result = element.render(context, area.clone(), style)?;
            area.add_offset(Position::new(0, result.size.height));
        }

        if let Some(footer) = &self.footer {
            let available = area.size().height;
            if footer.height > available {
                return Err(Error::new(
                    "Footer height exceeds available space",
                    ErrorKind::InvalidData,
                ));
            }

            let mut footer_area = area.clone();
            footer_area.add_offset(Position::new(0, available - footer.height));
            let mut element = (footer.factory)(self.page);
            let result = element.render(context, footer_area, style)?;
            if result.has_more {
                return Err(Error::new(
                    "Footer element does not fit into the reserved space",
                    ErrorKind::PageSizeExceeded,
                ));
            }

            area.set_height(available - footer.height);
        }

        Ok(area)
    }
}

/// Captures the page on which a section starts when inserted at the beginning of the section.
struct SectionMarker {
    tracker: PageTrackerHandle,
    index: usize,
    recorded: bool,
}

impl SectionMarker {
    fn new(tracker: PageTrackerHandle, index: usize) -> Self {
        Self {
            tracker,
            index,
            recorded: false,
        }
    }
}

impl Element for SectionMarker {
    fn render(
        &mut self,
        _context: &genpdf::Context,
        _area: genpdf::render::Area<'_>,
        _style: Style,
    ) -> Result<genpdf::RenderResult, Error> {
        if !self.recorded {
            self.tracker.borrow_mut().mark_section(self.index);
            self.recorded = true;
        }
        Ok(genpdf::RenderResult::default())
    }
}

/// Errors produced while preparing or rendering a PDF document.
#[derive(Debug)]
pub enum PdfBuildError {
    /// Failure while loading the bundled fonts into the document context.
    FontLoad(Error),
    /// Failure while converting model blocks into renderable elements.
    Content { message: String, source: Error },
    /// Failure reported by `genpdf` when rendering the final document.
    Render(Error),
    /// Hyphenation was requested but no dictionary could be loaded.
    HyphenationUnavailable { language: &'static str },
    /// Hyphenation dictionary failed to load from the embedded resources.
    #[cfg(feature = "hyphenation")]
    HyphenationLoad {
        language: &'static str,
        source: hyphenation::LoadError,
    },
}

impl PdfBuildError {
    fn content(message: impl Into<String>, source: Error) -> Self {
        Self::Content {
            message: message.into(),
            source,
        }
    }
}

impl fmt::Display for PdfBuildError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FontLoad(err) => write!(f, "Failed to load fonts: {}", err),
            Self::Content { message, .. } => write!(f, "{}", message),
            Self::Render(err) => write!(f, "Failed to render PDF: {}", err),
            Self::HyphenationUnavailable { language } => write!(
                f,
                "Hyphenation requested for language {} but the feature is not available",
                language
            ),
            #[cfg(feature = "hyphenation")]
            Self::HyphenationLoad { language, .. } => {
                write!(f, "Failed to load hyphenation dictionary for {}", language)
            }
        }
    }
}

impl std::error::Error for PdfBuildError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::FontLoad(err) | Self::Render(err) => Some(err),
            Self::Content { source, .. } => Some(source),
            Self::HyphenationUnavailable { .. } => None,
            #[cfg(feature = "hyphenation")]
            Self::HyphenationLoad { source, .. } => Some(source),
        }
    }
}

/// Result of a PDF render operation.
#[derive(Debug, Default)]
pub struct PdfRenderResult {
    /// Bytes containing the rendered PDF document.
    pub bytes: Vec<u8>,
    /// Recorded start page (1-indexed) for each section in the order provided to the builder.
    pub section_start_pages: Vec<Option<usize>>,
}

/// Builder responsible for turning [`Cover`] and [`Section`] definitions into rendered PDFs.
#[derive(Debug)]
pub struct PdfBuilder {
    paper_size: Option<Size>,
    margins: Option<Margins>,
    show_header: bool,
    show_footer: bool,
    enable_hyphenation: bool,
    cover: Option<Cover>,
    sections: Vec<Section>,
    include_toc: bool,
    toc_title: Option<String>,
    default_alignment: HorizontalAlignment,
    render_section_headings: bool,
    collect_section_pages: bool,
}

impl Default for PdfBuilder {
    fn default() -> Self {
        Self {
            paper_size: None,
            margins: None,
            show_header: false,
            show_footer: false,
            enable_hyphenation: false,
            cover: None,
            sections: Vec::new(),
            include_toc: false,
            toc_title: None,
            default_alignment: HorizontalAlignment::Left,
            render_section_headings: true,
            collect_section_pages: false,
        }
    }
}

impl PdfBuilder {
    /// Creates a new builder instance with default settings.
    pub fn new() -> Self {
        Self::default()
    }

    /// Overrides the paper size used for the generated document.
    pub fn with_paper_size(mut self, paper_size: impl Into<Size>) -> Self {
        self.paper_size = Some(paper_size.into());
        self
    }

    /// Overrides the page margins applied to every page.
    pub fn with_margins(mut self, margins: impl Into<Margins>) -> Self {
        self.margins = Some(margins.into());
        self
    }

    /// Controls whether the default header is printed.
    pub fn show_header(mut self, show: bool) -> Self {
        self.show_header = show;
        self
    }

    /// Controls whether the default footer with page numbers is printed.
    pub fn show_footer(mut self, show: bool) -> Self {
        self.show_footer = show;
        self
    }

    /// Toggles hyphenation support using the embedded dictionary.
    pub fn enable_hyphenation(mut self, enable: bool) -> Self {
        self.enable_hyphenation = enable;
        self
    }

    /// Attaches the cover description that should be rendered as the first page.
    pub fn with_cover(mut self, cover: impl Into<Option<Cover>>) -> Self {
        self.cover = cover.into();
        self
    }

    /// Appends an additional section to the document.
    pub fn add_section(mut self, section: Section) -> Self {
        self.sections.push(section);
        self
    }

    /// Replaces the entire section list.
    pub fn with_sections<I>(mut self, sections: I) -> Self
    where
        I: IntoIterator<Item = Section>,
    {
        self.sections = sections.into_iter().collect();
        self
    }

    /// Requests a printed table of contents before the sections are rendered.
    pub fn include_printed_toc(mut self, include: bool) -> Self {
        self.include_toc = include;
        self
    }

    /// Sets the title shown above the generated table of contents.
    pub fn with_toc_title(mut self, title: impl Into<Option<String>>) -> Self {
        self.toc_title = title.into();
        self
    }

    /// Sets the default horizontal alignment applied to paragraphs and images.
    pub fn with_default_alignment(mut self, alignment: HorizontalAlignment) -> Self {
        self.default_alignment = alignment;
        self
    }

    /// Controls whether section headings generated from their titles are rendered automatically.
    pub fn render_section_headings(mut self, enabled: bool) -> Self {
        self.render_section_headings = enabled;
        self
    }

    /// Toggles recording section start pages for the final render output.
    pub fn collect_section_pages(mut self, collect: bool) -> Self {
        self.collect_section_pages = collect;
        self
    }

    /// Renders the configured document and returns both the PDF bytes and section metadata.
    pub fn render(self) -> Result<PdfRenderResult, PdfBuildError> {
        let section_count = self.sections.len();
        let need_toc = self.include_toc && section_count > 0;
        let need_tracking = self.collect_section_pages || need_toc;

        let mut recorded_pages = vec![None; section_count];

        if need_tracking {
            let tracker = Rc::new(RefCell::new(PageTracker::new(section_count)));
            let _ = self.render_internal(Some(Rc::clone(&tracker)), None)?;
            recorded_pages = tracker.borrow().pages().to_vec();
        }

        let final_tracker = if need_tracking && section_count > 0 {
            Some(Rc::new(RefCell::new(PageTracker::new(section_count))))
        } else if self.collect_section_pages && section_count > 0 {
            Some(Rc::new(RefCell::new(PageTracker::new(section_count))))
        } else {
            None
        };

        let bytes = self.render_internal(
            final_tracker.clone(),
            if need_toc {
                Some(&recorded_pages)
            } else {
                None
            },
        )?;

        let section_start_pages = final_tracker
            .map(|tracker| tracker.borrow().pages().to_vec())
            .unwrap_or_else(|| vec![None; section_count]);

        Ok(PdfRenderResult {
            bytes,
            section_start_pages,
        })
    }

    fn render_internal(
        &self,
        tracker: Option<PageTrackerHandle>,
        toc_pages: Option<&[Option<usize>]>,
    ) -> Result<Vec<u8>, PdfBuildError> {
        let mut document = self.prepare_document(tracker.clone())?;
        self.populate_document(&mut document, tracker.as_ref(), toc_pages)?;
        let mut buffer = Vec::new();
        document
            .render(&mut buffer)
            .map_err(PdfBuildError::Render)?;
        Ok(buffer)
    }

    fn prepare_document(
        &self,
        tracker: Option<PageTrackerHandle>,
    ) -> Result<genpdf::Document, PdfBuildError> {
        let mut builder = DocumentBuilder::new();
        if let Some(size) = self.paper_size {
            builder = builder.with_paper_size(size);
        }
        if let Some(margins) = self.margins {
            builder = builder.with_margins(margins);
        }
        if let Some(tracker) = tracker.clone() {
            builder = builder.with_page_tracker(tracker);
        }

        #[cfg(feature = "hyphenation")]
        {
            builder = self.apply_hyphenation(builder)?;
        }
        #[cfg(not(feature = "hyphenation"))]
        {
            if self.enable_hyphenation {
                return Err(PdfBuildError::HyphenationUnavailable { language: "en-US" });
            }
        }

        if self.show_header {
            if let Some(title) = self.cover.as_ref().map(|cover| cover.title().to_string()) {
                let header_text = title.clone();
                builder = builder.with_header(move |_| {
                    let mut paragraph = Paragraph::new(header_text.clone());
                    paragraph.set_alignment(Alignment::Center);
                    paragraph
                });
            }
        }

        if self.show_footer {
            builder = builder.with_footer(mm_from_f64(12.0), |page| {
                let mut paragraph = Paragraph::new(format!("Page {}", page));
                paragraph.set_alignment(Alignment::Right);
                paragraph
            });
        }

        builder.build().map_err(PdfBuildError::FontLoad)
    }

    #[cfg(feature = "hyphenation")]
    fn apply_hyphenation(
        &self,
        builder: DocumentBuilder,
    ) -> Result<DocumentBuilder, PdfBuildError> {
        if self.enable_hyphenation {
            let hyphenator =
                HyphenationStandard::from_embedded(Language::EnglishUS).map_err(|source| {
                    PdfBuildError::HyphenationLoad {
                        language: "en-US",
                        source,
                    }
                })?;
            Ok(builder.with_hyphenator(hyphenator))
        } else {
            Ok(builder)
        }
    }

    fn populate_document(
        &self,
        document: &mut genpdf::Document,
        tracker: Option<&PageTrackerHandle>,
        toc_pages: Option<&[Option<usize>]>,
    ) -> Result<(), PdfBuildError> {
        if let Some(cover) = &self.cover {
            self.push_cover(document, cover)?;
            if self.include_toc || !self.sections.is_empty() {
                document.push(PageBreak::new());
            }
        }

        if self.include_toc && !self.sections.is_empty() {
            self.push_toc(document, toc_pages)?;
        }

        for (index, section) in self.sections.iter().enumerate() {
            if let Some(tracker) = tracker {
                document.push(SectionMarker::new(Rc::clone(tracker), index));
            }

            if self.render_section_headings {
                self.push_section_heading(document, section);
            }

            self.push_section_blocks(document, section.blocks())?;
        }

        Ok(())
    }

    fn push_cover(
        &self,
        document: &mut genpdf::Document,
        cover: &Cover,
    ) -> Result<(), PdfBuildError> {
        let mut title_style = Style::new();
        title_style.set_font_size(28);
        title_style.set_bold();
        let mut title = Paragraph::new(cover.title());
        title.set_alignment(Alignment::Center);
        document.push(title.styled(title_style));
        document.push(LineBreak::new(1.5));

        if let Some(subtitle) = cover.subtitle() {
            let mut subtitle_style = Style::new();
            subtitle_style.set_font_size(18);
            subtitle_style.set_italic();
            let mut paragraph = Paragraph::new(subtitle);
            paragraph.set_alignment(Alignment::Center);
            document.push(paragraph.styled(subtitle_style));
            document.push(LineBreak::new(1.0));
        }

        if let Some(identifier) = cover.identifier() {
            let mut paragraph = Paragraph::new(identifier);
            paragraph.set_alignment(Alignment::Center);
            document.push(paragraph);
            document.push(LineBreak::new(1.0));
        }

        self.push_section_blocks(document, cover.blocks())
    }

    fn push_toc(
        &self,
        document: &mut genpdf::Document,
        toc_pages: Option<&[Option<usize>]>,
    ) -> Result<(), PdfBuildError> {
        let mut title_style = Style::new();
        title_style.set_font_size(20);
        title_style.set_bold();
        let toc_title = self.toc_title.as_deref().unwrap_or("Table of Contents");
        let mut heading = Paragraph::new(toc_title);
        heading.set_alignment(Alignment::Center);
        document.push(heading.styled(title_style));
        document.push(LineBreak::new(1.0));

        let placeholder;
        let pages = if let Some(pages) = toc_pages {
            pages
        } else {
            placeholder = vec![None; self.sections.len()];
            &placeholder
        };

        let mut table = TableLayout::new(vec![6, 1]);
        for (section, page) in self.sections.iter().zip(pages.iter()) {
            let mut title = Paragraph::new(section.title());
            title.set_alignment(Alignment::Left);
            let mut page_number = Paragraph::new(
                page.map(|value| value.to_string())
                    .unwrap_or_else(|| "--".into()),
            );
            page_number.set_alignment(Alignment::Right);
            table
                .push_row(vec![Box::new(title), Box::new(page_number)])
                .map_err(|err| PdfBuildError::content("Failed to append TOC row", err))?;
        }

        document.push(table);
        document.push(PageBreak::new());
        Ok(())
    }

    fn push_section_heading(&self, document: &mut genpdf::Document, section: &Section) {
        let mut style = Style::new();
        style.set_bold();
        style.set_font_size(18);
        let mut heading = Paragraph::new(section.title());
        heading.set_alignment(self.resolve_alignment(self.default_alignment));
        document.push(heading.styled(style));
        document.push(LineBreak::new(0.75));
    }

    fn push_section_blocks(
        &self,
        document: &mut genpdf::Document,
        blocks: &[Block],
    ) -> Result<(), PdfBuildError> {
        for block in blocks {
            self.push_block(document, block)?;
        }
        Ok(())
    }

    fn push_block(
        &self,
        document: &mut genpdf::Document,
        block: &Block,
    ) -> Result<(), PdfBuildError> {
        match block {
            Block::Paragraph(paragraph) => {
                document.push(self.build_paragraph(paragraph));
            }
            Block::Image(image) => {
                let element = self.build_image(image)?;
                document.push(element);
            }
            Block::PageBreak => {
                document.push(PageBreak::new());
            }
        }
        Ok(())
    }

    fn build_paragraph(&self, paragraph: &RichParagraph) -> Paragraph {
        let mut iter = paragraph.spans().iter();
        let mut element = if let Some(first) = iter.next() {
            Paragraph::new(StyledString::from(first))
        } else {
            Paragraph::new(StyledString::new(String::new(), Style::new()))
        };
        for span in iter {
            element.push(StyledString::from(span));
        }
        element.set_alignment(self.resolve_alignment(paragraph.alignment()));
        element
    }

    fn build_image(&self, block: &ImageBlock) -> Result<CaptionedImage, PdfBuildError> {
        let alignment = self.resolve_alignment(block.alignment());
        let caption_paragraph = block
            .caption()
            .map(|caption| {
                let mut paragraph = self.build_paragraph(caption);
                paragraph.set_alignment(alignment);
                paragraph
            })
            .unwrap_or_else(|| {
                let mut paragraph = Paragraph::new(StyledString::new(String::new(), Style::new()));
                paragraph.set_alignment(alignment);
                paragraph
            });

        let mut element = match block.source() {
            ImageSource::Bytes(bytes) => CaptionedImage::from_bytes(bytes, caption_paragraph)
                .map_err(|err| PdfBuildError::content("Failed to decode image bytes", err))?,
            ImageSource::Path(path) => CaptionedImage::from_path(path, caption_paragraph)
                .map_err(|err| PdfBuildError::content("Failed to load image from path", err))?,
        };

        element.set_alignment(alignment);
        if let Some(width) = block.width_mm() {
            element.set_width(Some(mm_from_f64(width)));
        }
        Ok(element)
    }

    fn resolve_alignment(&self, requested: HorizontalAlignment) -> Alignment {
        self.map_alignment(match requested {
            HorizontalAlignment::Left => self.default_alignment,
            other => other,
        })
    }

    fn map_alignment(&self, alignment: HorizontalAlignment) -> Alignment {
        match alignment {
            HorizontalAlignment::Left | HorizontalAlignment::Justified => Alignment::Left,
            HorizontalAlignment::Center => Alignment::Center,
            HorizontalAlignment::Right => Alignment::Right,
        }
    }
}

fn mm_from_f64(value: f64) -> Mm {
    Mm::from(printpdf::Mm(value))
}
