#[path = "shared/report_util.rs"]
mod report_util;

#[cfg(feature = "bookmarks")]
use std::error::Error;

#[cfg(feature = "bookmarks")]
fn main() -> Result<(), Box<dyn Error>> {
    let builder = report_util::build_sample_report_builder()?;
    let pdf = builder.render_with_bookmarks()?;
    std::fs::write("report_with_bookmarks.pdf", &pdf.bytes)?;
    println!(
        "Generated report_with_bookmarks.pdf ({} bytes) with section bookmarks",
        pdf.bytes.len()
    );
    Ok(())
}

#[cfg(not(feature = "bookmarks"))]
fn main() {
    eprintln!(
        "Enable the `bookmarks` feature to run this example: \
         cargo run --example report_bookmarks --features bookmarks"
    );
}
