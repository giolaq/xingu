use anyhow::{bail, Context, Result};
use clap::{Args, Subcommand, ValueEnum};
use serde_json::{json, Map, Value};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::api::client::ApiClient;
use crate::output::{print_output, OutputFormat};

/// Dimensions documented for every Vitals metric set.
const DIMENSIONS: &[&str] = &[
    "versionCode",
    "countryCode",
    "deviceModel",
    "deviceType",
    "deviceOS",
    "osVersion",
];
const MAX_DIMENSIONS: usize = 5;
const MAX_PAGE_SIZE: u32 = 100_000;
const MAX_DAILY_RANGE_DAYS: i64 = 30;
const MAX_HOURLY_RANGE_DAYS: i64 = 15;
/// The API allows 3 req/sec; pause between sequential calls.
const PAGE_DELAY: Duration = Duration::from_millis(400);

#[derive(Subcommand, Debug)]
pub enum VitalsCommands {
    /// Show latest available data timestamp, metrics, and dimensions of a metric set
    Freshness {
        /// App package name (e.g. com.example.myapp)
        package: String,
        /// Metric set
        metric_set: MetricSet,
    },
    /// Query crash, ANR, or LMK rate metrics as a time series
    Query(QueryArgs),
    /// Top crash or ANR issues by frequency, with stacktraces
    Issues(IssuesArgs),
}

#[derive(ValueEnum, Clone, Copy, Debug, PartialEq)]
pub enum MetricSet {
    #[value(alias = "crashMetricSet")]
    Crash,
    #[value(alias = "anrMetricSet")]
    Anr,
    #[value(alias = "lmkMetricSet")]
    Lmk,
    #[value(alias = "issuesMetricSet")]
    Issues,
}

impl MetricSet {
    fn api_name(self) -> &'static str {
        match self {
            MetricSet::Crash => "crashMetricSet",
            MetricSet::Anr => "anrMetricSet",
            MetricSet::Lmk => "lmkMetricSet",
            MetricSet::Issues => "issuesMetricSet",
        }
    }
}

#[derive(ValueEnum, Clone, Copy, Debug, PartialEq)]
pub enum RateMetricSet {
    #[value(alias = "crashMetricSet")]
    Crash,
    #[value(alias = "anrMetricSet")]
    Anr,
    #[value(alias = "lmkMetricSet")]
    Lmk,
}

impl From<RateMetricSet> for MetricSet {
    fn from(m: RateMetricSet) -> Self {
        match m {
            RateMetricSet::Crash => MetricSet::Crash,
            RateMetricSet::Anr => MetricSet::Anr,
            RateMetricSet::Lmk => MetricSet::Lmk,
        }
    }
}

#[derive(ValueEnum, Clone, Copy, Debug, PartialEq, Default)]
pub enum Period {
    #[default]
    Daily,
    Hourly,
}

impl Period {
    fn api_name(self) -> &'static str {
        match self {
            Period::Daily => "DAILY",
            Period::Hourly => "HOURLY",
        }
    }

    fn max_range_days(self) -> i64 {
        match self {
            Period::Daily => MAX_DAILY_RANGE_DAYS,
            Period::Hourly => MAX_HOURLY_RANGE_DAYS,
        }
    }
}

#[derive(ValueEnum, Clone, Copy, Debug, PartialEq, Default)]
pub enum ReportType {
    #[default]
    Crash,
    Anr,
}

impl ReportType {
    fn api_name(self) -> &'static str {
        match self {
            ReportType::Crash => "CRASH",
            ReportType::Anr => "ANR",
        }
    }
}

#[derive(Args, Debug)]
pub struct WindowArgs {
    /// Start date (YYYY-MM-DD, UTC). Defaults to --end minus --days
    #[arg(long)]
    pub start: Option<String>,
    /// End date (YYYY-MM-DD, UTC). Defaults to today (UTC)
    #[arg(long)]
    pub end: Option<String>,
    /// Window length in days, used when --start is omitted
    #[arg(long, default_value_t = 7)]
    pub days: u32,
}

