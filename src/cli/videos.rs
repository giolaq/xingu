use anyhow::Result;
use clap::Subcommand;
use std::path::PathBuf;

use super::exec;
use crate::output::OutputFormat;

#[derive(Subcommand)]
pub enum VideosCommands {
    /// List videos for a locale
    List {
        /// Application ID
        app_id: String,
        /// Edit ID
        edit_id: String,
        /// Locale code (e.g. en-US)
        #[arg(long)]
        locale: String,
    },
    /// Upload a video for a locale
    Upload {
        /// Application ID
        app_id: String,
        /// Edit ID
        edit_id: String,
        /// Locale code (e.g. en-US)
        #[arg(long)]
        locale: String,
        /// Path to video file
        #[arg(long)]
        file: PathBuf,
    },
    /// Delete a specific video
    Delete {
        /// Application ID
        app_id: String,
        /// Edit ID
        edit_id: String,
        /// Locale code (e.g. en-US)
        #[arg(long)]
        locale: String,
        /// Video asset ID
        #[arg(long)]
        video_id: String,
    },
    /// Delete all videos for a locale
    DeleteAll {
        /// Application ID
        app_id: String,
        /// Edit ID
        edit_id: String,
        /// Locale code (e.g. en-US)
        #[arg(long)]
        locale: String,
    },
}

pub async fn run(
    cmd: &VideosCommands,
    format: OutputFormat,
    dry_run: bool,
    timeout: u64,
) -> Result<()> {
    let base = |app_id: &str, edit_id: &str, locale: &str| {
        format!("/applications/{app_id}/edits/{edit_id}/listings/{locale}/videos")
    };

    match cmd {
        VideosCommands::List {
            app_id,
            edit_id,
            locale,
        } => exec::api_get(&base(app_id, edit_id, locale), format, dry_run, timeout).await,
        VideosCommands::Upload {
            app_id,
            edit_id,
            locale,
            file,
        } => {
            let etag_path = base(app_id, edit_id, locale);
            exec::api_upload_with_etag(
                &format!("{}/upload", etag_path),
                &etag_path,
                file,
                "video/mp4",
                format,
                dry_run,
                timeout,
            )
            .await
        }
        VideosCommands::Delete {
            app_id,
            edit_id,
            locale,
            video_id,
        } => {
            exec::api_delete_with_etag(
                &format!("{}/{video_id}", base(app_id, edit_id, locale)),
                &base(app_id, edit_id, locale),
                format,
                dry_run,
                timeout,
            )
            .await
        }
        VideosCommands::DeleteAll {
            app_id,
            edit_id,
            locale,
        } => {
            exec::api_delete_with_etag(
                &base(app_id, edit_id, locale),
                &base(app_id, edit_id, locale),
                format,
                dry_run,
                timeout,
            )
            .await
        }
    }
}
