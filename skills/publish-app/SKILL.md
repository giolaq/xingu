---
name: publish-app
description: One-step publish — creates edit, uploads APK, and commits. The fastest path to publish when you only need to update the binary.
depends_on: []
---

# publish-app

Create an edit, upload an APK, and commit in one step.

## When to use

Use when you only need to publish a new APK with no listing or screenshot changes. This is the fastest path. For full control (update listing, add screenshots, validate first), use the individual commands in sequence.

## Parameters

| Name | Required | Description |
|------|----------|-------------|
| `app_id` | yes | The application ID |
| `file` | yes | Path to the APK file |

## Preconditions

- Auth configured (`xingu auth login` completed)
- The APK file must exist and be a valid Android package
- No active edit should already exist (this command creates one)

## Command

```sh
xingu +publish <app_id> --file <path_to_apk>
```

## Interpreting the result

- Response includes edit details, APK upload result, and commit confirmation.
- The app is now submitted for Amazon review (1-3 business days).

## Error handling

- Auth error (exit 2): Run `xingu auth login` and retry.
- Edit already exists: Run `xingu edits delete <app_id> <edit_id>` first, or use the step-by-step flow.
- Upload failed: Check file path, ensure APK is valid and under 2GB.
- Commit failed: The command auto-rolls back (deletes the edit). Check validation errors.
- Rollback failed: The dangling edit_id is printed. Delete manually: `xingu edits delete <app_id> <edit_id>`.

## Notes

- Use `--dry-run` to preview API calls without executing.
- The APK version code must be higher than the currently live version.
