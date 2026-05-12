# update-listing

Update store listing metadata for a specific locale within an active edit.

## Parameters

| Name | Required | Description |
|------|----------|-------------|
| `app_id` | yes | The application ID |
| `locale` | yes | Locale code (e.g., en-US, de-DE, ja-JP) |
| `title` | no | App title (max 250 characters) |
| `description` | no | Full description (max 4000 characters) |
| `short_description` | no | Short description shown in search (max 1200 characters) |
| `recent_changes` | no | What's new text for this version (max 4000 characters) |

At least one of the optional fields must be provided.

## Preconditions

- An active edit must exist in DRAFT status

## Command

```sh
xingu +update-listing <app_id> --locale <locale> \
  --title "My App" \
  --description "Full description here" \
  --short-description "Brief summary" \
  --recent-changes "Bug fixes and improvements"
```

Only include the flags you want to change. Omitted fields keep their current values.

## Interpreting the result

- Response contains the updated listing object with all fields.
- Changes are saved to the draft edit but NOT yet live.
- Commit the edit when all changes are complete.

## Error handling

- No active edit: Create one first with `xingu edits create <app_id>`.
- 404 on locale: The locale may not be enabled. Check available locales in the Developer Console.
- ETag conflict (412): Another process modified the listing. Retry the command.
- Validation error: Check character limits — title ≤250, short_description ≤1200, description ≤4000.

## Notes

- Common locales: en-US, en-GB, de-DE, fr-FR, es-ES, it-IT, ja-JP, pt-BR, zh-CN, hi-IN.
- To update multiple locales, run this once per locale.
- After updating, use `validate-edit` to check for issues before committing.
