# publish-app

Create an edit, upload an APK, and commit — the fastest path to publish.

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
- The app is now submitted for Amazon review.
- Review typically takes 1-3 business days.

## Error handling

- Auth error (exit 2): Run `xingu auth login` and retry.
- Edit already exists: Delete the existing edit first with `xingu edits delete <app_id> <edit_id>`, or use the individual `create-edit` + `upload-apk` + `commit-edit` flow.
- Upload failed: Check file path, ensure APK is valid and under 2GB.
- Commit failed: The command auto-rolls back (deletes the edit). Check validation errors — the APK may be missing required metadata.
- If rollback itself fails, the dangling edit ID is printed. Delete manually with `xingu edits delete <app_id> <edit_id>`.

## Notes

- This is a convenience wrapper. For more control (updating listings, adding screenshots), use the individual skills in sequence.
- Use `--dry-run` to preview what API calls will be made without executing.
- The APK version code must be higher than the currently live version.
