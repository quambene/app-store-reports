use crate::Period;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("failed to read private key file {}: {source}", path.display())]
    ReadPrivateKey {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("{} is not a valid EC private key: {source}", path.display())]
    InvalidPrivateKey {
        path: PathBuf,
        #[source]
        source: jsonwebtoken::errors::Error,
    },

    #[error("failed to sign JWT: {0}")]
    SignToken(#[source] jsonwebtoken::errors::Error),

    #[error("request to App Store Connect failed: {0}")]
    Request(#[from] reqwest::Error),

    /// Apple returned 404: no report exists for this region/period. This is
    /// an expected outcome (not every region has activity every month), not
    /// a hard failure.
    #[error("no financial report available for this region/period")]
    NotFound,

    #[error(
        "App Store Connect API error (HTTP {status}){}",
        detail.as_deref().map(|d| format!(": {d}")).unwrap_or_default()
    )]
    Api {
        status: reqwest::StatusCode,
        detail: Option<String>,
    },

    #[error("failed to decompress report: {0}")]
    Decompress(#[source] std::io::Error),

    #[error("invalid period '{0}', expected YYYY-MM")]
    InvalidPeriod(String),

    #[error("region code must not be empty")]
    EmptyRegion,

    #[error("end period {end} must not be before start period {start}")]
    InvalidRange { start: Period, end: Period },
}

pub type Result<T> = std::result::Result<T, Error>;
