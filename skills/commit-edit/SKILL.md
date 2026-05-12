# commit-edit

Commit (submit) an edit for Amazon review — makes changes live after approval.

## Parameters

| Name | Required | Description |
|------|----------|-------------|
| `app_id` | yes | The application ID |
| `edit_id` | yes | The edit ID to commit |

## Preconditions

- Edit must be in DRAFT status
- Recommended to run `validate-edit` first to catch issues
- At minimum, an APK must be attached to the edit

## Command

```sh
xingu edits commit <app_id> <edit_id>
```

## Interpreting the result

- The edit transitions to IN_REVIEW status.
- Amazon typically reviews within 1-3 business days.
- Once approved, changes go live automatically.
- You cannot modify this edit after commit — create a new one for further changes.

## Error handling

- Validation errors: Run `validate-edit` to see specific issues, fix them, then retry.
- ETag conflict (412): The edit was modified externally. Retry the command.
- Edit not found (404): The edit was deleted. Start over with `create-edit`.
- 403: The edit may already be committed or in a non-modifiable state.

## Notes

- Commit is irreversible — once submitted you cannot pull it back.
- If you realize there's an issue after commit, create a new edit with fixes.
- Always validate before committing to avoid review rejection.
