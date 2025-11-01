//! Document construction helpers for the pdf_helper_learning crate.

use crate::fonts;
use genpdf::error::{Error, ErrorKind};
use genpdf::style;
use genpdf::{self, Element, Margins, Mm, PageDecorator, Position, Size};

#[cfg(feature = "hyphenation")]
use genpdf::hyphenation;

/// Builder for `genpdf::Document` instances pre-configured with the crate defaults.
#[derive(Default)]
pub struct DocumentBuilder {
    paper_size: Option<Size>,
    margins: Option<Margins>,
    header: Option<Box<HeaderFactory>>,
    footer: Option<FooterSpec>,
    #[cfg(feature = "hyphenation")]
    hyphenator: Option<hyphenation::Standard>,
}

type HeaderFactory = dyn Fn(usize) -> Box<dyn Element>;

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
    pub fn with_hyphenator(mut self, hyphenator: hyphenation::Standard) -> Self {
        self.hyphenator = Some(hyphenator);
        self
    }

    /// Builds a fully configured `genpdf::Document` instance.
    pub fn build(self) -> Result<genpdf::Document, Error> {
        let font_family = fonts::default_font_family()?;
        let mut document = genpdf::Document::new(font_family);

        if let Some(paper_size) = self.paper_size {
            document.set_paper_size(paper_size);
        }

        let decorator = ConfiguredPageDecorator::new(self.margins, self.header, self.footer);
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
    factory: Box<HeaderFactory>,
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
}

impl ConfiguredPageDecorator {
    fn new(
        margins: Option<Margins>,
        header: Option<Box<HeaderFactory>>,
        footer: Option<FooterSpec>,
    ) -> Self {
        Self {
            page: 0,
            margins,
            header,
            footer,
        }
    }
}

impl PageDecorator for ConfiguredPageDecorator {
    fn decorate_page<'a>(
        &mut self,
        context: &genpdf::Context,
        mut area: genpdf::render::Area<'a>,
        style: style::Style,
    ) -> Result<genpdf::render::Area<'a>, Error> {
        self.page += 1;

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
