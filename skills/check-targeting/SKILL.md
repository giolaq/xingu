---
name: check-targeting
description: View device targeting for an APK. Use when you need to check which Fire TV or tablet devices an APK supports.
depends_on: [check-status]
---

# check-targeting

View and interpret device targeting for an APK.

## When to use

Use after uploading an APK when you need to verify which devices it targets. Most apps don't need this unless they restrict to specific Fire TV or tablet models.

## Parameters

| Name | Required | Description |
|------|----------|-------------|
| `app_id` | yes | The application ID |
| `edit_id` | yes | The edit ID |
| `apk_id` | yes | The APK ID |

## Preconditions

- An active edit must exist (run `xingu +status <app_id>` to verify)
- An APK must already be uploaded to the edit

## Command

```sh
xingu targeting get <app_id> <edit_id> <apk_id>
```

## Interpreting the result

- Response contains a list of supported device types and ASIN targets.
- Fire TV devices: FireTV, FireTVStick, FireTVCube.
- Fire tablets: KindleFire, KindleFireHD, etc.
- Empty list means the APK targets all compatible devices by default.

## Error handling

- 404: Invalid edit_id or apk_id. Run `xingu edits get <app_id>` then `xingu apks list <app_id> <edit_id>` to get correct IDs.
- 403 with errorCode: The edit may be locked (already committed). Create a new edit with `xingu edits create <app_id>`.

## Notes

- Device targeting is optional. Most apps don't need to restrict devices.
- To update: `xingu targeting update <app_id> <edit_id> <apk_id> --json '<targeting_json>'`
- Targeting changes require re-validation: `xingu edits validate <app_id> <edit_id>`
