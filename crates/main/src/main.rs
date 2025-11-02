use std::error::Error;

use clap::{Parser, Subcommand};

/// Runs the available pdf_helper examples from the command line.
///
/// Fonts must be present under `assets/fonts` relative to the `pdf_helper` crate
/// or provided via the `PDF_HELPER_FONTS_DIR` environment variable before
/// running the commands below.
#[derive(Parser)]
#[command(author, version, about = "Convenience CLI for pdf_helper examples")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Render the sample report to `report.pdf`.
    #[command(name = "report")]
    Report,

    /// Render the sample report with bookmarks to `report_with_bookmarks.pdf`.
    #[command(name = "report-bookmarks", aliases = ["report_bookmarks", "bookmarks"])]
    ReportBookmarks,

    /// Render all available report variants under `target/run_all_examples`.
    #[command(name = "run-all", aliases = ["run_all", "all"])]
    RunAll,
}

fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Report => pdf_helper::examples::report::run(),
        Commands::ReportBookmarks => pdf_helper::examples::report_bookmarks::run(),
        Commands::RunAll => pdf_helper::examples::run_all::run(),
    };

    if let Err(err) = result {
        eprintln!("Error: {}", err);
        print_error_sources(err.as_ref());
        std::process::exit(1);
    }
}

fn print_error_sources(mut error: &(dyn Error + 'static)) {
    while let Some(source) = error.source() {
        eprintln!("  caused by: {}", source);
        error = source;
    }
}
