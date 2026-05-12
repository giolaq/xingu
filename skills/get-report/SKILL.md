# get-report

Download sales, earnings, or subscription reports.

## Parameters

| Name | Required | Description |
|------|----------|-------------|
| `report_type` | yes | One of: `sales`, `earnings`, `subscription`, `subscriptions-overview` |
| `year` | yes | Year in YYYY format (2018 or later) |
| `month` | no | Month in MM format (01-12). Optional for yearly earnings. |

## Preconditions

- Auth configured with a security profile that has `adx_reporting::appstore:marketer` scope
- Valid year range: 2018–present
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

- Response contains a `downloadUrl` field with a pre-signed S3 URL.
- The URL is temporary (expires in minutes) — download immediately.
- Report format: CSV with headers.
- To download: `curl -o report.csv '<downloadUrl>'`

## Error handling

- Auth error mentioning "reporting" or "marketer": The Reporting API requires a **separate** security profile with scope `adx_reporting::appstore:marketer`. This is different from the submission API credentials.
- Empty URL returned: The report isn't available yet. Reports are generated ~30 days after the month ends.
- 404: No data exists for the requested period.
- Validation error about year/month: Year must be 2018–2099, month must be 01–12.

## Notes

- Sales reports are monthly only.
- Earnings reports can be yearly (omit month) or monthly.
- Subscription and subscriptions-overview reports are monthly.
- The download URL is a direct link to a CSV — no further authentication needed to fetch it.
- For programmatic use, pipe the URL to curl or wget immediately.
