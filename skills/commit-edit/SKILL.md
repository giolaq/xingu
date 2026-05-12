---
name: commit-edit
description: Submit an edit for Amazon review. Use after validate-edit passes to make changes live. Irreversible.
depends_on: [validate-edit]
---

# commit-edit

Commit (submit) an edit for Amazon review. Makes changes live after approval.

## When to use

Use after `xingu edits validate` passes with no errors. If you haven't validated yet, do that first. For a one-step publish, use `xingu +publish` instead.

## Parameters

| Name | Required | Description |
|------|----------|-------------|
| `app_id` | yes | The application ID |
| `edit_id` | yes | The edit ID to commit |

## Preconditions

- Edit must be in DRAFT status
- Run `xingu edits validate <app_id> <edit_id>` first to catch issues
- At minimum, an APK must be attached to the edit

## Command

```sh
xingu edits commit <app_id> <edit_id>
```

## Interpreting the result

- The edit transitions to IN_REVIEW status.
- Amazon typically reviews within 1-3 business days.
- Once approved, changes go live automatically.
- You cannot modify this edit after commit. Create a new one for further changes.

## Error handling

- Validation errors: Run `xingu edits validate <app_id> <edit_id>` to see issues, fix them, retry.
- ETag conflict (412): Retry the command.
- Edit not found (404): The edit was deleted. Run `xingu edits create <app_id>`.
- 403: The edit may already be committed.

## Notes

- Commit is irreversible. Once submitted you cannot pull it back.
- Always validate before committing to avoid review rejection.
