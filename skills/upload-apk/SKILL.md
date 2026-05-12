---
name: upload-apk
description: Upload a new APK file to an existing edit. Use after creating an edit when you have a new build to publish.
depends_on: [create-edit]
---

# upload-apk

Upload a new APK file to an existing edit.

## When to use

Use when you have an APK ready and an active edit in DRAFT status. If you want a one-step flow (create + upload + commit), use `xingu +publish` instead.

## Parameters

| Name | Required | Description |
|------|----------|-------------|
| `app_id` | yes | The application ID |
| `edit_id` | yes | The edit ID (get from `xingu edits get <app_id>`) |
| `file` | yes | Path to the APK file |

## Preconditions

- An active edit in DRAFT status must exist
- The APK file must exist, be non-empty, and be a valid .apk
- The APK version code must be higher than the currently live version

## Command

```sh
xingu apks upload <app_id> <edit_id> --file <path_to_apk>
```

## Interpreting the result

- Response contains the new APK metadata: apk_id, version code, version name.
- The APK is attached to the edit but the edit is not yet committed.

## Error handling

- Empty file error: Ensure the APK file is valid and non-empty.
- Auth error (exit 2): Run `xingu auth login`.
- 413 (too large): APK must be under 2GB.
- Version code error: The new APK must have a higher versionCode than the live version.
- Edit not found (404): Run `xingu edits get <app_id>` to get the current edit_id.
- Timeout: Retry with `--timeout 120` or `--timeout 300` for large files.

## Notes

- To replace an uploaded APK: `xingu apks replace <app_id> <edit_id> <apk_id> --file <new_file>`
- Check uploads: `xingu apks list <app_id> <edit_id>`
- For Fire TV apps, ensure your APK targets Android TV/Fire TV APIs.
