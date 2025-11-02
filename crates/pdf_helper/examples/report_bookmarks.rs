#[cfg(feature = "bookmarks")]
use std::error::Error;

#[cfg(feature = "bookmarks")]
fn main() -> Result<(), Box<dyn Error>> {
    pdf_helper::examples::report_bookmarks::run()
}

#[cfg(not(feature = "bookmarks"))]
fn main() {
    eprintln!(
        "Enable the `bookmarks` feature to run this example:\n    cargo run --example report_bookmarks --features bookmarks"
    );
}
