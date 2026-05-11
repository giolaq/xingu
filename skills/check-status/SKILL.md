# check-status

Get app info, active edit status, and suggest next actions.

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

- No active edit → use `create-edit` to start one
- Edit in DRAFT → use `upload-apk`, `update-listing`, or `manage-screenshots`
- Edit in DRAFT and ready → use `commit-edit` to submit
- Edit in IN_REVIEW → wait for Amazon review or use `delete-edit` to cancel
