pub mod error;
pub mod report;

// Re-export the most commonly used types so callers can write
// `use zarqaa_types::Verdict` instead of `use zarqaa_types::report::Verdict`
pub use error::{Result, ZarqaaError};
pub use report::{ChainAddress, LegReport, RouteReport, Verdict};
