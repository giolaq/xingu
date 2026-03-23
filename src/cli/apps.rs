use anyhow::Result;
use clap::Subcommand;

use super::exec;
use crate::output::OutputFormat;

#[derive(Subcommand)]
pub enum AppsCommands {
    /// List all apps in your developer account
    List,
    /// Get details for a specific app
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
        AppsCommands::List => exec::api_get("/applications", format, dry_run, timeout).await,
        AppsCommands::Get { app_id } => {
            exec::api_get(&format!("/applications/{app_id}"), format, dry_run, timeout).await
        }
    }
}
