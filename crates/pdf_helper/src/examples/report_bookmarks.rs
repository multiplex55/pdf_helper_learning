use std::io;

#[cfg(feature = "bookmarks")]
use super::shared;

/// Renders the sample report with generated bookmarks.
#[cfg(feature = "bookmarks")]
pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    let builder = shared::build_sample_report_builder()?;
    let pdf = builder.render_with_bookmarks()?;
    std::fs::write("report_with_bookmarks.pdf", &pdf.bytes)?;
    println!(
        "Generated report_with_bookmarks.pdf ({} bytes) with section bookmarks",
        pdf.bytes.len()
    );
    Ok(())
}

/// Stub implementation used when the `bookmarks` feature is disabled.
#[cfg(not(feature = "bookmarks"))]
pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    Err(io::Error::new(
        io::ErrorKind::Other,
        "Enable the `bookmarks` feature to render bookmarked output",
    )
    .into())
}
