# create-edit

Create a new draft edit for an app — required before any changes.

## Parameters

| Name | Required | Description |
|------|----------|-------------|
| `app_id` | yes | The application ID |

## Preconditions

- Auth configured (`xingu auth login` completed)
- No active edit should already exist for this app (only one edit at a time is allowed)

## Command

```sh
xingu edits create <app_id>
```

## Interpreting the result

- Response contains the edit object with an `id` field — save this for subsequent commands.
- The edit starts in DRAFT status and copies the current live app state.
- You can now upload APKs, update listings, add screenshots, etc.

## Error handling

- 409 Conflict: An edit already exists. Get it with `xingu edits get <app_id>`, or delete it with `xingu edits delete <app_id> <edit_id>` to start fresh.
- Auth error (exit 2): Run `xingu auth login`.
- 404: The app_id is invalid. Verify it in the Amazon Developer Console.

## Next steps

- Upload an APK → use `upload-apk`
- Update store listing → use `update-listing`
- Upload screenshots → use `manage-screenshots`
- When done → use `validate-edit` then `commit-edit`
