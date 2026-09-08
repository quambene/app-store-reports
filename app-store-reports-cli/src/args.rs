use app_store_reports::{Period, RegionCode};
use clap::Parser;
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(
    name = "app-store-reports",
    version,
    about = "Download App Store Connect financial reports"
)]
pub struct Args {
    /// First period to fetch, format YYYY-MM.
    #[arg(long)]
    pub start: Period,

    /// Last period to fetch, format YYYY-MM. Defaults to --start.
    #[arg(long)]
    pub end: Option<Period>,

    /// Comma-separated Apple financial report region codes, e.g. US,EU,JP,WW,CA.
    /// Must be omitted when --detailed is set.
    #[arg(long, value_delimiter = ',', required_unless_present = "detailed")]
    pub regions: Vec<RegionCode>,

    /// Fetch Apple's transaction-level "All Countries or Regions (Detailed)"
    /// report instead of the aggregated per-region report. Each row is one
    /// individual transaction with exact Transaction Date/Settlement Date
    /// columns. Covers all regions in a single request under Apple's special
    /// Z1 region code, so --regions must be omitted.
    #[arg(long, action = clap::ArgAction::SetTrue, conflicts_with = "regions")]
    pub detailed: bool,

    /// Your Apple vendor number.
    #[arg(long, env = "VENDOR_NUMBER")]
    pub vendor_number: String,

    /// App Store Connect API Issuer ID.
    #[arg(long, env = "ISSUER_ID")]
    pub issuer_id: String,

    /// App Store Connect API Key ID.
    #[arg(long, env = "KEY_ID")]
    pub key_id: String,

    /// Path to the .p8 private key file downloaded from App Store Connect.
    #[arg(long, env = "PRIVATE_KEY_PATH", value_name = "FILE")]
    pub key_path: PathBuf,

    /// Directory to save downloaded reports into.
    #[arg(long, default_value = "./reports")]
    pub output_dir: PathBuf,

    /// Skip saving a decompressed .txt alongside the raw .a.gz file.
    #[arg(long, action = clap::ArgAction::SetTrue)]
    pub no_decompress: bool,
}

impl Args {
    pub fn decompress_enabled(&self) -> bool {
        !self.no_decompress
    }

    pub fn end_or_start(&self) -> Period {
        self.end.unwrap_or(self.start)
    }

    /// Regions to actually request: `Z1` (Apple's "all regions" code) when
    /// `--detailed` is set, otherwise `--regions` as given.
    pub fn effective_regions(&self) -> Vec<RegionCode> {
        if self.detailed {
            vec!["Z1".parse().expect("\"Z1\" is a valid RegionCode")]
        } else {
            self.regions.clone()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    fn parse(args: &[&str]) -> Result<Args, clap::Error> {
        let mut full = vec!["app-store-reports"];
        full.extend_from_slice(args);
        Args::try_parse_from(full)
    }

    fn required_flags() -> Vec<&'static str> {
        vec![
            "--start",
            "2025-01",
            "--regions",
            "US",
            "--vendor-number",
            "123",
            "--issuer-id",
            "issuer",
            "--key-id",
            "key",
            "--key-path",
            "key.p8",
        ]
    }

    #[test]
    fn parses_minimal_required_args() {
        let args = parse(&required_flags()).unwrap();
        assert_eq!(args.regions, vec!["US".parse::<RegionCode>().unwrap()]);
        assert!(args.end.is_none());
        assert_eq!(args.end_or_start(), args.start);
        assert!(args.decompress_enabled());
        assert_eq!(args.output_dir, PathBuf::from("./reports"));
    }

    #[test]
    fn missing_required_field_errors() {
        assert!(parse(&["--start", "2025-01"]).is_err());
    }

    #[test]
    fn regions_are_split_trimmed_and_uppercased() {
        let mut flags = required_flags();
        let regions_index = flags.iter().position(|f| *f == "US").unwrap();
        flags[regions_index] = "US, eu ,jp";
        let args = parse(&flags).unwrap();
        assert_eq!(
            args.regions,
            vec![
                "US".parse::<RegionCode>().unwrap(),
                "EU".parse::<RegionCode>().unwrap(),
                "JP".parse::<RegionCode>().unwrap(),
            ]
        );
    }

    #[test]
    fn no_decompress_flag_toggles_off() {
        let mut flags = required_flags();
        flags.push("--no-decompress");
        let args = parse(&flags).unwrap();
        assert!(!args.decompress_enabled());
    }

    #[test]
    fn end_defaults_to_start_when_omitted() {
        let args = parse(&required_flags()).unwrap();
        assert_eq!(args.end_or_start(), "2025-01".parse::<Period>().unwrap());
    }

    #[test]
    fn end_overrides_start_when_given() {
        let mut flags = required_flags();
        flags.push("--end");
        flags.push("2025-03");
        let args = parse(&flags).unwrap();
        assert_eq!(args.end_or_start(), "2025-03".parse::<Period>().unwrap());
    }

    #[test]
    fn detailed_flag_makes_regions_optional() {
        let flags: Vec<&str> = vec![
            "--start",
            "2025-01",
            "--detailed",
            "--vendor-number",
            "123",
            "--issuer-id",
            "issuer",
            "--key-id",
            "key",
            "--key-path",
            "key.p8",
        ];
        let args = parse(&flags).unwrap();
        assert!(args.detailed);
        assert_eq!(
            args.effective_regions(),
            vec!["Z1".parse::<RegionCode>().unwrap()]
        );
    }

    #[test]
    fn detailed_conflicts_with_regions() {
        let mut flags = required_flags();
        flags.push("--detailed");
        assert!(parse(&flags).is_err());
    }

    #[test]
    fn without_detailed_effective_regions_matches_regions() {
        let args = parse(&required_flags()).unwrap();
        assert_eq!(args.effective_regions(), args.regions);
    }
}
