use anyhow::{bail, Context, Result};
use clap::Subcommand;

use crate::api::client::ApiClient;
use crate::output::{print_output, OutputFormat};

#[derive(Subcommand, Debug)]
pub enum ReportsCommands {
    /// Download monthly sales report
    Sales {
        /// Year (YYYY)
        year: String,
        /// Month (MM)
        month: String,
    },
    /// Download earnings report (yearly or monthly)
    Earnings {
        /// Year (YYYY)
        year: String,
        /// Month (MM, optional — omit for yearly)
        month: Option<String>,
    },
    /// Download monthly subscription report
    Subscription {
        /// Year (YYYY)
        year: String,
        /// Month (MM)
        month: String,
    },
    /// Download monthly subscriptions overview report
    SubscriptionsOverview {
        /// Year (YYYY)
        year: String,
        /// Month (MM)
        month: String,
    },
}

pub async fn run(
    cmd: &ReportsCommands,
    format: OutputFormat,
    dry_run: bool,
    timeout: u64,
) -> Result<()> {
    let path = match cmd {
        ReportsCommands::Sales { year, month } => {
            validate_year_month(year, month)?;
            format!("/download/report/sales/{year}/{month}")
        }
        ReportsCommands::Earnings { year, month } => {
            validate_year(year)?;
            if let Some(m) = month {
                validate_month(m)?;
                format!("/download/report/earnings/{year}/{m}")
            } else {
                format!("/download/report/earnings/{year}")
            }
        }
        ReportsCommands::Subscription { year, month } => {
            validate_year_month(year, month)?;
            format!("/download/report/subscription/{year}/{month}")
        }
        ReportsCommands::SubscriptionsOverview { year, month } => {
            validate_year_month(year, month)?;
            format!("/download/report/subscriptions_overview/{year}/{month}")
        }
    };

    if dry_run {
        println!("GET {path}");
        return Ok(());
    }

    let client = ApiClient::new_reporting(timeout).await?;
    let s3_url = client.get_raw(&path).await?;
    let s3_url = s3_url.trim().to_string();

    if s3_url.is_empty() {
        bail!("No report URL returned. The report may not be available.");
    }

    print_output(
        &serde_json::json!({ "downloadUrl": s3_url }),
        format,
    );
    Ok(())
}

fn validate_year(year: &str) -> Result<()> {
    let y: u16 = year.parse().context("year must be a 4-digit number")?;
    if !(2018..=2099).contains(&y) {
        bail!("Year must be between 2018 and 2099 (earliest available report is Jan 2018)");
    }
    Ok(())
}

fn validate_month(month: &str) -> Result<()> {
    let m: u8 = month.parse().context("month must be a 2-digit number")?;
    if !(1..=12).contains(&m) {
        bail!("Month must be between 01 and 12");
    }
    Ok(())
}

fn validate_year_month(year: &str, month: &str) -> Result<()> {
    validate_year(year)?;
    validate_month(month)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_year_valid() {
        assert!(validate_year("2018").is_ok());
        assert!(validate_year("2024").is_ok());
        assert!(validate_year("2099").is_ok());
    }

    #[test]
    fn test_validate_year_too_early() {
        assert!(validate_year("2017").is_err());
    }

    #[test]
    fn test_validate_year_too_late() {
        assert!(validate_year("2100").is_err());
    }

    #[test]
    fn test_validate_year_not_a_number() {
        assert!(validate_year("abcd").is_err());
    }

    #[test]
    fn test_validate_month_valid() {
        assert!(validate_month("01").is_ok());
        assert!(validate_month("6").is_ok());
        assert!(validate_month("12").is_ok());
    }

    #[test]
    fn test_validate_month_zero() {
        assert!(validate_month("0").is_err());
    }

    #[test]
    fn test_validate_month_thirteen() {
        assert!(validate_month("13").is_err());
    }

    #[test]
    fn test_validate_month_not_a_number() {
        assert!(validate_month("ab").is_err());
    }
}
