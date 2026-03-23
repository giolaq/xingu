<p align="center">
  <img src="assets/logo.png" alt="xingu logo" width="200">
</p>

<h1 align="center">xingu</h1>

<p align="center">
  <strong>Unofficial</strong> Amazon AppStore CLI — for humans and agents.<br>
  Named after the <a href="https://en.wikipedia.org/wiki/Xingu_River">Xingu River</a>, a major tributary of the Amazon.<br>
  <sub>Not affiliated with or endorsed by Amazon.</sub>
</p>

## Install

### From source

```bash
cargo install --path .
```

### Build from repo

```bash
git clone <repo-url>
cd xingu
cargo build --release
./target/release/xingu --help
```

## Setup

1. Go to the [Amazon Developer Console](https://developer.amazon.com/)
2. Navigate to **Tools & Services > API Access**
3. Create a new **Security Profile** and note your Client ID and Client Secret
4. Attach the security profile to the **App Submission API**

```bash
xingu auth setup
# Enter your Client ID and Client Secret when prompted

xingu auth login
# Acquires and caches an OAuth token (~1 hour TTL)
```

### Environment variables

| Variable | Description |
|----------|-------------|
| `XINGU_TOKEN` | Pre-obtained bearer token (highest priority) |
| `XINGU_CLIENT_ID` | OAuth client ID |
| `XINGU_CLIENT_SECRET` | OAuth client secret |
| `XINGU_BASE_URL` | Override API base URL (for testing) |

## Usage

```bash
# List all your apps
xingu apps list

# Get app details
xingu apps get <app-id>

# Create an edit (draft version)
xingu edits create <app-id>

# Upload an APK
xingu apks upload <app-id> <edit-id> --file app.apk

# Commit (publish) the edit
xingu edits commit <app-id> <edit-id>

# One-step publish: create edit → upload APK → commit
xingu +publish <app-id> --file app.apk

# Get app status + active edit
xingu +status <app-id>

# Update store listing
xingu +update-listing <app-id> --locale en-US --title "My App" --description "..."
```

## Commands

| Command | Description |
|---------|-------------|
| `auth setup` | Configure API credentials |
| `auth login` | Acquire fresh OAuth token |
| `auth token` | Print current access token |
| `apps list` | List all apps |
| `apps get` | Get app details |
| `edits create/get/delete/commit` | Manage edits |
| `apks list/upload/replace/delete` | Manage APK files |
| `listings get/update` | Manage store listings |
| `details get/update` | Manage app details |
| `images list/upload/delete` | Manage screenshots/icons |
| `videos upload` | Upload videos |
| `availability get/update` | Manage availability |
| `+publish` | One-step: edit → upload → commit |
| `+status` | App info + active edit summary |
| `+update-listing` | Update listing fields directly |

## Global flags

| Flag | Default | Description |
|------|---------|-------------|
| `--output json\|table` | `json` | Output format |
| `--dry-run` | `false` | Preview requests without executing |
| `--verbose` | `false` | Show HTTP method, URL, status, timing |
| `--timeout <secs>` | `30` | Request timeout in seconds |

## Exit codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | API error |
| 2 | Authentication error |
| 3 | Validation error |
| 4 | Network error |

## Agent integration

Agents should invoke `xingu` as a subprocess with proper argument arrays (not shell string interpolation) to avoid command injection. All commands output structured JSON by default.

### Skills

YAML skill definitions in `skills/` for common workflows:
- `upload-apk.yaml` — Upload an APK
- `publish-app.yaml` — One-step publish
- `update-listing.yaml` — Update store listing
- `check-status.yaml` — Check app status

## Security

### Credential storage

Credentials are stored in the OS keyring (macOS Keychain, Linux secret-service) when available. Falls back to `~/.config/xingu/credentials.json` with `0600` permissions. The config directory is set to `0700`.

### Token handling

- `xingu auth token` outputs the full bearer token to stdout (for piping). Be careful with shell history and logging.
- Token cache (`token_cache.json`) has `0600` permissions and expires after ~1 hour.
- `--verbose` mode never logs tokens or credentials.

### Base URL override

`XINGU_BASE_URL` is restricted to HTTPS amazon.com domains and localhost. This prevents credential exfiltration via SSRF if an attacker controls environment variables.

## Disclaimer

This is an unofficial, community-built tool. It is not affiliated with, endorsed by, or supported by Amazon. "Amazon", "Amazon Appstore", and related names are trademarks of Amazon.com, Inc.

## License

MIT
