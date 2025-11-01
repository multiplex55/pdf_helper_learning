//! High-level helpers for assembling richly formatted PDF documents with `genpdf`.
//!
//! The crate exposes a serialisation-friendly content model (see [`model`]) together with
//! builders that translate the logical description into a fully configured [`genpdf::Document`].
//! [`PdfBuilder`](crate::builder::PdfBuilder) wires covers, sections, tables of contents, and
//! optional bookmarks into a deterministic render pipeline that produces final PDF bytes and
//! section metadata.
//!
//! # Examples
//!
//! ```no_run
//! use pdf_helper_learning::builder::PdfBuilder;
//! use pdf_helper_learning::model::{Block, Cover, Section};
//! use pdf_helper_learning::richtext::Span;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let cover = Cover::new("Sample Report").with_block(Block::paragraph(vec![
//!     Span::new("Prepared by ").italic(),
//!     Span::new("pdf_helper_learning").bold(),
//! ]));
//!
//! let section = Section::new("Highlights").with_block(Block::paragraph(vec![
//!     Span::new("The builder ensures consistent output across runs.")
//! ]));
//!
//! let pdf = PdfBuilder::new()
//!     .show_header(true)
//!     .show_footer(true)
//!     .with_cover(cover)
//!     .add_section(section)
//!     .render()?;
//!
//! std::fs::write("report.pdf", &pdf.bytes)?;
//! # Ok(())
//! # }
//! ```
//!
//! Enable the `bookmarks` feature to post-process rendered bytes with hierarchical outlines via
//! [`PdfBuilder::render_with_bookmarks`](crate::builder::PdfBuilder::render_with_bookmarks).
//! Enabling the `hyphenation` feature wires an embedded US-English dictionary into the generated
//! document to improve paragraph flow.
//!
//! ## Fonts
//!
//! The helper APIs look for the Roboto font family under `assets/fonts`.  The repository keeps that
//! directory but omits the actual `.ttf` files; add the regular, bold, italic, and bold italic
//! variants before running the examples or integration tests.

pub mod builder;
pub mod elements;
pub mod fonts;
pub mod model;
pub mod richtext;

#[cfg(feature = "bookmarks")]
pub mod bookmarks;
