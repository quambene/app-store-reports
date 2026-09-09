# App Store Reports

A Rust library and CLI for downloading App Store Connect [Sales and Finance](https://developer.apple.com/documentation/appstoreconnectapi/sales-and-finance)
financial reports, so you don't have to click through App Store Connect once per month per region.

- `app-store-reports`: the library. Handles JWT auth, the `financeReports` request, and gzip decompression.
- `app-store-reports-cli`: a CLI (binary name `app-store-reports`) built on top of the library.

---

- [Requirements](#requirements)
- [Install CLI](#install-cli)
- [CLI usage](#cli-usage)
- [Library usage](#library-usage)

## Requirements

1. In App Store Connect, go to "Users and Access > Integrations > App Store Connect API" and
   create an API key with the "Finance" (or Admin) role. Download the `.p8` private key file
   (Apple only lets you download it once) and note the Issuer ID and Key ID.
2. Find your vendor number on the [Payments and Financial Reports](https://appstoreconnect.apple.com/trends)
   page in App Store Connect.
3. Create `.env` with `ISSUER_ID`, `KEY_ID`, `PRIVATE_KEY_PATH` (path to the
   `.p8` file), and `VENDOR_NUMBER`.

## Install CLI

``` bash
git clone git@github.com:quambene/app-store-reports.git
cd app-store-reports

# Build and install app-store-reports binary to ~/.cargo/bin
cargo install --path ./app-store-reports-cli
```

## CLI usage

```sh
# Single month, one region
app-store-reports --start 2026-01 --regions US

# Range of months, multiple regions
app-store-reports --start 2025-01 --end 2025-12 --regions US,EU,JP,WW,CA

# Detailed report including all regions
app-store-reports --start 2025-01 --end 2025-12 --detailed
```

Reports are saved to `./reports/` (override with `--output-dir`) as both the raw
`<period>_<region>_financial_report.a.gz` file Apple returns and a decompressed
`.txt` file for opening directly in Excel/pandas. Pass `--no-decompress` to skip
the extraction step.

Not every region has a report for every month — those are skipped with a `skip` message
rather than treated as failures.

## Library usage

```rust
let credentials = app_store_reports::Credentials::load(issuer_id, key_id, key_path)?;
let client = app_store_reports::Client::new(credentials)?;
let request = app_store_reports::FinanceReportRequest {
    vendor_number: vendor_number.to_string(),
    region: "US".parse()?,
    period: "2025-01".parse()?,
};

match client.fetch_report(&request) {
    Ok(gzip_bytes) => { /* write, or app_store_reports::decompress(&gzip_bytes)? */ }
    Err(app_store_reports::Error::NotFound) => { /* no report for this region/period */ }
    Err(err) => { /* real error */ }
}
```

## Changelog

The `app-store-reports` repository contains multiple crates with separate changelogs:

- `app-store-reports`: [view changelog](https://github.com/quambene/app-store-reports/blob/main/app-store-reports/CHANGELOG.md)
- `app-store-reports-cli`: [view changelog](https://github.com/quambene/app-store-reports/blob/main/app-store-reports-cli/CHANGELOG.md)
