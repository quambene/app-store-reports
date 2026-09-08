//! Download App Store Connect Sales and Finance reports.
//!
//! See <https://developer.apple.com/documentation/appstoreconnectapi/sales-and-finance>.
//!
//! ```no_run
//! # fn main() -> Result<(), app_store_reports::Error> {
//! let credentials = app_store_reports::Credentials::load("issuer-id", "key-id", "AuthKey.p8")?;
//! let client = app_store_reports::Client::new(credentials)?;
//! let request = app_store_reports::FinanceReportRequest {
//!     vendor_number: "12345678".to_string(),
//!     region: "US".parse()?,
//!     period: "2025-01".parse()?,
//!     detailed: false,
//! };
//! match client.fetch_report(&request) {
//!     Ok(gzip_bytes) => {
//!         let report = app_store_reports::decompress(&gzip_bytes)?;
//!         println!("{} bytes", report.len());
//!     }
//!     Err(app_store_reports::Error::NotFound) => println!("no report for this region/period"),
//!     Err(err) => return Err(err),
//! }
//! # Ok(())
//! # }
//! ```

mod auth;
mod client;
mod error;
mod period;
mod region;

pub use auth::Credentials;
pub use client::{Client, FinanceReportRequest, decompress};
pub use error::{Error, Result};
pub use period::{Period, PeriodRange};
pub use region::RegionCode;
