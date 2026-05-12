# check-targeting

View and interpret device targeting for an APK.

## Parameters

| Name | Required | Description |
|------|----------|-------------|
| `app_id` | yes | The application ID |
| `edit_id` | yes | The edit ID |
| `apk_id` | yes | The APK ID |

## Preconditions

- An active edit must exist (use `check-status` to verify)
- An APK must already be uploaded to the edit

## Command

```sh
xingu targeting get <app_id> <edit_id> <apk_id>
```

## Interpreting the result

- Response contains a list of supported device types and ASIN targets.
- Fire TV devices appear as device types like: FireTV, FireTVStick, FireTVCube.
- Fire tablets appear as: KindleFire, KindleFireHD, etc.
- If the list is empty, the APK targets all compatible devices by default.

## Error handling

- 404: Either the edit_id or apk_id is invalid. Use `xingu edits get <app_id>` to get the current edit, then `xingu apks list <app_id> <edit_id>` to get APK IDs.
- 403 with errorCode: The edit may be locked (already committed). Create a new edit.

## Notes

- Device targeting is optional — most apps don't need to restrict devices.
- To update targeting: `xingu targeting update <app_id> <edit_id> <apk_id> --json '<targeting_json>'`
- Targeting changes require the edit to be re-validated before commit.
