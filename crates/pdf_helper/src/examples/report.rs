use std::error::Error;

use super::shared;

/// Renders the standard sample report to `report.pdf` using the default output path.
///
/// The generated document relies on font assets located under `assets/fonts`. See
/// [`crate::fonts`] for the search order and environment variables that influence it.
pub fn run() -> Result<(), Box<dyn Error>> {
    let builder = shared::build_sample_report_builder()?;
    let pdf = builder.render()?;
    std::fs::write("report.pdf", &pdf.bytes)?;
    println!("Generated report.pdf ({} bytes)", pdf.bytes.len());
    Ok(())
}
