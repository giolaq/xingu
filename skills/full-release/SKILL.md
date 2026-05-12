# full-release

Orchestrated full release: create edit, upload APK, update listing, add screenshots, validate, and commit.

## Parameters

| Name | Required | Description |
|------|----------|-------------|
| `app_id` | yes | The application ID |
| `apk_file` | yes | Path to the APK file |
| `locale` | no | Primary locale code (default: en-US) |
| `title` | no | Updated app title (keeps current if omitted) |
| `description` | no | Updated full description |
| `short_description` | no | Updated short description |
| `recent_changes` | no | What's new text for this version |
| `screenshot_files` | no | Paths to screenshot images (replaces existing if provided) |

## Preconditions

- Auth configured (`xingu auth login` completed)
- APK file exists and is valid
- No active edit should exist (or you're okay deleting it)
- If providing screenshots, files must be PNG or JPG

## Workflow

### Step 1 — Check current state

Run `check-status` for the app.

- If an active DRAFT edit exists: decide whether to reuse it or delete and start fresh.
- If an IN_REVIEW edit exists: cannot proceed — wait for review to complete.
- If no edit exists: proceed.

### Step 2 — Create edit

Run `create-edit`. Save the `edit_id` from the response.

If this fails, abort.

### Step 3 — Upload APK

Run `upload-apk` with the `edit_id` and `apk_file`.

If this fails, run `delete-edit` to clean up, then abort.

### Step 4 — Update listing (optional)

If any of title, description, short_description, or recent_changes were provided, run `update-listing`.

If this fails, warn but continue — listing update is not blocking.

### Step 5 — Upload screenshots (optional)

If screenshot_files were provided, run `manage-screenshots` with action `replace-all`.

If this fails, warn but continue — screenshots can be added later.

### Step 6 — Validate

Run `validate-edit`.

- If validation passes: proceed to commit.
- If validation fails: use `troubleshoot-validation` to diagnose. Fix issues and re-validate.
- If errors are unfixable via API (e.g., content rating): abort and inform the user what needs manual action.

### Step 7 — Commit

Run `commit-edit`.

On success: release submitted. Amazon review expected in 1-3 business days.

On failure: the edit remains in DRAFT (not lost). Fix issues and retry commit.

## Rollback

- If any step after create-edit fails, the edit can be deleted with `delete-edit`.
- The edit is never auto-deleted on partial failure in this workflow (unlike `publish-app`).
- This gives you the chance to inspect and fix rather than losing all progress.

## Notes

- This is the recommended workflow for a complete app update.
- For APK-only updates with no listing changes, use `publish-app` instead (faster, atomic).
- Each step can be run independently — this skill defines the recommended sequence.
- Use `--dry-run` on individual commands to preview without executing.
