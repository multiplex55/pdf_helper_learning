//! Font loading utilities for the pdf_helper_learning crate.

use genpdf::error::Error;
use genpdf::fonts::{FontData, FontFamily};
use genpdf::Document;

/// Name of the bundled font family.
pub const DEFAULT_FONT_FAMILY_NAME: &str = "Roboto";

const ROBOTO_REGULAR: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/assets/fonts/Roboto-Regular.ttf"
));
const ROBOTO_BOLD: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/assets/fonts/Roboto-Bold.ttf"
));
const ROBOTO_ITALIC: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/assets/fonts/Roboto-Italic.ttf"
));
const ROBOTO_BOLD_ITALIC: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/assets/fonts/Roboto-BoldItalic.ttf"
));

fn load_font(data: &[u8]) -> Result<FontData, Error> {
    FontData::new(data.to_vec(), None)
}

/// Returns the bundled Roboto font family as a `genpdf` font family definition.
pub fn default_font_family() -> Result<FontFamily<FontData>, Error> {
    Ok(FontFamily {
        regular: load_font(ROBOTO_REGULAR)?,
        bold: load_font(ROBOTO_BOLD)?,
        italic: load_font(ROBOTO_ITALIC)?,
        bold_italic: load_font(ROBOTO_BOLD_ITALIC)?,
    })
}

/// Adds the bundled Roboto font family to the given document and returns the cached fonts.
pub fn install_default_fonts(
    document: &mut Document,
) -> Result<FontFamily<genpdf::fonts::Font>, Error> {
    let family = default_font_family()?;
    Ok(document.add_font_family(family))
}