#[derive(Args, Debug)]
pub struct SliceArgs {
    /// Break down by dimensions (comma-separated, max 5):
    /// versionCode, countryCode, deviceModel, deviceType, deviceOS, osVersion
    #[arg(long, value_delimiter = ',')]
    pub dimensions: Vec<String>,
    /// Filter as DIMENSION=V1,V2 (repeatable; OR within a dimension, AND across)
    #[arg(long = "filter", value_name = "DIM=VALUES")]
    pub filters: Vec<String>,
    /// Max rows per page (default 1000, max 100000)
    #[arg(long)]
    pub page_size: Option<u32>,
    /// Page token from a previous response's nextPageToken
    #[arg(long, conflicts_with = "all")]
    pub page_token: Option<String>,
    /// Follow nextPageToken and return all rows
    #[arg(long)]
    pub all: bool,
}

#[derive(Args, Debug)]
pub struct QueryArgs {
    /// App package name (e.g. com.example.myapp)
    pub package: String,
    /// Rate metric set
    pub metric_set: RateMetricSet,
    /// Aggregation period
    #[arg(long, value_enum, default_value_t = Period::Daily)]
    pub period: Period,
    /// Metrics to return (comma-separated). Omit to return all metrics in the set
    #[arg(long, value_delimiter = ',')]
    pub metrics: Vec<String>,
    #[command(flatten)]
    pub window: WindowArgs,
    #[command(flatten)]
    pub slice: SliceArgs,
}

#[derive(Args, Debug)]
pub struct IssuesArgs {
    /// App package name (e.g. com.example.myapp)
    pub package: String,
    /// Error type
    #[arg(long = "type", value_enum, default_value_t = ReportType::Crash)]
    pub report_type: ReportType,
    #[command(flatten)]
    pub window: WindowArgs,
    #[command(flatten)]
    pub slice: SliceArgs,
}

#[derive(Args)]
pub struct HealthArgs {
    /// App package name (e.g. com.example.myapp)
    pub package: String,
    /// Look-back window in days (max 30)
    #[arg(long, default_value_t = 7)]
    pub days: u32,
    /// Number of top crash/ANR issues to include
    #[arg(long, default_value_t = 5)]
    pub top: u32,
}

pub async fn run(
    cmd: &VitalsCommands,
    format: OutputFormat,
    dry_run: bool,
    timeout: u64,
) -> Result<()> {
    match cmd {
        VitalsCommands::Freshness {
            package,
            metric_set,
        } => {
            let path = metric_set_path(package, *metric_set)?;
            if dry_run {
                println!("GET {path}");
                return Ok(());
            }
            let client = ApiClient::new_reporting(timeout).await?;
            let result = client.reporting_get(&path).await?;
            print_output(&result, format);
            Ok(())
        }
        VitalsCommands::Query(args) => {
            let path = format!(
                "{}:query",
                metric_set_path(&args.package, args.metric_set.into())?
            );
            let body = build_query_body(args, today_utc())?;
            execute_query(&path, body, &args.slice, format, dry_run, timeout).await
        }
        VitalsCommands::Issues(args) => {
            let path = format!(
                "{}:query",
                metric_set_path(&args.package, MetricSet::Issues)?
            );
            let body = build_issues_body(args, today_utc())?;
            execute_query(&path, body, &args.slice, format, dry_run, timeout).await
        }
    }
}

