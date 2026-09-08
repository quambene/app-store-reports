use app_store_reports::{Period, RegionCode};
use std::path::{Path, PathBuf};

pub fn output_paths(
    dir: &Path,
    period: Period,
    region: &RegionCode,
    detailed: bool,
) -> (PathBuf, PathBuf) {
    let kind = if detailed {
        "financial_detail_report"
    } else {
        "financial_report"
    };
    let base = format!("{period}_{region}_{kind}");
    (
        dir.join(format!("{base}.a.gz")),
        dir.join(format!("{base}.txt")),
    )
}

pub fn summary_line(downloaded: u32, skipped: u32, failed: u32) -> String {
    format!("downloaded={downloaded} skipped={skipped} failed={failed}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_expected_filenames() {
        let period = "2025-01".parse::<Period>().unwrap();
        let region = "US".parse::<RegionCode>().unwrap();
        let (gz, txt) = output_paths(Path::new("./reports"), period, &region, false);
        assert_eq!(
            gz,
            PathBuf::from("./reports/2025-01_US_financial_report.a.gz")
        );
        assert_eq!(
            txt,
            PathBuf::from("./reports/2025-01_US_financial_report.txt")
        );
    }

    #[test]
    fn builds_expected_filenames_for_detailed_report() {
        let period = "2025-01".parse::<Period>().unwrap();
        let region = "Z1".parse::<RegionCode>().unwrap();
        let (gz, txt) = output_paths(Path::new("./reports"), period, &region, true);
        assert_eq!(
            gz,
            PathBuf::from("./reports/2025-01_Z1_financial_detail_report.a.gz")
        );
        assert_eq!(
            txt,
            PathBuf::from("./reports/2025-01_Z1_financial_detail_report.txt")
        );
    }

    #[test]
    fn formats_summary_line() {
        assert_eq!(summary_line(3, 2, 1), "downloaded=3 skipped=2 failed=1");
    }
}
