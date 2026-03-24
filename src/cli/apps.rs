use anyhow::Result;
use clap::Subcommand;

use crate::output::OutputFormat;

#[derive(Subcommand)]
pub enum AppsCommands {
    /// Get active edit for an app (the API has no standalone app-info endpoint)
    Get {
        /// Application ID
        app_id: String,
    },
}

pub async fn run(
    cmd: &AppsCommands,
    format: OutputFormat,
    dry_run: bool,
    timeout: u64,
) -> Result<()> {
    match cmd {
        AppsCommands::Get { app_id } => {
            super::exec::api_get(
                &format!("/applications/{app_id}/edits"),
                format,
                dry_run,
                timeout,
            )
            .await
        }
    }
}
