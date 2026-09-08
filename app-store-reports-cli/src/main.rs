mod args;
mod output;

use clap::Parser;
use std::process::ExitCode;

fn main() -> ExitCode {
    match try_main() {
        Ok(code) => code,
        Err(err) => {
            eprintln!("Error: {err:#}");
            ExitCode::FAILURE
        }
    }
}

fn try_main() -> anyhow::Result<ExitCode> {
    // Must run before Args::parse() so --vendor-number/--issuer-id/--key-id/--key-path
    // can fall back to values loaded from .env.
    dotenvy::dotenv().ok();
    let args = args::Args::parse();

    let periods = args.start.inclusive_range(args.end_or_start())?;
    let credentials =
        app_store_reports::Credentials::load(&args.issuer_id, &args.key_id, &args.key_path)?;
    let client = app_store_reports::Client::new(credentials)?;
    std::fs::create_dir_all(&args.output_dir)?;
    let decompress = args.decompress_enabled();

    let (mut downloaded, mut skipped, mut failed) = (0u32, 0u32, 0u32);
    let regions = args.effective_regions();
    // The detailed report's Transaction Date/Settlement Date columns already
    // give the exact date per row, so the ~3-month reportDate/sales-period
    // shift (only observed for the aggregated FINANCIAL report) isn't applied
    // to its filename label.
    let label_period = |period: app_store_reports::Period| {
        if args.detailed {
            period
        } else {
            period.approx_sales_period()
        }
    };

    for period in periods {
        for region in &regions {
            let request = app_store_reports::FinanceReportRequest {
                vendor_number: args.vendor_number.clone(),
                region: region.clone(),
                period,
                detailed: args.detailed,
            };
            match client.fetch_report(&request) {
                Ok(raw) => {
                    let (gz_path, txt_path) = output::output_paths(
                        &args.output_dir,
                        label_period(period),
                        region,
                        args.detailed,
                    );
                    std::fs::write(&gz_path, &raw)?;
                    println!("saved  {}", gz_path.display());
                    downloaded += 1;

                    if decompress {
                        let text = app_store_reports::decompress(&raw)?;
                        std::fs::write(&txt_path, &text)?;
                        println!("saved  {}", txt_path.display());
                    }
                }
                Err(app_store_reports::Error::NotFound) => {
                    eprintln!("skip   {period} {region}: no report available");
                    skipped += 1;
                }
                Err(err) => {
                    eprintln!("FAILED {period} {region}: {err}");
                    failed += 1;
                }
            }
        }
    }

    println!(
        "\nDone. {}",
        output::summary_line(downloaded, skipped, failed)
    );
    Ok(if failed > 0 {
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    })
}
