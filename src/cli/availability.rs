use anyhow::Result;
use clap::Subcommand;

use super::exec;
use crate::output::OutputFormat;

#[derive(Subcommand)]
pub enum AvailabilityCommands {
    /// Get app availability and scheduling info
    Get {
        /// Application ID
        app_id: String,
        /// Edit ID
        edit_id: String,
    },
    /// Update app availability (e.g. schedule a release date)
    Update {
        /// Application ID
        app_id: String,
        /// Edit ID
        edit_id: String,
        /// JSON data for the availability update
        #[arg(long)]
        json: String,
    },
}

pub async fn run(
    cmd: &AvailabilityCommands,
    format: OutputFormat,
    dry_run: bool,
    timeout: u64,
) -> Result<()> {
    match cmd {
        AvailabilityCommands::Get { app_id, edit_id } => {
            exec::api_get(
                &format!("/applications/{app_id}/edits/{edit_id}/availability"),
                format,
                dry_run,
                timeout,
            )
            .await
        }
        AvailabilityCommands::Update {
            app_id,
            edit_id,
            json,
        } => {
            let body: serde_json::Value = serde_json::from_str(json)?;
            exec::api_put_with_etag(
                &format!("/applications/{app_id}/edits/{edit_id}/availability"),
                &body,
                format,
                dry_run,
                timeout,
            )
            .await
        }
    }
}
