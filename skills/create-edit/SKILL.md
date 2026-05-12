---
name: create-edit
description: Create a new draft edit for an app. Required before uploading APKs, updating listings, or making any changes. Use when check-status shows no active edit.
depends_on: [check-status]
---

# create-edit

Create a new draft edit for an app. Required before any changes.

## When to use

Use when `xingu +status` shows no active edit and you need to make changes. If you just want to publish an APK with no other changes, prefer `xingu +publish` which handles this automatically.

## Parameters

| Name | Required | Description |
|------|----------|-------------|
| `app_id` | yes | The application ID |

## Preconditions

- Auth configured (`xingu auth login` completed)
- No active edit should already exist (only one at a time is allowed)

## Command

```sh
xingu edits create <app_id>
```

## Interpreting the result

- Response contains the edit object with an `id` field. Save this for subsequent commands.
- The edit starts in DRAFT status and copies the current live app state.

## Error handling

- 409 Conflict: An edit already exists. Run `xingu edits get <app_id>` to get it, or `xingu edits delete <app_id> <edit_id>` to start fresh.
- Auth error (exit 2): Run `xingu auth login`.
- 404: The app_id is invalid.

## Next steps

- Upload an APK: `xingu apks upload <app_id> <edit_id> --file <path>`
- Update listing: `xingu +update-listing <app_id> --locale en-US --title "..."`
- Upload screenshots: `xingu images upload <app_id> <edit_id> --locale en-US --image-type screenshots --file <path>`
- When done: `xingu edits validate <app_id> <edit_id>` then `xingu edits commit <app_id> <edit_id>`
