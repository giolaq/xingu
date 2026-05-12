---
name: delete-edit
description: Delete a draft edit to discard all changes or recover from a stuck state. Use when you need to start over or clean up a failed operation.
depends_on: []
---

# delete-edit

Delete a draft edit. Use to cancel changes or recover from a stuck state.

## When to use

Use when an edit is in a bad state (failed upload, inconsistent, want to start over). For diagnosing the problem before deleting, use `rollback-edit` which checks state first.

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
- Create a fresh edit with `xingu edits create <app_id>`.

## Error handling

- 404: The edit doesn't exist. Already deleted or committed.
- 403: Not in DRAFT status (may be IN_REVIEW). Cannot delete edits under review.
- ETag conflict (412): Retry the command.

## Notes

- Destructive: all uncommitted work in the edit is lost.
- After deleting, the app remains in its last published state.
