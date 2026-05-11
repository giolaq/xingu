# troubleshoot-validation

Diagnose and fix validation errors — run after validate-edit reports issues.

## Parameters

| Name | Required | Description |
|------|----------|-------------|
| `app_id` | yes | The application ID |
| `edit_id` | yes | The edit ID with validation issues |

## Preconditions

- A `validate-edit` or `commit-edit` has returned validation errors

## Step 1 — Get current errors

```sh
xingu edits validate <app_id> <edit_id>
```

## Step 2 — Diagnose and fix each error

| Error pattern | Diagnosis | Fix |
|---------------|-----------|-----|
| APK missing / no apk / apk required | No APK attached to the edit | Use `upload-apk` to upload one |
| version code / versionCode | APK version code not higher than live | Rebuild APK with incremented versionCode, re-upload |
| listing missing / locale required | Required locale listing is missing | Use `update-listing` to add title and descriptions |
| screenshot missing / image required | Screenshots required but not uploaded | Use `manage-screenshots` to upload at least 3 |
| icon missing / small icon / large icon | App icon is missing | `xingu images upload <app_id> <edit_id> --locale en-US --image-type small-icons --file icon_114.png` and `--image-type large-icons --file icon_512.png` |
| content rating / rating missing | Content rating questionnaire not completed | Must be done in the Developer Console web UI — cannot be set via API |
| target audience / age rating / COPPA | Target audience not configured | Configure in Developer Console under "Content Rating" |
| privacy policy / policy url | Privacy policy URL required | Set in Developer Console under app details |
| category / genre | App category not selected | Set in Developer Console |

## Step 3 — Re-validate

```sh
xingu edits validate <app_id> <edit_id>
```

- All clear: proceed with `commit-edit`.
- Still errors: repeat diagnosis for remaining issues.

## Notes

- Some errors can only be fixed in the Developer Console web UI (content rating, categories, privacy policy).
- Validation is idempotent — safe to run repeatedly.
- Validation errors don't damage the edit — fix and re-validate as many times as needed.
