# upload-apk

Upload a new APK file to an existing edit.

## Parameters

| Name | Required | Description |
|------|----------|-------------|
| `app_id` | yes | The application ID |
| `edit_id` | yes | The edit ID (create one first with `xingu edits create`) |
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

- Response contains the new APK metadata including apk_id, version code, and version name.
- The APK is attached to the edit but the edit is not yet committed.
- Proceed to validate and commit, or make additional changes (listings, screenshots).

## Error handling

- Empty file error: Ensure the APK file is valid and non-empty.
- Auth error (exit 2): Run `xingu auth login`.
- 413 (too large): APK must be under 2GB.
- Validation error about version code: The new APK must have a higher versionCode than the live version.
- Edit not found (404): The edit_id may be stale. Get the current one with `xingu edits get <app_id>`.
- Timeout: Large APKs may need more time. Retry with `--timeout 120` or `--timeout 300`.

## Notes

- To replace an already-uploaded APK in the same edit: `xingu apks replace <app_id> <edit_id> <apk_id> --file <new_file>`
- Check what was uploaded: `xingu apks list <app_id> <edit_id>`
- For Fire TV apps, ensure your APK targets the correct Android TV/Fire TV APIs.
