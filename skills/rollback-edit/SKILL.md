---
name: rollback-edit
description: Diagnose and recover from a stuck or failed edit state. Use when something went wrong and you need to get back to a clean state.
depends_on: [check-status, delete-edit]
---

# rollback-edit

Recover from a stuck or failed edit state. Diagnose and clean up.

## When to use

Use when an edit is in a bad state and you need to recover. If you already know the edit_id and just want to delete, use `xingu edits delete` directly.

## Parameters

| Name | Required | Description |
|------|----------|-------------|
| `app_id` | yes | The application ID |

## Workflow

### Step 1 — Check current state

```sh
xingu +status <app_id>
```

- No active edit: nothing to roll back. App is clean.
- Edit in DRAFT: can be deleted (step 2).
- Edit in IN_REVIEW: cannot delete. Wait for Amazon review.

### Step 2 — Delete the draft edit

Extract `edit_id` from the response's `activeEdit.id` field:

```sh
xingu edits delete <app_id> <edit_id>
```

On success: app is back to its last published state.

### Step 3 — If delete fails, investigate

```sh
xingu edits get <app_id>
xingu edits get-previous <app_id>
```

- 403: Not deletable (IN_REVIEW or already committed).
- ETag conflict (412): Retry the delete.
- 404: Already deleted by another process.

## Notes

- For recovery only. Common scenarios: failed upload, stuck edit, want to start over.
- If IN_REVIEW and you need to cancel, contact Amazon developer support.
- After rollback, the app remains in its last published state. Nothing is lost from production.
