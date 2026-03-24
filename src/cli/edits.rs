use anyhow::Result;
use clap::Subcommand;

use super::exec;
use crate::output::OutputFormat;

#[derive(Subcommand)]
pub enum EditsCommands {
    /// Create a new edit for an app
    Create {
        /// Application ID
        app_id: String,
    },
    /// Get the active edit for an app
    Get {
        /// Application ID
        app_id: String,
    },
    /// Get the previous (live) edit for an app
    GetPrevious {
        /// Application ID
        app_id: String,
    },
    /// Validate an edit without committing
    Validate {
        /// Application ID
        app_id: String,
        /// Edit ID
        edit_id: String,
    },
    /// Delete a draft edit
    Delete {
        /// Application ID
        app_id: String,
        /// Edit ID
        edit_id: String,
    },
    /// Commit (submit) an edit
    Commit {
        /// Application ID
        app_id: String,
        /// Edit ID
        edit_id: String,
    },
}

pub async fn run(
    cmd: &EditsCommands,
    format: OutputFormat,
    dry_run: bool,
    timeout: u64,
) -> Result<()> {
    match cmd {
        EditsCommands::Create { app_id } => {
            exec::api_post(
                &format!("/applications/{app_id}/edits"),
                &serde_json::json!({}),
                format,
                dry_run,
                timeout,
            )
            .await
        }
        EditsCommands::Get { app_id } => {
            exec::api_get(
                &format!("/applications/{app_id}/edits"),
                format,
                dry_run,
                timeout,
            )
            .await
        }
        EditsCommands::GetPrevious { app_id } => {
            exec::api_get(
                &format!("/applications/{app_id}/edits/previous"),
                format,
                dry_run,
                timeout,
            )
            .await
        }
        EditsCommands::Validate { app_id, edit_id } => {
            exec::api_post_with_etag(
                &format!("/applications/{app_id}/edits/{edit_id}/validate"),
                &format!("/applications/{app_id}/edits"),
                format,
                dry_run,
                timeout,
            )
            .await
        }
        EditsCommands::Delete { app_id, edit_id } => {
            exec::api_delete_with_etag(
                &format!("/applications/{app_id}/edits/{edit_id}"),
                &format!("/applications/{app_id}/edits"),
                format,
                dry_run,
                timeout,
            )
            .await
        }
        EditsCommands::Commit { app_id, edit_id } => {
            exec::api_post_with_etag(
                &format!("/applications/{app_id}/edits/{edit_id}/commit"),
                &format!("/applications/{app_id}/edits"),
                format,
                dry_run,
                timeout,
            )
            .await
        }
    }
}