/// `+health`: crash/ANR/LMK daily series plus top crash and ANR issues in one call.
pub async fn health(
    args: &HealthArgs,
    format: OutputFormat,
    dry_run: bool,
    timeout: u64,
) -> Result<()> {
    validate_package(&args.package)?;
    let end = today_utc();
    let start = window_start(end, args.days, Period::Daily)?;
    let timeline = json!({
        "aggregationPeriod": Period::Daily.api_name(),
        "startTime": start.to_json(),
        "endTime": end.to_json(),
    });
    let rate_sets = [MetricSet::Crash, MetricSet::Anr, MetricSet::Lmk];
    let issue_types = [ReportType::Crash, ReportType::Anr];

    if dry_run {
        for set in rate_sets {
            println!("POST {}:query", metric_set_path(&args.package, set)?);
        }
        for t in issue_types {
            println!(
                "POST {}:query (reportType {})",
                metric_set_path(&args.package, MetricSet::Issues)?,
                t.api_name()
            );
        }
        return Ok(());
    }

    let client = ApiClient::new_reporting(timeout).await?;
    let mut out = Map::new();
    out.insert("package".into(), json!(args.package));
    out.insert(
        "window".into(),
        json!({ "start": start.to_string(), "end": end.to_string() }),
    );

    for (i, set) in rate_sets.into_iter().enumerate() {
        if i > 0 {
            tokio::time::sleep(PAGE_DELAY).await;
        }
        let path = format!("{}:query", metric_set_path(&args.package, set)?);
        let body = json!({ "timelineSpec": timeline });
        let section = match client.reporting_post(&path, &body).await {
            Ok(resp) => json!({
                "freshness": resp["freshness"],
                "rows": flatten_rows(&resp["rows"], false),
            }),
            Err(e) => json!({ "error": format!("{e:#}") }),
        };
        let key = match set {
            MetricSet::Crash => "crash",
            MetricSet::Anr => "anr",
            _ => "lmk",
        };
        out.insert(key.into(), section);
    }

    for t in issue_types {
        tokio::time::sleep(PAGE_DELAY).await;
        let path = format!(
            "{}:query",
            metric_set_path(&args.package, MetricSet::Issues)?
        );
        let body = json!({
            "startTime": start.to_json(),
            "endTime": end.to_json(),
            "reportType": t.api_name(),
            "pageSize": args.top,
        });
        let section = match client.reporting_post(&path, &body).await {
            Ok(resp) => flatten_rows(&resp["rows"], true),
            Err(e) => json!({ "error": format!("{e:#}") }),
        };
        let key = match t {
            ReportType::Crash => "topCrashIssues",
            ReportType::Anr => "topAnrIssues",
        };
        out.insert(key.into(), section);
    }

    print_output(&Value::Object(out), format);
    Ok(())
}

async fn execute_query(
    path: &str,
    mut body: Value,
    slice: &SliceArgs,
    format: OutputFormat,
    dry_run: bool,
    timeout: u64,
) -> Result<()> {
    if dry_run {
        println!("POST {path}");
        println!("{}", serde_json::to_string_pretty(&body)?);
        return Ok(());
    }

    let client = ApiClient::new_reporting(timeout).await?;
    let mut result = client.reporting_post(path, &body).await?;

    if slice.all {
        let mut rows = take_rows(&mut result);
        let mut pages = 1;
        while let Some(token) = result["nextPageToken"].as_str().map(str::to_string) {
            tokio::time::sleep(PAGE_DELAY).await;
            body["pageToken"] = json!(token);
            result = client.reporting_post(path, &body).await?;
            rows.extend(take_rows(&mut result));
            pages += 1;
        }
        if let Some(obj) = result.as_object_mut() {
            obj.remove("nextPageToken");
            obj.insert("resultCount".into(), json!(rows.len()));
            obj.insert("pages".into(), json!(pages));
            obj.insert("rows".into(), Value::Array(rows));
        }
    }

    match format {
        OutputFormat::Json => print_output(&result, format),
        OutputFormat::Table => {
            print_output(&flatten_rows(&result["rows"], false), format);
            if let Some(token) = result["nextPageToken"].as_str() {
                eprintln!("More rows available: --page-token {token} (or use --all)");
            }
        }
    }
    Ok(())
}

fn take_rows(result: &mut Value) -> Vec<Value> {
    match result.get_mut("rows").map(Value::take) {
        Some(Value::Array(rows)) => rows,
        _ => Vec::new(),
    }
}

