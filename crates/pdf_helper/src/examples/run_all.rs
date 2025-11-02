use std::error::Error;
use std::fs;
use std::path::Path;

use super::shared;

const OUTPUT_DIR: &str = "target/run_all_examples";

/// Renders all available variants to the `target/run_all_examples` directory.
pub fn run() -> Result<(), Box<dyn Error>> {
    let output_dir = Path::new(OUTPUT_DIR);
    fs::create_dir_all(output_dir)?;

    run_standard_variant(output_dir)?;

    #[cfg(feature = "bookmarks")]
    {
        run_bookmarks_variant(output_dir)?;
    }

    #[cfg(not(feature = "bookmarks"))]
    {
        println!(
            "Skipping bookmarks render. Enable the feature to include it:\n    cargo run --example run_all --features bookmarks"
        );
    }

    println!("All renders completed successfully.");
    Ok(())
}

fn run_standard_variant(output_dir: &Path) -> Result<(), Box<dyn Error>> {
    let builder = shared::build_sample_report_builder()?;
    let pdf = builder.render()?;
    let output_path = output_dir.join("sample_report_standard.pdf");
    fs::write(&output_path, &pdf.bytes)?;
    println!(
        "Generated {} ({} bytes)",
        output_path.display(),
        pdf.bytes.len()
    );
    Ok(())
}

#[cfg(feature = "bookmarks")]
fn run_bookmarks_variant(output_dir: &Path) -> Result<(), Box<dyn Error>> {
    let builder = shared::build_sample_report_builder()?;
    let pdf = builder.render_with_bookmarks()?;
    let output_path = output_dir.join("sample_report_with_bookmarks.pdf");
    fs::write(&output_path, &pdf.bytes)?;
    println!(
        "Generated {} ({} bytes) with section bookmarks",
        output_path.display(),
        pdf.bytes.len()
    );
    Ok(())
}
