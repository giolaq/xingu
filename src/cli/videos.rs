use anyhow::Result;
use clap::Subcommand;
use std::path::PathBuf;

use super::exec;
use crate::output::OutputFormat;

#[derive(Subcommand)]
pub enum VideosCommands {
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
}

pub async fn run(
    cmd: &VideosCommands,
    format: OutputFormat,
    dry_run: bool,
    timeout: u64,
) -> Result<()> {
    match cmd {
        VideosCommands::Upload {
            app_id,
            edit_id,
            locale,
            file,
        } => {
            exec::api_upload(
                &format!("/applications/{app_id}/edits/{edit_id}/listings/{locale}/videos/upload"),
                file,
                "video/mp4",
                format,
                dry_run,
                timeout,
            )
            .await
        }
    }
}