fn metric_set_path(package: &str, set: MetricSet) -> Result<String> {
    validate_package(package)?;
    Ok(format!("/vitals/apps/{package}/{}", set.api_name()))
}

fn build_query_body(args: &QueryArgs, today: Date) -> Result<Value> {
    if args.period == Period::Hourly {
        if let Some(m) = args.metrics.iter().find(|m| m.contains("28d")) {
            bail!("{m} is only available with --period daily");
        }
    }
    let (start, end) = resolve_window(&args.window, args.period, today)?;
    let mut body = json!({
        "timelineSpec": {
            "aggregationPeriod": args.period.api_name(),
            "startTime": start.to_json(),
            "endTime": end.to_json(),
        },
    });
    if !args.metrics.is_empty() {
        body["metrics"] = json!(args.metrics);
    }
    apply_slice(&mut body, &args.slice)?;
    Ok(body)
}

fn build_issues_body(args: &IssuesArgs, today: Date) -> Result<Value> {
    let (start, end) = resolve_window(&args.window, Period::Daily, today)?;
    // issuesMetricSet takes top-level start/end and aggregates over the whole range.
    let mut body = json!({
        "startTime": start.to_json(),
        "endTime": end.to_json(),
        "reportType": args.report_type.api_name(),
    });
    apply_slice(&mut body, &args.slice)?;
    Ok(body)
}

fn apply_slice(body: &mut Value, slice: &SliceArgs) -> Result<()> {
    if slice.dimensions.len() > MAX_DIMENSIONS {
        bail!("At most {MAX_DIMENSIONS} dimensions are allowed");
    }
    for d in &slice.dimensions {
        validate_dimension(d)?;
    }
    if !slice.dimensions.is_empty() {
        body["dimensions"] = json!(slice.dimensions);
    }
    if !slice.filters.is_empty() {
        body["filter"] = parse_filters(&slice.filters)?;
    }
    if let Some(size) = slice.page_size {
        if !(1..=MAX_PAGE_SIZE).contains(&size) {
            bail!("--page-size must be between 1 and {MAX_PAGE_SIZE}");
        }
        body["pageSize"] = json!(size);
    }
    if let Some(token) = &slice.page_token {
        body["pageToken"] = json!(token);
    }
    Ok(())
}

/// Parse repeated `DIM=V1,V2` flags into `{"DIM": ["V1", "V2"]}`.
fn parse_filters(filters: &[String]) -> Result<Value> {
    let mut map = Map::new();
    for f in filters {
        let (dim, values) = f
            .split_once('=')
            .with_context(|| format!("invalid filter '{f}', expected DIMENSION=V1,V2"))?;
        let dim = dim.trim();
        validate_dimension(dim)?;
        let values: Vec<&str> = values
            .split(',')
            .map(str::trim)
            .filter(|v| !v.is_empty())
            .collect();
        if values.is_empty() {
            bail!("filter '{f}' has no values");
        }
        let entry = map.entry(dim.to_string()).or_insert_with(|| json!([]));
        if let Value::Array(arr) = entry {
            arr.extend(values.into_iter().map(|v| json!(v)));
        }
    }
    Ok(Value::Object(map))
}

fn validate_dimension(d: &str) -> Result<()> {
    if DIMENSIONS.contains(&d) {
        Ok(())
    } else {
        bail!(
            "Unknown dimension '{d}'. Valid dimensions: {}",
            DIMENSIONS.join(", ")
        )
    }
}

fn validate_package(package: &str) -> Result<()> {
    let valid = !package.is_empty()
        && package
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '_')
        && !package.starts_with('.');
    if !valid {
        bail!("Invalid package name '{package}' (expected e.g. com.example.myapp)");
    }
    Ok(())
}

