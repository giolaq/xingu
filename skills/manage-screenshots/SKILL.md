# manage-screenshots

Upload, list, or replace screenshots for a locale and device type.

## Parameters

| Name | Required | Description |
|------|----------|-------------|
| `app_id` | yes | The application ID |
| `edit_id` | yes | The edit ID |
| `locale` | yes | Locale code (e.g., en-US) |
| `action` | yes | One of: `list`, `upload`, `replace-all` |
| `files` | for upload/replace-all | Paths to screenshot image files (PNG or JPG) |
| `image_type` | no | Default: `screenshots`. Options: `screenshots`, `large-icons`, `small-icons`, `promotional-images` |

## Preconditions

- An active edit in DRAFT status must exist
- For upload/replace-all: image files must exist and be PNG or JPG
- Screenshot requirements: min 320px shortest side, max 3840px longest side

## Commands

### List existing screenshots

```sh
xingu images list <app_id> <edit_id> --locale <locale> --image-type screenshots
```

Returns an array of image objects with IDs and URLs. Empty array means none uploaded.

### Upload new screenshots (appends to existing)

```sh
xingu images upload <app_id> <edit_id> --locale <locale> --image-type screenshots --file <path>
```

Run once per file. Screenshots are ordered by upload sequence. Maximum 10 per type per locale.

### Replace all screenshots (delete existing, then upload new)

```sh
xingu images delete-all <app_id> <edit_id> --locale <locale> --image-type screenshots
xingu images upload <app_id> <edit_id> --locale <locale> --image-type screenshots --file <path1>
xingu images upload <app_id> <edit_id> --locale <locale> --image-type screenshots --file <path2>
```

Delete all first, then upload each new file.

## Error handling

- 415: Unsupported image format. Use PNG or JPG only.
- 413: Image too large. Resize to under 5MB.
- Limit reached: Delete existing screenshots first, then upload.
- If delete succeeds but upload fails: screenshots are gone. Re-upload manually.
- ETag conflict on delete (412): Retry from the beginning.

## Notes

- Image types: `screenshots` (phone/tablet), `large-icons` (512x512), `small-icons` (114x114), `promotional-images`
- Upload order determines display order in the store.
- For multi-locale apps, repeat for each locale.
- Fire TV apps should include landscape screenshots (1920x1080 recommended).
