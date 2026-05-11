# validate-edit

Validate an edit before committing — catches errors early without submitting.

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

- No errors: The edit is ready to commit with `xingu edits commit <app_id> <edit_id>`.
- Warnings: Non-blocking issues are reported but won't prevent submission.
- Errors: Must be fixed before committing. Use `troubleshoot-validation` for guidance.

## Error handling

- ETag conflict (412): Retry — another process may have modified the edit.
- Edit not found (404): The edit was deleted or already committed.

## Common validation failures

- Missing APK: Upload one with `upload-apk`.
- Missing listing for required locale: Add via `update-listing`.
- Missing screenshots: Upload with `manage-screenshots`.
- Icon missing: Upload with `xingu images upload` (image_type: `small-icons` or `large-icons`).
- Version code not incremented: Rebuild APK with higher versionCode.
- Content rating missing: Must be set in the Developer Console UI.

## Notes

- Validation is idempotent — safe to run multiple times.
- Always validate before committing to avoid failed submissions.
- Some issues (like content rating) can only be fixed in the web console.
- For detailed error diagnosis, use the `troubleshoot-validation` skill.