/// Turn API rows (`dimensions`/`metrics` as name-value lists) into flat objects.
/// With `trim_trace`, long `reportText` stacktraces are cut to their first lines.
fn flatten_rows(rows: &Value, trim_trace: bool) -> Value {
    let Some(rows) = rows.as_array() else {
        return json!([]);
    };
    let flat = rows
        .iter()
        .map(|row| {
            let mut out = Map::new();
            let Some(obj) = row.as_object() else {
                return row.clone();
            };
            for (k, v) in obj {
                match (k.as_str(), v) {
                    ("startTime", Value::Object(t)) => {
                        out.insert("time".into(), json!(format_time(t)));
                    }
                    ("dimensions" | "metrics", Value::Array(pairs)) => {
                        for p in pairs {
                            if let Some(name) = p["name"].as_str() {
                                out.insert(name.into(), p["value"].clone());
                            }
                        }
                    }
                    ("reportText", Value::String(s)) if trim_trace => {
                        let head: Vec<&str> = s.lines().take(5).collect();
                        out.insert(k.clone(), json!(head.join("\n")));
                    }
                    _ => {
                        out.insert(k.clone(), v.clone());
                    }
                }
            }
            Value::Object(out)
        })
        .collect();
    Value::Array(flat)
}

fn format_time(t: &Map<String, Value>) -> String {
    let n = |k: &str| t.get(k).and_then(Value::as_i64).unwrap_or(0);
    let date = format!("{:04}-{:02}-{:02}", n("year"), n("month"), n("day"));
    match t
        .get("hours")
        .or_else(|| t.get("hour"))
        .and_then(Value::as_i64)
    {
        Some(h) => format!("{date}T{h:02}"),
        None => date,
    }
}

fn resolve_window(window: &WindowArgs, period: Period, today: Date) -> Result<(Date, Date)> {
    let end = match &window.end {
        Some(s) => Date::parse(s)?,
        None => today,
    };
    let start = match &window.start {
        Some(s) => Date::parse(s)?,
        None => window_start(end, window.days, period)?,
    };
    let span = end.days() - start.days();
    if span < 0 {
        bail!("--start ({start}) must not be after --end ({end})");
    }
    let max = period.max_range_days();
    if span > max {
        bail!(
            "{} queries are limited to {max} days (got {span})",
            period.api_name()
        );
    }
    Ok((start, end))
}

fn window_start(end: Date, days: u32, period: Period) -> Result<Date> {
    let max = period.max_range_days();
    if days == 0 || i64::from(days) > max {
        bail!(
            "--days must be between 1 and {max} for {} queries",
            period.api_name()
        );
    }
    Ok(Date::from_days(end.days() - i64::from(days)))
}

fn today_utc() -> Date {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    Date::from_days((secs / 86_400) as i64)
}

/// Calendar date in UTC, as the Vitals API expects `{year, month, day}`.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Date {
    year: i64,
    month: u32,
    day: u32,
}

impl Date {
    fn parse(s: &str) -> Result<Self> {
        let parts: Vec<&str> = s.split('-').collect();
        let err = || format!("invalid date '{s}', expected YYYY-MM-DD");
        if parts.len() != 3 || parts[0].len() != 4 {
            bail!(err());
        }
        let year: i64 = parts[0].parse().with_context(err)?;
        let month: u32 = parts[1].parse().with_context(err)?;
        let day: u32 = parts[2].parse().with_context(err)?;
        if !(1..=12).contains(&month) || day == 0 || day > days_in_month(year, month) {
            bail!(err());
        }
        Ok(Self { year, month, day })
    }

    fn to_json(self) -> Value {
        json!({ "year": self.year, "month": self.month, "day": self.day })
    }

    /// Days since 1970-01-01 (Howard Hinnant's days_from_civil).
    fn days(self) -> i64 {
        let y = if self.month <= 2 {
            self.year - 1
        } else {
            self.year
        };
        let era = y.div_euclid(400);
        let yoe = y - era * 400;
        let m = i64::from(self.month);
        let doy = (153 * (if m > 2 { m - 3 } else { m + 9 }) + 2) / 5 + i64::from(self.day) - 1;
        let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
        era * 146_097 + doe - 719_468
    }

