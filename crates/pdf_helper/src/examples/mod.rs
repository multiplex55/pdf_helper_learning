//! Example runners mirroring the original binary examples.

pub mod report;
pub mod report_bookmarks;
pub mod run_all;
pub mod shared;

pub use report::run as run_report;
pub use report_bookmarks::run as run_report_bookmarks;
pub use run_all::run as run_all_examples;
