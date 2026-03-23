use anyhow::Result;
use clap::Subcommand;

use super::exec;
use crate::output::OutputFormat;

#[derive(Subcommand)]
pub enum DetailsCommands {
    /// Get app details
    Get {
        /// Application ID
        app_id: String,
        /// Edit ID
        edit_id: String,
    },
    /// Update app details
    Update {
        /// Application ID
        app_id: String,
        /// Edit ID
        edit_id: String,
        /// JSON data for the details update
        #[arg(long)]
        json: String,
    },
}

pub async fn run(
    cmd: &DetailsCommands,
    format: OutputFormat,
    dry_run: bool,
    timeout: u64,
) -> Result<()> {
    match cmd {
        DetailsCommands::Get { app_id, edit_id } => {
            exec::api_get(
                &format!("/applications/{app_id}/edits/{edit_id}/details"),
                format,
                dry_run,
                timeout,
            )
            .await
        }
        DetailsCommands::Update {
            app_id,
            edit_id,
            json,
        } => {
            let body: serde_json::Value = serde_json::from_str(json)?;
            exec::api_put_with_etag(
                &format!("/applications/{app_id}/edits/{edit_id}/details"),
                &body,
                format,
                dry_run,
                timeout,
            )
            .await
        }
    }
}
