---
name: troubleshoot-validation
description: Diagnose and fix validation errors returned by validate-edit. Use when validation fails and you need to understand what's wrong.
depends_on: [validate-edit]
---

# troubleshoot-validation

Diagnose and fix validation errors. Run after `xingu edits validate` reports issues.

## When to use

Use when `xingu edits validate` or `xingu edits commit` returned validation errors and you need to diagnose and fix them.

## Parameters

| Name | Required | Description |
|------|----------|-------------|
| `app_id` | yes | The application ID |
| `edit_id` | yes | The edit ID with validation issues |

## Step 1 — Get current errors

```sh
xingu edits validate <app_id> <edit_id>
```

## Step 2 — Diagnose and fix each error

| Error pattern | Diagnosis | Fix |
|---------------|-----------|-----|
| APK missing / no apk / apk required | No APK attached | `xingu apks upload <app_id> <edit_id> --file <path>` |
| version code / versionCode | Version not incremented | Rebuild APK with higher versionCode, re-upload |
| listing missing / locale required | Required locale missing | `xingu +update-listing <app_id> --locale <locale> --title "..."` |
| screenshot missing / image required | Screenshots not uploaded | `xingu images upload <app_id> <edit_id> --locale en-US --image-type screenshots --file <path>` |
| icon missing / small icon / large icon | App icon missing | `xingu images upload <app_id> <edit_id> --locale en-US --image-type small-icons --file icon_114.png` |
| content rating / rating missing | Questionnaire not completed | Developer Console web UI only (not available via API) |
| target audience / age rating / COPPA | Not configured | Developer Console under "Content Rating" |
| privacy policy / policy url | URL required | Developer Console under app details |
| category / genre | Not selected | Developer Console |

## Step 3 — Re-validate

```sh
xingu edits validate <app_id> <edit_id>
```

- All clear: `xingu edits commit <app_id> <edit_id>`
- Still errors: repeat step 2 for remaining issues.

## Notes

- Some errors can only be fixed in the Developer Console web UI.
- Validation is idempotent. Fix and re-validate as many times as needed.
