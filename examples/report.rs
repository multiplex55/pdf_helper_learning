#[path = "shared/report_util.rs"]
mod report_util;

use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let builder = report_util::build_sample_report_builder()?;
    let pdf = builder.render()?;
    std::fs::write("report.pdf", &pdf.bytes)?;
    println!("Generated report.pdf ({} bytes)", pdf.bytes.len());
    Ok(())
}
