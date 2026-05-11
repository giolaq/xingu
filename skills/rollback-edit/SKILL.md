# rollback-edit

Recover from a stuck or failed edit state — diagnose and clean up.

## Parameters

| Name | Required | Description |
|------|----------|-------------|
| `app_id` | yes | The application ID |

## Preconditions

- Auth configured (`xingu auth login` completed)

## Workflow

### Step 1 — Check current state

```sh
xingu +status <app_id>
```

- No active edit: Nothing to roll back. App is in a clean state.
- Edit in DRAFT status: Can be safely deleted (step 2).
- Edit in IN_REVIEW: Cannot delete — must wait for Amazon review to complete.
- Edit shows errors or unexpected state: Proceed to delete.

### Step 2 — Delete the draft edit

Extract `edit_id` from the status response's `activeEdit.id` field, then:

```sh
xingu edits delete <app_id> <edit_id>
```

On success: Edit deleted. App is back to its last published state. You can now start fresh with `create-edit`.

### Step 3 — If delete fails, investigate

```sh
xingu edits get <app_id>
xingu edits get-previous <app_id>
```

- 403: Edit is not in a deletable state (may be IN_REVIEW or already committed).
- ETag conflict (412): Retry the delete command.
- 404: Edit was already deleted by another process.

## Notes

- This skill is for recovery only — use when an edit is in a bad state.
- Common scenarios: failed upload left a dangling edit, validation errors you can't fix, want to start over.
- If the edit is IN_REVIEW and you need to cancel, contact Amazon developer support.
- After rollback, the app remains in its last published state — nothing is lost from production.
