---
name: check-status
description: Get app info and active edit status. Use this first to understand what state an app is in before making any changes.
depends_on: []
---

# check-status

Get app info, active edit status, and suggest next actions.

## When to use

Use this as your starting point before any operation. If you don't know whether an app has an active edit, run this first. For actual publishing, prefer `xingu +publish` (fast) or the full-release workflow.

## Parameters

| Name | Required | Description |
|------|----------|-------------|
| `app_id` | yes | The application ID |

## Preconditions

- Auth configured (`xingu auth login` completed)

## Command

```sh
xingu +status <app_id>
```

## Interpreting the result

- Edit with status **DRAFT**: the app has an open edit ready for changes.
- Edit with status **IN_REVIEW**: submission is pending Amazon review.
- No active edit (empty or null): you need to create one before making changes.

## Error handling

- Exit code 2 (auth): Run `xingu auth login` to refresh credentials.
- Exit code 4 (network): Check connectivity, retry with `--timeout 60`.
- API error 404: The app_id may be incorrect. Verify it in the Amazon Developer Console.

## Next steps

- No active edit → `xingu edits create <app_id>`
- Edit in DRAFT → `xingu apks upload`, `xingu +update-listing`, or `xingu images upload`
- Edit in DRAFT and ready → `xingu edits commit <app_id> <edit_id>`
- Edit in IN_REVIEW → wait for Amazon review or `xingu edits delete <app_id> <edit_id>`
