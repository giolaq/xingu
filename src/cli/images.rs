use anyhow::Result;
use clap::Subcommand;
use std::path::PathBuf;

use super::exec;
use crate::output::OutputFormat;

#[derive(Subcommand)]
pub enum ImagesCommands {
    /// List images for a given type
    List {
        /// Application ID
        app_id: String,
        /// Edit ID
        edit_id: String,
        /// Image type (e.g. screenshots, icon, etc.)
        #[arg(long)]
        image_type: String,
    },
    /// Upload an image
    Upload {
        /// Application ID
        app_id: String,
        /// Edit ID
        edit_id: String,
        /// Image type
        #[arg(long)]
        image_type: String,
        /// Path to image file
        #[arg(long)]
        file: PathBuf,
    },
    /// Delete an image
    Delete {
        /// Application ID
        app_id: String,
        /// Edit ID
        edit_id: String,
        /// Image ID to delete
        #[arg(long)]
        image_id: String,
    },
}

pub async fn run(
    cmd: &ImagesCommands,
    format: OutputFormat,
    dry_run: bool,
    timeout: u64,
) -> Result<()> {
    match cmd {
        ImagesCommands::List {
            app_id,
            edit_id,
            image_type,
        } => {
            exec::api_get(
                &format!("/applications/{app_id}/edits/{edit_id}/images/{image_type}"),
                format,
                dry_run,
                timeout,
            )
            .await
        }
        ImagesCommands::Upload {
            app_id,
            edit_id,
            image_type,
            file,
        } => {
            let content_type = exec::content_type_for_image(file);
            exec::api_upload(
                &format!("/applications/{app_id}/edits/{edit_id}/images/{image_type}/upload"),
                file,
                content_type,
                format,
                dry_run,
                timeout,
            )
            .await
        }
        ImagesCommands::Delete {
            app_id,
            edit_id,
            image_id,
        } => {
            exec::api_delete(
                &format!("/applications/{app_id}/edits/{edit_id}/images/{image_id}"),
                format,
                dry_run,
                timeout,
            )
            .await
        }
    }
}