    /// Inverse of `days` (Howard Hinnant's civil_from_days).
    fn from_days(z: i64) -> Self {
        let z = z + 719_468;
        let era = z.div_euclid(146_097);
        let doe = z - era * 146_097;
        let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
        let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
        let mp = (5 * doy + 2) / 153;
        let day = (doy - (153 * mp + 2) / 5 + 1) as u32;
        let month = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
        let year = yoe + era * 400 + i64::from(month <= 2);
        Self { year, month, day }
    }
}

impl std::fmt::Display for Date {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:04}-{:02}-{:02}", self.year, self.month, self.day)
    }
}

fn days_in_month(year: i64, month: u32) -> u32 {
    match month {
        2 if (year % 4 == 0 && year % 100 != 0) || year % 400 == 0 => 29,
        2 => 28,
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(s: &str) -> Date {
        Date::parse(s).unwrap()
    }

    fn window(start: Option<&str>, end: Option<&str>, days: u32) -> WindowArgs {
        WindowArgs {
            start: start.map(String::from),
            end: end.map(String::from),
            days,
        }
    }

    fn slice() -> SliceArgs {
        SliceArgs {
            dimensions: vec![],
            filters: vec![],
            page_size: None,
            page_token: None,
            all: false,
        }
    }

    #[test]
    fn test_date_roundtrip() {
        for s in ["1970-01-01", "2024-02-29", "2026-04-25", "2000-12-31"] {
            let date = d(s);
            assert_eq!(Date::from_days(date.days()), date);
            assert_eq!(date.to_string(), s);
        }
        assert_eq!(d("1970-01-01").days(), 0);
    }

    #[test]
    fn test_date_parse_invalid() {
        for s in [
            "2026-02-30",
            "2025-02-29",
            "2026-13-01",
            "26-01-01",
            "2026/01/01",
            "x",
        ] {
            assert!(Date::parse(s).is_err(), "{s} should be invalid");
        }
    }

    #[test]
    fn test_window_defaults_to_days_before_today() {
        let (start, end) =
            resolve_window(&window(None, None, 7), Period::Daily, d("2026-03-03")).unwrap();
        assert_eq!(start, d("2026-02-24"));
        assert_eq!(end, d("2026-03-03"));
    }

    #[test]
    fn test_window_limits() {
        let today = d("2026-04-30");
        assert!(resolve_window(&window(Some("2026-04-01"), None, 7), Period::Daily, today).is_ok());
        assert!(
            resolve_window(&window(Some("2026-03-01"), None, 7), Period::Daily, today).is_err()
        );
        assert!(
            resolve_window(&window(Some("2026-04-01"), None, 7), Period::Hourly, today).is_err()
        );
        assert!(resolve_window(&window(None, None, 16), Period::Hourly, today).is_err());
        assert!(resolve_window(
            &window(Some("2026-04-10"), Some("2026-04-09"), 7),
            Period::Daily,
            today
        )
        .is_err());
    }

    #[test]
    fn test_parse_filters() {
        let f = parse_filters(&[
            "countryCode=US,CA".into(),
            "deviceType=AMAZON_FIRE_TV".into(),
            "countryCode=DE".into(),
        ])
        .unwrap();
        assert_eq!(
            f,
            json!({"countryCode": ["US", "CA", "DE"], "deviceType": ["AMAZON_FIRE_TV"]})
        );
        assert!(parse_filters(&["bogus=1".into()]).is_err());
        assert!(parse_filters(&["countryCode".into()]).is_err());
        assert!(parse_filters(&["countryCode=".into()]).is_err());
    }

    #[test]
    fn test_validate_package() {
        assert!(validate_package("com.example.my_app").is_ok());
        assert!(validate_package("").is_err());
        assert!(validate_package("../etc").is_err());
        assert!(validate_package("com/example").is_err());
    }

    #[test]
    fn test_build_query_body() {
        let mut s = slice();
        s.dimensions = vec!["versionCode".into(), "countryCode".into()];
        s.filters = vec!["countryCode=US".into()];
        s.page_size = Some(500);
        let args = QueryArgs {
            package: "com.example".into(),
            metric_set: RateMetricSet::Crash,
            period: Period::Daily,
            metrics: vec!["crashRate".into(), "crashCount".into()],
            window: window(Some("2026-04-01"), Some("2026-04-08"), 7),
            slice: s,
        };
        let body = build_query_body(&args, d("2026-04-30")).unwrap();
        assert_eq!(
            body,
            json!({
                "timelineSpec": {
                    "aggregationPeriod": "DAILY",
                    "startTime": {"year": 2026, "month": 4, "day": 1},
                    "endTime": {"year": 2026, "month": 4, "day": 8},
                },
                "metrics": ["crashRate", "crashCount"],
                "dimensions": ["versionCode", "countryCode"],
                "filter": {"countryCode": ["US"]},
                "pageSize": 500,
            })
        );
    }

    #[test]
    fn test_build_query_body_rejects_28d_hourly() {
        let args = QueryArgs {
            package: "com.example".into(),
            metric_set: RateMetricSet::Crash,
            period: Period::Hourly,
            metrics: vec!["crashRate28dUserWeighted".into()],
            window: window(None, None, 3),
            slice: slice(),
        };
        assert!(build_query_body(&args, d("2026-04-30")).is_err());
    }

    #[test]
    fn test_build_query_body_rejects_too_many_dimensions() {
        let mut s = slice();
        s.dimensions = DIMENSIONS.iter().map(|d| d.to_string()).collect();
        let args = QueryArgs {
            package: "com.example".into(),
            metric_set: RateMetricSet::Anr,
            period: Period::Daily,
            metrics: vec![],
            window: window(None, None, 7),
            slice: s,
        };
        assert!(build_query_body(&args, d("2026-04-30")).is_err());
    }

    #[test]
    fn test_build_issues_body() {
        let args = IssuesArgs {
            package: "com.example".into(),
            report_type: ReportType::Anr,
            window: window(Some("2025-06-01"), Some("2025-06-15"), 7),
            slice: slice(),
        };
        let body = build_issues_body(&args, d("2025-06-20")).unwrap();
        assert_eq!(
            body,
            json!({
                "startTime": {"year": 2025, "month": 6, "day": 1},
                "endTime": {"year": 2025, "month": 6, "day": 15},
                "reportType": "ANR",
            })
        );
    }

    #[test]
    fn test_flatten_rows() {
        let rows = json!([{
            "startTime": {"year": 2026, "month": 4, "day": 25},
            "aggregationPeriod": "DAILY",
            "dimensions": [{"name": "versionCode", "value": "151"}],
            "metrics": [{"name": "crashRate", "value": "0.0142"}],
        }]);
        assert_eq!(
            flatten_rows(&rows, false),
            json!([{
                "time": "2026-04-25",
                "aggregationPeriod": "DAILY",
                "versionCode": "151",
                "crashRate": "0.0142",
            }])
        );
    }

    #[test]
    fn test_flatten_issue_rows_trims_trace() {
        let trace = (0..10)
            .map(|i| format!("line{i}"))
            .collect::<Vec<_>>()
            .join("\n");
        let rows = json!([{
            "crashDescriptor": "67f3",
            "reportText": trace,
            "metrics": [{"name": "errorEventCount", "value": "78"}],
        }]);
        let flat = flatten_rows(&rows, true);
        assert_eq!(flat[0]["crashDescriptor"], "67f3");
        assert_eq!(flat[0]["errorEventCount"], "78");
        assert_eq!(flat[0]["reportText"].as_str().unwrap().lines().count(), 5);
    }

    #[test]
    fn test_take_rows() {
        let mut v = json!({"rows": [1, 2], "nextPageToken": "x"});
        assert_eq!(take_rows(&mut v), vec![json!(1), json!(2)]);
        assert_eq!(take_rows(&mut json!({})), Vec::<Value>::new());
    }
}
