---
name: manage-screenshots
description: Upload, list, or replace screenshots and icons for a locale. Use when adding or updating app store images.
depends_on: [create-edit]
---

# manage-screenshots

Upload, list, or replace screenshots for a locale and device type.

## When to use

Use when you need to add, view, or replace screenshots or icons for an app edit. For a full release that includes screenshots, this is one step in the full-release workflow.

## Parameters

| Name | Required | Description |
|------|----------|-------------|
| `app_id` | yes | The application ID |
| `edit_id` | yes | The edit ID |
| `locale` | yes | Locale code (e.g., en-US) |
| `files` | for upload/replace | Paths to image files (PNG or JPG) |
| `image_type` | no | Default: `screenshots`. Options: `screenshots`, `large-icons`, `small-icons`, `promotional-images` |

## Preconditions

- An active edit in DRAFT status must exist
- Image files must be PNG or JPG
- Screenshots: min 320px shortest side, max 3840px longest side

## Commands

### List existing screenshots

```sh
xingu images list <app_id> <edit_id> --locale <locale> --image-type screenshots
```

### Upload new screenshots (appends to existing)

```sh
xingu images upload <app_id> <edit_id> --locale <locale> --image-type screenshots --file <path>
```

Run once per file. Maximum 10 per type per locale.

### Replace all (delete existing, then upload new)

```sh
xingu images delete-all <app_id> <edit_id> --locale <locale> --image-type screenshots
xingu images upload <app_id> <edit_id> --locale <locale> --image-type screenshots --file <path1>
xingu images upload <app_id> <edit_id> --locale <locale> --image-type screenshots --file <path2>
```

## Error handling

- 415: Unsupported format. Use PNG or JPG only.
- 413: Too large. Resize to under 5MB.
- Limit reached: Delete existing screenshots first.
- Delete succeeded but upload failed: Screenshots are gone. Re-upload.
- ETag conflict (412): Retry from the beginning.

## Notes

- Image types: `screenshots` (phone/tablet), `large-icons` (512x512), `small-icons` (114x114), `promotional-images`
- Upload order = display order in the store.
- For multi-locale apps, repeat for each locale.
- Fire TV apps: include landscape screenshots (1920x1080).
