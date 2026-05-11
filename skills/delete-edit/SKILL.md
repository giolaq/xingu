# delete-edit

Delete a draft edit — use to cancel changes or recover from a stuck state.

## Parameters

| Name | Required | Description |
|------|----------|-------------|
| `app_id` | yes | The application ID |
| `edit_id` | yes | The edit ID to delete |

## Preconditions

- Edit must exist and be in DRAFT status
- Edits in IN_REVIEW or LIVE status cannot be deleted

## Command

```sh
xingu edits delete <app_id> <edit_id>
```

## Interpreting the result

- The edit and all its changes (APKs, listing updates, screenshots) are discarded.
- You can now create a fresh edit with `create-edit`.

## Error handling

- 404: The edit doesn't exist — it may have already been deleted or committed.
- 403: The edit is not in DRAFT status (may be IN_REVIEW). Cannot delete edits under review.
- ETag conflict (412): Retry the command.

## Notes

- This is a destructive operation — all uncommitted work in the edit is lost.
- Use this to recover when an edit is in a bad state (failed upload, inconsistent state, want to start over).
- After deleting, create a new edit to start over.
