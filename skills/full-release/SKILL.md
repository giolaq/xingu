---
name: full-release
description: Orchestrated full release workflow — create edit, upload APK, update listing, add screenshots, validate, and commit. Use for complete app updates with metadata changes.
depends_on: [check-status, create-edit, upload-apk, update-listing, manage-screenshots, validate-edit, commit-edit]
---

# full-release

Orchestrated full release: create edit, upload APK, update listing, add screenshots, validate, and commit.

## When to use

Use for a complete app update that includes APK + listing changes + screenshots. For APK-only updates with no metadata changes, prefer `xingu +publish` (faster, atomic). Each step below can also be run independently.

## Parameters

| Name | Required | Description |
|------|----------|-------------|
| `app_id` | yes | The application ID |
| `apk_file` | yes | Path to the APK file |
| `locale` | no | Primary locale code (default: en-US) |
| `title` | no | Updated app title |
| `description` | no | Updated full description |
| `short_description` | no | Updated short description |
| `recent_changes` | no | What's new text |
| `screenshot_files` | no | Paths to screenshot images (replaces existing if provided) |

## Preconditions

- Auth configured (`xingu auth login` completed)
- APK file exists and is valid
- No active edit should exist (or delete the existing one first)

## Workflow

### Step 1 — Check current state

```sh
xingu +status <app_id>
```

- Active DRAFT edit exists: decide to reuse or delete with `xingu edits delete <app_id> <edit_id>`.
- IN_REVIEW edit exists: cannot proceed. Wait for review.
- No edit: proceed.

### Step 2 — Create edit

```sh
xingu edits create <app_id>
```

Save the `id` from the response as `<edit_id>`. If this fails, abort.

### Step 3 — Upload APK

```sh
xingu apks upload <app_id> <edit_id> --file <apk_file>
```

If this fails, clean up with `xingu edits delete <app_id> <edit_id>`, then abort.

### Step 4 — Update listing (optional)

If title, description, short_description, or recent_changes are provided:

```sh
xingu +update-listing <app_id> --locale <locale> --title "..." --description "..." --recent-changes "..."
```

If this fails, warn but continue.

### Step 5 — Upload screenshots (optional)

If screenshot_files are provided:

```sh
xingu images delete-all <app_id> <edit_id> --locale <locale> --image-type screenshots
xingu images upload <app_id> <edit_id> --locale <locale> --image-type screenshots --file <path>
```

If this fails, warn but continue.

### Step 6 — Validate

```sh
xingu edits validate <app_id> <edit_id>
```

- Passes: proceed to commit.
- Fails: diagnose with the troubleshoot-validation error table. Fix and re-validate.
- Unfixable (content rating, etc.): abort and inform user what needs manual action.

### Step 7 — Commit

```sh
xingu edits commit <app_id> <edit_id>
```

On success: release submitted. Amazon review in 1-3 business days.
On failure: edit stays in DRAFT. Fix issues and retry.

## Rollback

- If any step after create-edit fails, clean up: `xingu edits delete <app_id> <edit_id>`
- Unlike `xingu +publish`, this workflow does NOT auto-delete on failure. You keep progress.

## Notes

- Use `--dry-run` on individual commands to preview.
- For APK-only updates, use `xingu +publish` instead.
