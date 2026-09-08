use crate::{auth::Credentials, error::Error, period::Period, region::RegionCode};
use reqwest::StatusCode;
use serde::Deserialize;
use std::time::Duration;

const BASE_URL: &str = "https://api.appstoreconnect.apple.com/v1/financeReports";

/// One (vendor, region, month) financial report to fetch. Apple returns one
/// region's report per request, not a single country's.
#[derive(Debug, Clone)]
pub struct FinanceReportRequest {
    pub vendor_number: String,
    pub region: RegionCode,
    pub period: Period,
}

/// Wraps a reused HTTP client and a set of credentials.
pub struct Client {
    http: reqwest::blocking::Client,
    credentials: Credentials,
}

impl Client {
    pub fn new(credentials: Credentials) -> Result<Self, Error> {
        let http = reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(60))
            .build()?;
        Ok(Self { http, credentials })
    }

    /// Returns raw gzip bytes on success. `Err(Error::NotFound)` means Apple
    /// has no report for this region/period — a normal, expected outcome (not
    /// every region has activity every month).
    pub fn fetch_report(&self, request: &FinanceReportRequest) -> Result<Vec<u8>, Error> {
        let token = self.credentials.generate_token()?;
        let response = self
            .http
            .get(BASE_URL)
            .bearer_auth(token)
            .header(reqwest::header::ACCEPT, "application/a-gzip")
            .query(&[
                ("filter[reportType]", "FINANCIAL"),
                ("filter[regionCode]", request.region.as_ref()),
                ("filter[reportDate]", request.period.to_string().as_str()),
                ("filter[vendorNumber]", request.vendor_number.as_str()),
            ])
            .send()?;
        let status = response.status();
        let body = response.bytes()?.to_vec();

        interpret_response(status, body)
    }
}

pub(crate) fn interpret_response(status: StatusCode, body: Vec<u8>) -> Result<Vec<u8>, Error> {
    if status.is_success() {
        return Ok(body);
    }

    if status == StatusCode::NOT_FOUND {
        return Err(Error::NotFound);
    }

    Err(Error::Api {
        status,
        detail: parse_error_body(&body),
    })
}

#[derive(Deserialize)]
struct ErrorBody {
    errors: Vec<ApiErrorEntry>,
}

#[derive(Deserialize)]
struct ApiErrorEntry {
    code: Option<String>,
    title: Option<String>,
    detail: Option<String>,
}

/// Parses Apple's JSON:API `{"errors":[{code,title,detail}]}` error shape,
/// falling back to raw body text if it isn't valid JSON in that shape.
fn parse_error_body(body: &[u8]) -> Option<String> {
    if let Ok(parsed) = serde_json::from_slice::<ErrorBody>(body) {
        let messages: Vec<String> = parsed
            .errors
            .iter()
            .map(|entry| {
                let code = entry.code.as_deref().unwrap_or("");
                let message = entry
                    .detail
                    .as_deref()
                    .or(entry.title.as_deref())
                    .map(|d| format!(": {d}"))
                    .unwrap_or_default();
                format!("{code}{message}")
            })
            .collect();
        if !messages.is_empty() {
            return Some(messages.join("; "));
        }
    }
    let text = String::from_utf8_lossy(body).trim().to_string();
    (!text.is_empty()).then_some(text)
}

/// Gunzip a raw report body, as returned by [`Client::fetch_report`].
pub fn decompress(gzip_bytes: &[u8]) -> Result<Vec<u8>, Error> {
    use std::io::Read;

    let mut out = Vec::new();
    flate2::read::GzDecoder::new(gzip_bytes)
        .read_to_end(&mut out)
        .map_err(Error::Decompress)?;
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn success_status_returns_body() {
        let result = interpret_response(StatusCode::OK, b"gzip-bytes".to_vec());
        assert_eq!(result.unwrap(), b"gzip-bytes");
    }

    #[test]
    fn not_found_maps_to_not_found_error() {
        let result = interpret_response(StatusCode::NOT_FOUND, b"anything".to_vec());
        assert!(matches!(result, Err(Error::NotFound)));
    }

    #[test]
    fn error_status_parses_apple_json_api_error_shape() {
        // Captured live from a real 401 response while validating the request shape.
        let body = br#"{
            "errors": [{
                "status": "401",
                "code": "NOT_AUTHORIZED",
                "title": "Authentication credentials are missing or invalid.",
                "detail": "Provide a properly configured and signed bearer token, and make sure that it has not expired."
            }]
        }"#;
        let result = interpret_response(StatusCode::UNAUTHORIZED, body.to_vec());
        match result {
            Err(Error::Api { status, detail }) => {
                assert_eq!(status, StatusCode::UNAUTHORIZED);
                let detail = detail.unwrap();
                assert!(detail.contains("NOT_AUTHORIZED"));
                assert!(detail.contains("properly configured and signed bearer token"));
            }
            other => panic!("expected Api error, got {other:?}"),
        }
    }

    #[test]
    fn error_status_falls_back_to_raw_text_for_non_json_body() {
        let result = interpret_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            b"upstream timeout".to_vec(),
        );
        match result {
            Err(Error::Api { status, detail }) => {
                assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
                assert_eq!(detail.as_deref(), Some("upstream timeout"));
            }
            other => panic!("expected Api error, got {other:?}"),
        }
    }

    #[test]
    fn error_status_with_empty_body_has_no_detail() {
        let result = interpret_response(StatusCode::INTERNAL_SERVER_ERROR, Vec::new());
        match result {
            Err(Error::Api { detail, .. }) => assert_eq!(detail, None),
            other => panic!("expected Api error, got {other:?}"),
        }
    }

    #[test]
    fn decompress_round_trips_gzip_data() {
        let mut encoder = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
        encoder.write_all(b"hello financial report").unwrap();
        let gzipped = encoder.finish().unwrap();

        let decompressed = decompress(&gzipped).unwrap();
        assert_eq!(decompressed, b"hello financial report");
    }

    #[test]
    fn decompress_rejects_non_gzip_input() {
        let result = decompress(b"not gzip data");
        assert!(matches!(result, Err(Error::Decompress(_))));
    }
}
