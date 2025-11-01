//! Font loading utilities for the pdf_helper_learning crate.

use std::io;
use std::path::{Path, PathBuf};

use genpdf::error::Error;
use genpdf::fonts::{self, FontData, FontFamily};
use genpdf::Document;

/// Name of the bundled font family.
pub const DEFAULT_FONT_FAMILY_NAME: &str = "Roboto";

const FONT_FILES: &[&str] = &[
    "Roboto-Regular.ttf",
    "Roboto-Bold.ttf",
    "Roboto-Italic.ttf",
    "Roboto-BoldItalic.ttf",
];

fn bundled_font_directory() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets/fonts")
}

fn ensure_directory_exists(path: &Path) -> Result<(), Error> {
    if path.exists() {
        Ok(())
    } else {
        Err(Error::new(
            format!(
                "Bundled font directory missing at {}. See assets/fonts/README.md for setup.",
                path.display()
            ),
            io::Error::new(io::ErrorKind::NotFound, "bundled fonts directory not found"),
        ))
    }
}

fn ensure_required_fonts_present(path: &Path) -> Result<(), Error> {
    let missing: Vec<_> = FONT_FILES
        .iter()
        .map(|name| path.join(name))
        .filter(|candidate| !candidate.is_file())
        .collect();

    if missing.is_empty() {
        Ok(())
    } else {
        let display_list = missing
            .iter()
            .map(|path| path.display().to_string())
            .collect::<Vec<_>>()
            .join(", ");

        Err(Error::new(
            format!(
                "Missing bundled font files: {}. See assets/fonts/README.md for instructions.",
                display_list
            ),
            io::Error::new(io::ErrorKind::NotFound, "bundled fonts missing"),
        ))
    }
}

/// Returns the bundled Roboto font family as a `genpdf` font family definition.
pub fn default_font_family() -> Result<FontFamily<FontData>, Error> {
    let directory = bundled_font_directory();
    ensure_directory_exists(&directory)?;
    ensure_required_fonts_present(&directory)?;

    fonts::from_files(&directory, DEFAULT_FONT_FAMILY_NAME, None).map_err(|err| {
        Error::new(
            format!(
                "Failed to load default font family '{}' from {}: {}",
                DEFAULT_FONT_FAMILY_NAME,
                directory.display(),
                err
            ),
            io::Error::new(io::ErrorKind::Other, err.to_string()),
        )
    })
}

/// Adds the bundled Roboto font family to the given document and returns the cached fonts.
pub fn install_default_fonts(
    document: &mut Document,
) -> Result<FontFamily<genpdf::fonts::Font>, Error> {
    let family = default_font_family()?;
    Ok(document.add_font_family(family))
}

/// Indicates whether all bundled fonts required for the default font family are present on disk.
pub fn default_fonts_available() -> bool {
    let directory = bundled_font_directory();
    directory.exists()
        && FONT_FILES
            .iter()
            .map(|name| directory.join(name))
            .all(|path| path.is_file())
}
