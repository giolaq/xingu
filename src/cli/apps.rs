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
        AppsCommands::List => {
            anyhow::bail!(
                "The Amazon App Submission API does not support listing all apps.\n\
                 Get your App ID from the Developer Console:\n\
                 1. Go to https://developer.amazon.com/dashboard\n\
                 2. Navigate to My Apps → select your app\n\
                 3. Find the App ID under Additional Information\n\n\
                 Then use: xingu apps get <app-id>"
            );
        }
        AppsCommands::Get { app_id } => {
            // The API has no GET /applications/{appId} endpoint.
            // Fetch the active edit as a proxy to confirm the app exists.
            exec::api_get(&format!("/applications/{app_id}/edits"), format, dry_run, timeout).await
        }
    }
}
