pub mod error;
pub mod report;

// Re-export the most commonly used types so callers can write
// `use zarqa_types::Verdict` instead of `use zarqa_types::report::Verdict`
pub use error::{Result, ZarqaError};
pub use report::{ChainAddress, LegReport, RouteReport, Verdict};
