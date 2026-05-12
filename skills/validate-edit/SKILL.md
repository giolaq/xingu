---
name: validate-edit
description: Validate an edit before committing to catch errors early. Always run this before commit-edit.
depends_on: [upload-apk]
---

# validate-edit

Validate an edit before committing. Catches errors early without submitting.

## When to use

Always run this before `xingu edits commit`. It's free, idempotent, and saves you from rejected submissions.

## Parameters

| Name | Required | Description |
|------|----------|-------------|
| `app_id` | yes | The application ID |
| `edit_id` | yes | The edit ID to validate |

## Preconditions

- An active edit must exist in DRAFT status
- The edit should have at least an APK uploaded

## Command

```sh
xingu edits validate <app_id> <edit_id>
```

## Interpreting the result

- No errors: Ready to commit with `xingu edits commit <app_id> <edit_id>`.
- Warnings: Non-blocking, won't prevent submission.
- Errors: Must be fixed before committing. See the troubleshoot-validation skill for a diagnosis table.

## Error handling

- ETag conflict (412): Retry.
- Edit not found (404): The edit was deleted or already committed.

## Common validation failures

- Missing APK: `xingu apks upload <app_id> <edit_id> --file <path>`
- Missing listing: `xingu +update-listing <app_id> --locale en-US --title "..."`
- Missing screenshots: `xingu images upload <app_id> <edit_id> --locale en-US --image-type screenshots --file <path>`
- Missing icon: `xingu images upload <app_id> <edit_id> --locale en-US --image-type small-icons --file icon.png`
- Version code not incremented: Rebuild APK with higher versionCode.
- Content rating missing: Must be set in the Developer Console UI (not available via API).

## Notes

- Idempotent. Run as many times as needed.
- Some issues (content rating, categories) can only be fixed in the web console.
