<p align="center">
  <img src="assets/logo.png" alt="xingu logo" width="200">
</p>

<h1 align="center">xingu</h1>

<p align="center">
  <strong>Unofficial</strong> Amazon Appstore CLI — for humans and agents.<br>
  Publish apps, download reports, and monitor app health from your terminal or CI.
</p>

<p align="center">
  <a href="https://github.com/giolaq/xingu/actions/workflows/ci.yml"><img src="https://github.com/giolaq/xingu/actions/workflows/ci.yml/badge.svg" alt="CI"></a>
  <a href="https://crates.io/crates/xingu"><img src="https://img.shields.io/crates/v/xingu.svg" alt="crates.io"></a>
  <a href="./LICENSE"><img src="https://img.shields.io/badge/license-MIT-blue.svg" alt="MIT license"></a>
</p>

<p align="center">
  Named after the <a href="https://en.wikipedia.org/wiki/Xingu_River">Xingu River</a>, a major tributary of the Amazon.
</p>

---

## Features

- **Publish** — create edits, upload APKs, update listings, images, videos, availability, and device targeting ([App Submission API](https://developer.amazon.com/docs/app-submission-api/overview.html)).
- **Report** — download sales, earnings, and subscription reports ([Reporting API](https://developer.amazon.com/docs/reports-promo/reporting-API.html)).
- **Monitor** — crash, ANR, and LMK rates plus top crash stacktraces for Fire TV and Fire tablet apps ([Vitals API](https://developer.amazon.com/docs/reports-promo/vitals-api-overview.html), beta).
- **Agent-friendly** — JSON output by default, stable exit codes, `--dry-run` on every command, and bundled agent skills.
- **Safe by default** — credentials in the OS keyring, automatic ETag handling, retries with backoff, token refresh.

## Contents

- [Quick start](#quick-start)
- [Install](#install)
- [Authentication](#authentication)
- [Usage](#usage)
  - [Publishing](#publishing)
  - [Financial reports](#financial-reports)
  - [App health (Vitals API)](#app-health-vitals-api)
- [Command reference](#command-reference)
- [Configuration](#configuration)
- [Agent integration](#agent-integration)
- [Troubleshooting](#troubleshooting)
- [Security](#security)
- [Development](#development)
- [Disclaimer](#disclaimer) · [License](#license)

## Quick start

```bash
cargo install xingu
xingu auth setup                      # paste Client ID + Client Secret
xingu +status <app-id>                # app info + active edit
xingu +publish <app-id> --file app.apk
xingu +health com.example.myapp       # crash/ANR/LMK snapshot
```

See [Authentication](#authentication) for how to get a Client ID and Secret.

## Install

### From crates.io (recommended)

```bash
cargo install xingu
```

### Prebuilt binaries

Download from the [Releases](https://github.com/giolaq/xingu/releases) page:

| Platform | Binary |
|----------|--------|
| macOS (Apple Silicon) | `xingu-macos-arm64` |
| macOS (Intel) | `xingu-macos-amd64` |
| Linux (x86_64) | `xingu-linux-amd64` |

```bash
# Example: macOS Apple Silicon
curl -L -o xingu https://github.com/giolaq/xingu/releases/latest/download/xingu-macos-arm64
chmod +x xingu
sudo mv xingu /usr/local/bin/
```

### From source

```bash
git clone https://github.com/giolaq/xingu.git
cd xingu
cargo build --release
./target/release/xingu --help
```

Keep it current with `xingu update`. Enable shell completions with `xingu completions zsh` (or `bash`, `fish`).

## Authentication

xingu uses a Login with Amazon **security profile** (client credentials). Each Amazon API needs the profile attached to it:

| Commands | Attach the profile to | OAuth scope (handled by xingu) |
|----------|-----------------------|--------------------------------|
| `apps`, `edits`, `apks`, `listings`, `details`, `images`, `videos`, `availability`, `targeting`, `+publish`, `+status`, `+update-listing` | **App Submission API** | `appstore::apps:readwrite` |
| `reports`, `vitals`, `+health` | **Reporting API** | `adx_reporting::appstore:marketer` |

### 1. Create a security profile

You need **Admin** access to your Amazon Developer account.

1. Sign in to the [Amazon Developer Console](https://developer.amazon.com/).
2. Go to **My Settings → API Access**.
3. Select an API (e.g. **App Submission API**), click **Create a new security profile**, and save it.
4. On the API Access page, select the profile from the drop-down and click **Attach**. Repeat for every API you need (see the table above). Using **one profile for both APIs** is simplest — xingu stores a single Client ID/Secret.
5. Open the profile's **Web Settings** tab and copy the **Client ID** and **Client Secret**.

### 2. Save credentials

```bash
xingu auth setup
# Client ID: amzn1.application-oa2-client.xxxxxxxx
# Client Secret: amzn1.oa2-cs.v1.xxxxxxxx
# → Credentials saved to OS keyring.
```

On macOS, allow Keychain access when prompted ("Always Allow").

### 3. Verify

```bash
xingu auth login                                           # App Submission API
xingu --verbose vitals freshness com.example.myapp crash   # Reporting API
```

Tokens are fetched and cached automatically (~1 hour) per API, so you rarely need `auth login`. If your profile is only attached to the Reporting API, `auth login` fails with `invalid_scope` — that is expected; use a `reports`/`vitals` command to verify instead.

## Usage

All commands print JSON by default. Add `--output table` for humans, `--dry-run` to preview the HTTP requests, and `--verbose` to see method, URL, status, and timing.

### Publishing

> The App Submission API only manages **updates**. Submit the first version of an app through the [Developer Console](https://developer.amazon.com/). Find the App ID under **My Apps → your app → Additional Information**.

```bash
# App status + active edit
xingu +status <app-id>

# One step: create edit → upload APK → commit
xingu +publish <app-id> --file app.apk

# Step by step, with validation before submitting
EDIT=$(xingu edits create <app-id> | jq -r .id)
xingu apks upload <app-id> "$EDIT" --file app.apk
xingu edits validate <app-id> "$EDIT"
xingu edits commit <app-id> "$EDIT"

# Update listing text without a new APK
xingu +update-listing <app-id> --locale en-US --title "My App" --description "..."
```

**App state requirements.** The API returns `412 Precondition Failed` when the app's state blocks edits:

| App state | Can create edits? |
|-----------|-------------------|
| Published | ✅ Yes |
| Incomplete (never submitted) | ❌ No — submit the first version via the Console |
| Submitted / Under Review | ❌ No — wait for the review to finish |
| Suppressed | ❌ No |

**ETags.** Most `PUT`, `DELETE`, and some `POST` operations require an `If-Match` header. xingu fetches the current ETag automatically before mutating requests.

### Financial reports

Each command returns a pre-signed S3 `downloadUrl` that is valid for **5 minutes**. Reports are available from January 2018.

```bash
xingu reports sales 2024 04                    # monthly sales
xingu reports earnings 2024                    # yearly earnings
xingu reports earnings 2024 03                 # monthly earnings
xingu reports subscription 2024 06             # monthly subscriptions
xingu reports subscriptions-overview 2024 06   # subscriptions overview

# Download right away
curl -o sales.zip "$(xingu reports sales 2024 04 | jq -r .downloadUrl)"
```

### App health (Vitals API)

> **Beta.** The [Vitals API](https://developer.amazon.com/docs/reports-promo/vitals-api-overview.html) is in open beta and may change. It uses the same credentials as `reports` (Reporting API).

Vitals takes your app's **package name** (e.g. `com.example.myapp`), not the App ID.

#### Snapshot

```bash
xingu +health com.example.myapp --days 7 --top 5
```

Returns daily crash, ANR, and LMK series for the window plus the top crash and ANR issues (stacktraces trimmed to 5 lines). A failing section shows an `error` field; the rest still returns.

#### Freshness

Check what data is available before querying. This also lists the valid metrics and dimensions for a metric set.

```bash
xingu vitals freshness com.example.myapp crash    # crash | anr | lmk | issues
```

| Granularity | Available after | Retention | Max range per query |
|-------------|-----------------|-----------|---------------------|
| `DAILY` | T-1 day | 30 days | 30 days |
| `HOURLY` | T-2 hours | 15 days | 15 days |

#### Rate metrics (time series)

```bash
# Last 7 days, all metrics (defaults)
xingu vitals query com.example.myapp crash --output table

# Crash rate per app version, last 14 days
xingu vitals query com.example.myapp crash \
  --metrics crashRate,crashCount,distinctDevices --dimensions versionCode --days 14

# Hourly ANR rate on Fire TV in the US and Canada (early regression detection)
xingu vitals query com.example.myapp anr --period hourly --days 2 \
  --filter deviceType=AMAZON_FIRE_TV --filter countryCode=US,CA

# Explicit date range (UTC)
xingu vitals query com.example.myapp lmk --start 2026-04-01 --end 2026-04-15
```

#### Top issues (stacktraces)

```bash
xingu vitals issues com.example.myapp --type crash --page-size 10
xingu vitals issues com.example.myapp --type anr --dimensions deviceType,osVersion
```

`issues` aggregates over the whole date range and returns one row per crash signature, sorted by `errorEventCount` (highest first). LMK has no issues report because LMK events have no stacktrace.

#### Reference

| Metric set | Metrics |
|------------|---------|
| `crash` | `crashRate`, `crashRate7dUserWeighted`, `crashRate28dUserWeighted`, `userPerceivedCrashRate` (+ `7d`/`28d` variants), `distinctDevices`, `crashCount` |
| `anr` | Same shape with `anr…` names, `distinctDevices`, `anrCount` |
| `lmk` | Same shape with `lmk…` names, `distinctDevices`, `lmkCount` |
| `issues` | `errorEventCount`, `affectedDeviceCount`, `reportText` |

| Dimension | Example values |
|-----------|----------------|
| `versionCode` | `151` |
| `countryCode` | `US`, `CA` |
| `deviceModel` | `AFTSS` |
| `deviceType` | `AMAZON_FIRE_TV`, `AMAZON_TABLETS` |
| `deviceOS` | `FIRE_OS`, `VEGA_OS` |
| `osVersion` | `RS8401/2542` |

| Option | Description |
|--------|-------------|
| `--period daily\|hourly` | Granularity (`query` only, default `daily`) |
| `--start`, `--end` | UTC dates, `YYYY-MM-DD`. `--end` defaults to today |
| `--days N` | Window length when `--start` is omitted (default 7) |
| `--metrics a,b` | Subset of metrics (`query` only). Omit for all — slower |
| `--dimensions a,b` | Break down by up to 5 dimensions: one row per combination per period |
| `--filter DIM=V1,V2` | Narrow results without adding rows. Repeatable: OR within a dimension, AND across |
| `--page-size N` | Rows per page (default 1000, max 100000) |
| `--page-token T` / `--all` | Fetch the next page / follow all pages |
| `--type crash\|anr` | Error type (`issues` only, default `crash`) |

Tips:

- Use **rates** to compare versions or devices and **counts** for raw volume. Don't sum `distinctDevices` across periods.
- 28-day metrics are DAILY only; xingu rejects them with `--period hourly`.
- `rows: []` is not an error — there is no data for that window. Check `freshness`.
- Requests are limited to 3/sec. xingu paces pagination and retries `429`/`5xx` with backoff.
- Numbers should match the **App Health Insights** dashboard in the Developer Console.

## Command reference

| Command | Description |
|---------|-------------|
| `auth setup` | Configure API credentials |
| `auth login` | Acquire a fresh App Submission API token |
| `auth token` | Print the current App Submission API token |
| `apps get` | Get the active edit for an app |
| `edits create/get/get-previous/validate/delete/commit` | Manage edits |
| `apks list/get/upload/replace/delete` | Manage APK files |
| `listings list/get/update/delete` | Manage store listings per locale |
| `details get/update` | Manage app details |
| `images list/upload/delete/delete-all` | Manage screenshots and icons per locale |
| `videos list/upload/delete/delete-all` | Manage videos per locale |
| `availability get/update` | Manage availability and scheduling |
| `targeting get/update` | Manage APK device targeting |
| `reports sales/earnings/subscription/subscriptions-overview` | Download sales and financial reports |
| `vitals freshness/query/issues` | Crash, ANR, LMK metrics and top issues (beta) |
| `+publish` | One step: edit → upload → commit |
| `+status` | App info + active edit summary |
| `+update-listing` | Update listing fields directly |
| `+health` | Crash/ANR/LMK snapshot + top crash/ANR issues |
| `skills list/show/find/add` | Manage agent skills |
| `init` | Initialize a project for agent use (writes `AGENTS.md`) |
| `info` | Show environment info (config dir, caches, overrides) |
| `update` | Self-update from GitHub Releases |
| `completions <shell>` | Generate shell completions (bash, zsh, fish) |

Run `xingu <command> --help` for all options.

## Configuration

### Global flags

| Flag | Default | Description |
|------|---------|-------------|
| `--output json\|table` | `json` | Output format |
| `--dry-run` | `false` | Print the requests without executing them |
| `--verbose` | `false` | Show HTTP method, URL, status, timing, and retries |
| `--timeout <secs>` | `30` | Request timeout in seconds |

### Environment variables

| Variable | Description |
|----------|-------------|
| `XINGU_TOKEN` | Pre-obtained bearer token. Highest priority; you manage its scope |
| `XINGU_CLIENT_ID` | OAuth client ID. Overrides stored credentials |
| `XINGU_CLIENT_SECRET` | OAuth client secret. Overrides stored credentials |
| `XINGU_BASE_URL` | Override the API base URL (HTTPS amazon.com or localhost only, for testing) |

Credentials are resolved in this order: environment variables → OS keyring → credentials file.

### Exit codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | API error (or invalid arguments) |
| 2 | Authentication error |
| 3 | Validation error |
| 4 | Network error |

## Agent integration

xingu is designed to be driven by AI agents and scripts: structured JSON output, predictable exit codes, and no interactive prompts outside `auth setup`.

- Invoke `xingu` with an argument array, not shell string interpolation, to avoid command injection.
- Use `XINGU_CLIENT_ID` / `XINGU_CLIENT_SECRET` for non-interactive auth.
- Use `--dry-run` to show the user what will happen before mutating anything.
- Run `xingu init` in a project to write an `AGENTS.md` quick reference.

### Typical workflow

```bash
xingu +status <app-id>
# → { "appId": "...", "activeEdit": { "id": "...", "status": "IN_PROGRESS" } }
# → { "appId": "...", "activeEdit": {} }   ← no active edit

xingu +publish <app-id> --file app.apk --dry-run
# → POST /applications/<app-id>/edits
# → POST /applications/<app-id>/edits/<edit_id>/apks/upload (file: app.apk)
# → POST /applications/<app-id>/edits/<edit_id>/commit

xingu +publish <app-id> --file app.apk
xingu +health com.example.myapp --days 1   # watch for regressions after release
```

### Error handling

```bash
output=$(xingu edits create <app-id> 2>&1)
case $? in
  0) edit_id=$(echo "$output" | jq -r .id) ;;
  1) echo "API error: $output" ;;        # e.g. 412 Precondition Failed
  2) echo "Auth error: $output" ;;       # check credentials / attached APIs
  3) echo "Validation error: $output" ;;
  4) echo "Network error: $output" ;;
esac
```

App Submission API errors are structured JSON:

```json
{
  "httpCode": 412,
  "message": "Precondition Failed",
  "errors": [{ "errorCode": "error_new_version_creation_not_allowed", "errorMessage": "..." }]
}
```

Vitals API errors include a code and a message listing valid values:

```json
{ "code": "INVALID_ARGUMENT", "message": "Unknown metric 'bogus'. Valid metrics for crashMetricSet: ...", "status": 400 }
```

### Skills

Skills are Markdown runbooks (`skills/<name>/SKILL.md`) that agents read and follow.

```bash
xingu skills list                     # list bundled skills
xingu skills show check-vitals        # print a skill
xingu skills add                      # install all skills
xingu skills add --skill upload-apk   # install one skill
```

`skills add` installs into `~/.xingu/skills/` and detected agent directories (`~/.claude/skills/`, `~/.gemini/skills/`, `~/.kiro/skills/`, `~/.cursor/skills/`, `~/.antigravity/skills/`).

| Skill | Description |
|-------|-------------|
| `check-status` | Get app info, active edit status, and suggest next actions |
| `check-targeting` | View and interpret device targeting for an APK |
| `create-edit` | Create a new draft edit — required before any changes |
| `upload-apk` | Upload a new APK file to an existing edit |
| `update-listing` | Update store listing metadata for a locale |
| `manage-screenshots` | Upload, list, or replace screenshots |
| `validate-edit` | Validate an edit before committing |
| `commit-edit` | Submit an edit for Amazon review |
| `delete-edit` | Delete a draft edit to cancel or recover |
| `publish-app` | One step: edit → upload → commit |
| `full-release` | Orchestrated workflow: edit → APK → listing → screenshots → validate → commit |
| `get-report` | Download sales, earnings, or subscription reports |
| `check-vitals` | Crash/ANR/LMK rates and top stacktraces (Vitals API) |
| `troubleshoot-validation` | Diagnose and fix validation errors |
| `rollback-edit` | Recover from a stuck or failed edit state |

## Troubleshooting

| Symptom | Cause | Fix |
|---------|-------|-----|
| `OAuth token request failed (401) … invalid_client` | Wrong Client ID/Secret, or stale credentials | Check `env \| grep XINGU` (env vars win), then re-run `xingu auth setup` with values from the profile's **Web Settings** tab |
| `OAuth token request failed … invalid_scope` | Profile not attached to the API this command needs | Attach it under **My Settings → API Access** (see [Authentication](#authentication)) |
| `Authentication failed (403)` on `reports`/`vitals` | Profile not attached to the **Reporting API**, or expired token | Attach the profile; xingu refreshes tokens automatically |
| `412 Precondition Failed` on edits | App state blocks edits | See [App state requirements](#publishing) |
| Vitals `404 NOT_FOUND` | Wrong package name or metric set, or the package isn't owned by your account | Use the package name, not the App ID |
| Vitals `rows: []` | No data in the window yet | Check `xingu vitals freshness`; data lags 1 day (daily) or 2 hours (hourly) |
| Vitals `400 INVALID_ARGUMENT` | Unknown metric/dimension or range too wide | Read the message — it lists valid values |

Run any command with `--verbose` to see the request, status, and retries, and `xingu info` to see which config dir, caches, and overrides are in use.

## Security

### Credential storage

Credentials are stored in the OS keyring on macOS (Keychain) and Windows (Credential Manager). On Linux, or if the keyring is unavailable, they are stored in `credentials.json` in the xingu config directory (see `xingu info`) with `0600` permissions. The config directory is set to `0700`.

### Token handling

- The App Submission API and Reporting API (including Vitals) use different OAuth scopes. xingu keeps a separate cached token for each (`token_cache.json`, `reporting_token_cache.json`, `0600`, ~1 hour) and refreshes the right one on `401`/`403`.
- `xingu auth token` prints the full bearer token to stdout for piping. Mind shell history and logs.
- `--verbose` never logs tokens or credentials.

### Base URL override

`XINGU_BASE_URL` is restricted to HTTPS amazon.com domains and localhost, so bearer tokens can't be sent to untrusted hosts if environment variables are tampered with.

## Development

```bash
cargo build
cargo test
cargo fmt -- --check
cargo clippy --all-targets -- -D warnings
```

CI runs all four on Linux and macOS. Pull requests are welcome — please keep them green. Planned work lives in [TODOS.md](./TODOS.md).

## Disclaimer

This is an unofficial, community-built tool, not affiliated with or endorsed by Amazon. "Amazon", "Amazon Appstore", "Fire TV", and related names are trademarks of Amazon.com, Inc. Use of the Amazon APIs is subject to Amazon's terms, including the Program Materials License Agreement for the Vitals API.

## License

[MIT](./LICENSE)
