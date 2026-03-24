use anyhow::Result;
use clap::Subcommand;
use std::path::PathBuf;

use super::exec;
use crate::output::OutputFormat;

#[derive(Subcommand)]
pub enum ImagesCommands {
    /// List images for a locale and image type
    List {
        /// Application ID
        app_id: String,
        /// Edit ID
        edit_id: String,
        /// Locale code (e.g. en-US)
        #[arg(long)]
        locale: String,
        /// Image type (e.g. screenshots, small-icons, large-icons, etc.)
        #[arg(long)]
        image_type: String,
    },
    /// Upload an image
    Upload {
        /// Application ID
        app_id: String,
        /// Edit ID
        edit_id: String,
        /// Locale code (e.g. en-US)
        #[arg(long)]
        locale: String,
        /// Image type
        #[arg(long)]
        image_type: String,
        /// Path to image file
        #[arg(long)]
        file: PathBuf,
    },
    /// Delete a specific image
    Delete {
        /// Application ID
        app_id: String,
        /// Edit ID
        edit_id: String,
        /// Locale code (e.g. en-US)
        #[arg(long)]
        locale: String,
        /// Image type
        #[arg(long)]
        image_type: String,
        /// Image asset ID to delete
        #[arg(long)]
        image_id: String,
    },
    /// Delete all images of a given type for a locale
    DeleteAll {
        /// Application ID
        app_id: String,
        /// Edit ID
        edit_id: String,
        /// Locale code (e.g. en-US)
        #[arg(long)]
        locale: String,
        /// Image type
        #[arg(long)]
        image_type: String,
    },
}

pub async fn run(
    cmd: &ImagesCommands,
    format: OutputFormat,
    dry_run: bool,
    timeout: u64,
) -> Result<()> {
    let base = |app_id: &str, edit_id: &str, locale: &str, image_type: &str| {
        format!("/applications/{app_id}/edits/{edit_id}/listings/{locale}/{image_type}")
    };

    match cmd {
        ImagesCommands::List {
            app_id,
            edit_id,
            locale,
            image_type,
        } => exec::api_get(&base(app_id, edit_id, locale, image_type), format, dry_run, timeout).await,
        ImagesCommands::Upload {
            app_id,
            edit_id,
            locale,
            image_type,
            file,
        } => {
            let ct = exec::content_type_for_image(file);
            exec::api_upload(
                &format!("{}/upload", base(app_id, edit_id, locale, image_type)),
                file,
                ct,
                format,
                dry_run,
                timeout,
            )
            .await
        }
        ImagesCommands::Delete {
            app_id,
            edit_id,
            locale,
            image_type,
            image_id,
        } => {
            let path = format!("{}/{image_id}", base(app_id, edit_id, locale, image_type));
            exec::api_delete_with_etag(&path, &base(app_id, edit_id, locale, image_type), format, dry_run, timeout).await
        }
        ImagesCommands::DeleteAll {
            app_id,
            edit_id,
            locale,
            image_type,
        } => {
            exec::api_delete_with_etag(
                &base(app_id, edit_id, locale, image_type),
                &base(app_id, edit_id, locale, image_type),
                format,
                dry_run,
                timeout,
            )
            .await
        }
    }
}
