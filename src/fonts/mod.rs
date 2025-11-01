//! Font loading utilities for the pdf_helper_learning crate.

use std::env;
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

fn font_directory_candidates() -> Vec<PathBuf> {
    let mut candidates = Vec::new();

    if let Ok(path) = env::var("PDF_HELPER_FONTS_DIR") {
        if !path.trim().is_empty() {
            candidates.push(PathBuf::from(path));
        }
    }

    if let Ok(current_exe) = env::current_exe() {
        if let Some(bin_dir) = current_exe.parent() {
            let candidate = bin_dir.join("assets/fonts");
            if !candidates.iter().any(|existing| existing == &candidate) {
                candidates.push(candidate);
            }
        }
    }

    let manifest_candidate = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets/fonts");
    if !candidates
        .iter()
        .any(|existing| existing == &manifest_candidate)
    {
        candidates.push(manifest_candidate);
    }

    candidates
}

fn missing_font_files(path: &Path) -> Vec<PathBuf> {
    FONT_FILES
        .iter()
        .map(|name| path.join(name))
        .filter(|candidate| !candidate.is_file())
        .collect()
}

fn resolve_font_directory() -> Result<PathBuf, Error> {
    let mut attempts = Vec::new();

    for candidate in font_directory_candidates() {
        let exists = candidate.is_dir();
        let missing = missing_font_files(&candidate);

        if exists && missing.is_empty() {
            return Ok(candidate);
        }

        let reason = if !exists {
            format!("directory missing at {}", candidate.display())
        } else {
            let missing_list = missing
                .iter()
                .map(|path| path.file_name().unwrap_or_default().to_string_lossy())
                .collect::<Vec<_>>()
                .join(", ");
            format!("missing files [{}]", missing_list)
        };

        attempts.push(format!("{} ({})", candidate.display(), reason));
    }

    let summary = if attempts.is_empty() {
        "no search paths were available".to_owned()
    } else {
        attempts.join(", ")
    };

    Err(Error::new(
        format!(
            "Unable to locate bundled font directory. Checked: {}. See assets/fonts/README.md or set PDF_HELPER_FONTS_DIR.",
            summary
        ),
        io::Error::new(io::ErrorKind::NotFound, "bundled fonts directory not found"),
    ))
}

/// Returns the bundled Roboto font family as a `genpdf` font family definition.
pub fn default_font_family() -> Result<FontFamily<FontData>, Error> {
    let directory = resolve_font_directory()?;

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
    resolve_font_directory().is_ok()
}
