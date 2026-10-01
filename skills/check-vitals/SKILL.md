---
name: check-vitals
description: Check app health (crash, ANR, LMK rates) and find top crash/ANR stacktraces via the Vitals API. Use after a release to detect regressions or when investigating stability.
depends_on: []
---

# check-vitals

Query Fire TV / Fire tablet app health metrics from the Vitals API (beta).

## When to use

- After publishing a new version, to catch crash/ANR regressions early (hourly data, T-2h).
- When investigating stability: find which version, device, or OS drives a rate spike, then drill into the top stacktraces.

## Parameters

| Name | Required | Description |
|------|----------|-------------|
| `package` | yes | App package name, e.g. `com.example.myapp` (not the App ID) |
| `metric_set` | for `query`/`freshness` | `crash`, `anr`, `lmk` (`issues` for freshness only) |

## Preconditions

- Same credentials as `reports`: security profile attached to the **Reporting API** (scope `adx_reporting::appstore:marketer`).
- Package must be owned by your vendor account.

## Commands

```sh
# One-shot snapshot: daily crash/ANR/LMK series for the last 7 days + top 5 crash and ANR issues
xingu +health com.example.myapp --days 7 --top 5

# Latest available data per granularity, plus valid metrics/dimensions for a set
xingu vitals freshness com.example.myapp crash

# Daily crash rate broken down by version
xingu vitals query com.example.myapp crash --metrics crashRate,crashCount,distinctDevices --dimensions versionCode --days 14

# Hourly ANR rate for the newest version on Fire TV only
xingu vitals query com.example.myapp anr --period hourly --days 2 --filter versionCode=151 --filter deviceType=AMAZON_FIRE_TV

# Top crash signatures with stacktraces over a date range
xingu vitals issues com.example.myapp --type crash --start 2026-04-01 --end 2026-04-15 --page-size 10
```

## Reference

- Metric sets: `crash` (crashRate, userPerceivedCrashRate, *7dUserWeighted, *28dUserWeighted, distinctDevices, crashCount), `anr` and `lmk` (same shape), `issues` (errorEventCount, affectedDeviceCount, reportText).
- Dimensions (max 5): `versionCode`, `countryCode`, `deviceModel`, `deviceType` (AMAZON_FIRE_TV, AMAZON_TABLETS), `deviceOS` (FIRE_OS, VEGA_OS), `osVersion`.
- `--filter DIM=V1,V2` is OR within a dimension, AND across repeated flags. Filters narrow results without adding rows; dimensions add one row per combination per period.
- Limits: DAILY covers 30 days, HOURLY 15 days. 28-day metrics are DAILY only. Dates are UTC (`YYYY-MM-DD`).

## Interpreting the result

- Rate queries return `rows[]` with `startTime`, `dimensions[]`, `metrics[]` (values are strings). `--output table` flattens them.
- Use rates for comparing versions/devices; counts for raw volume. Do not sum `distinctDevices` across periods.
- `issues` aggregates over the whole range: one row per crash signature, sorted by `errorEventCount` descending. LMK has no issues (no stacktrace).
- `rows: []` is not an error, just no data. Check `freshness` for the latest available timestamp.
- Use `--all` to follow `nextPageToken`; otherwise pass `--page-token` yourself.

## Error handling

- 400 `INVALID_ARGUMENT`: bad metric, dimension, or date range. Fix the request; the message lists valid values.
- 403: token expired or the security profile isn't attached to the Reporting API.
- 404 `NOT_FOUND`: unknown metric set or package.
- 429/5xx: retried automatically with backoff (limit is 3 req/sec).
