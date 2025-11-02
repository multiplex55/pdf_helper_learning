//! Font loading utilities for the `pdf_helper` crate.

use std::env;
use std::io;
use std::path::{Path, PathBuf};

use genpdf::error::{Error, ErrorKind};
use genpdf::fonts::{self, FontData, FontFamily};
use genpdf::Document;
use log::warn;

/// Name of the bundled font family.
pub const DEFAULT_FONT_FAMILY_NAME: &str = "Roboto";

const FONT_FILES: &[&str] = &[
    "Roboto-Regular.ttf",
    "Roboto-Bold.ttf",
    "Roboto-Italic.ttf",
    "Roboto-BoldItalic.ttf",
];

const WINDOWS_FALLBACK_FAMILY_NAME: &str = "Arial";

struct WindowsFontFiles {
    regular: &'static str,
    bold: &'static str,
    italic: &'static str,
    bold_italic: &'static str,
}

const WINDOWS_FONT_FILES: WindowsFontFiles = WindowsFontFiles {
    regular: "arial.ttf",
    bold: "arialbd.ttf",
    italic: "ariali.ttf",
    bold_italic: "arialbi.ttf",
};

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

fn load_bundled_font_family() -> Result<FontFamily<FontData>, Error> {
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

fn env_path(var: &str) -> Option<PathBuf> {
    env::var_os(var).and_then(|value| {
        let path = PathBuf::from(value);
        if path.as_os_str().is_empty() {
            None
        } else {
            Some(path)
        }
    })
}

fn windows_font_directory() -> Option<PathBuf> {
    if let Some(path) = env_path("PDF_HELPER_WINDOWS_FONTS_DIR") {
        return Some(path);
    }

    #[cfg(windows)]
    {
        for var in ["WINDIR", "SystemRoot"] {
            if let Some(root) = env_path(var) {
                let candidate = root.join("Fonts");
                if candidate.is_dir() {
                    return Some(candidate);
                }
            }
        }
    }

    None
}

fn load_windows_font(directory: &Path, file: &str, style: &str) -> Result<FontData, Error> {
    let path = directory.join(file);
    FontData::load(&path, None).map_err(|err| {
        let io_kind = if path.is_file() {
            io::ErrorKind::Other
        } else {
            io::ErrorKind::NotFound
        };
        Error::new(
            format!(
                "Failed to load Windows fallback {} font at {}: {}",
                style,
                path.display(),
                err
            ),
            io::Error::new(io_kind, err.to_string()),
        )
    })
}

fn windows_fallback_font_family() -> Result<FontFamily<FontData>, Error> {
    let directory = windows_font_directory().ok_or_else(|| {
        Error::new(
            "Windows font directory not found for fallback",
            io::Error::new(io::ErrorKind::NotFound, "windows fonts directory not found"),
        )
    })?;

    Ok(FontFamily {
        regular: load_windows_font(&directory, WINDOWS_FONT_FILES.regular, "regular")?,
        bold: load_windows_font(&directory, WINDOWS_FONT_FILES.bold, "bold")?,
        italic: load_windows_font(&directory, WINDOWS_FONT_FILES.italic, "italic")?,
        bold_italic: load_windows_font(&directory, WINDOWS_FONT_FILES.bold_italic, "bold italic")?,
    })
}

fn fonts_missing(err: &Error) -> bool {
    matches!(
        err.kind(),
        ErrorKind::IoError(io_err)
            if io_err.kind() == io::ErrorKind::NotFound
                || io_err.kind() == io::ErrorKind::PermissionDenied
    )
}

/// Returns the bundled Roboto font family if available and falls back to the Windows Arial family
/// when the bundled fonts are missing.
pub fn default_font_family() -> Result<FontFamily<FontData>, Error> {
    match load_bundled_font_family() {
        Ok(family) => Ok(family),
        Err(err) if fonts_missing(&err) => match windows_fallback_font_family() {
            Ok(fallback) => {
                warn!(
                    "Bundled fonts unavailable ({}); falling back to Windows '{}' family.",
                    err, WINDOWS_FALLBACK_FAMILY_NAME
                );
                Ok(fallback)
            }
            Err(fallback_err) => {
                warn!(
                    "Bundled fonts unavailable ({}); Windows fallback failed: {}",
                    err, fallback_err
                );
                Err(Error::new(
                    format!(
                        "Bundled fonts unavailable and Windows fallback failed: {}",
                        fallback_err
                    ),
                    io::Error::new(io::ErrorKind::NotFound, "default fonts are not available"),
                ))
            }
        },
        Err(err) => Err(err),
    }
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
