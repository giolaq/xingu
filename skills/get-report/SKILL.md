---
name: get-report
description: Download sales, earnings, or subscription reports. Use when you need financial data about app performance.
depends_on: []
---

# get-report

Download sales, earnings, or subscription reports.

## When to use

Use when you need sales data, earnings, or subscription metrics. This is independent of the app submission workflow. Requires a separate security profile with reporting scope.

## Parameters

| Name | Required | Description |
|------|----------|-------------|
| `report_type` | yes | One of: `sales`, `earnings`, `subscription`, `subscriptions-overview` |
| `year` | yes | Year in YYYY format (2018 or later) |
| `month` | no | Month in MM format (01-12). Optional for yearly earnings. |

## Preconditions

- Auth configured with a security profile that has `adx_reporting::appstore:marketer` scope
- Reports are available ~30 days after the period ends

## Command

```sh
xingu reports <report_type> <year> [month]
```

Examples:

```sh
xingu reports sales 2025 04
xingu reports earnings 2025
xingu reports earnings 2025 03
xingu reports subscription 2025 04
```

## Interpreting the result

- Response contains a `downloadUrl` with a pre-signed S3 URL.
- The URL expires in minutes. Download immediately: `curl -o report.csv '<downloadUrl>'`
- Report format: CSV with headers.

## Error handling

- Auth error mentioning "reporting" or "marketer": The Reporting API requires a **separate** security profile with scope `adx_reporting::appstore:marketer`. Different from the submission API credentials.
- Empty URL: Report not available yet (~30 days after month ends).
- 404: No data for the requested period.
- Year/month validation: Year 2018-2099, month 01-12.

## Notes

- Sales: monthly only.
- Earnings: yearly (omit month) or monthly.
- Subscription/subscriptions-overview: monthly.
- The download URL needs no further auth to fetch.
