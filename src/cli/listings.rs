use anyhow::Result;
use clap::Subcommand;

use super::exec;
use crate::output::OutputFormat;

#[derive(Subcommand)]
pub enum ListingsCommands {
    /// List all localized listings for an edit
    List {
        /// Application ID
        app_id: String,
        /// Edit ID
        edit_id: String,
    },
    /// Get listing for a locale
    Get {
        /// Application ID
        app_id: String,
        /// Edit ID
        edit_id: String,
        /// Locale code (e.g. en-US)
        #[arg(long)]
        locale: String,
    },
    /// Update listing for a locale (requires ETag)
    Update {
        /// Application ID
        app_id: String,
        /// Edit ID
        edit_id: String,
        /// Locale code (e.g. en-US)
        #[arg(long)]
        locale: String,
        /// JSON data for the listing update
        #[arg(long)]
        json: String,
    },
    /// Delete a localized listing
    Delete {
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
    cmd: &ListingsCommands,
    format: OutputFormat,
    dry_run: bool,
    timeout: u64,
) -> Result<()> {
    match cmd {
        ListingsCommands::List { app_id, edit_id } => {
            exec::api_get(
                &format!("/applications/{app_id}/edits/{edit_id}/listings"),
                format,
                dry_run,
                timeout,
            )
            .await
        }
        ListingsCommands::Get {
            app_id,
            edit_id,
            locale,
        } => {
            exec::api_get(
                &format!("/applications/{app_id}/edits/{edit_id}/listings/{locale}"),
                format,
                dry_run,
                timeout,
            )
            .await
        }
        ListingsCommands::Update {
            app_id,
            edit_id,
            locale,
            json,
        } => {
            let body: serde_json::Value = serde_json::from_str(json)?;
            exec::api_put_with_etag(
                &format!("/applications/{app_id}/edits/{edit_id}/listings/{locale}"),
                &body,
                format,
                dry_run,
                timeout,
            )
            .await
        }
        ListingsCommands::Delete {
            app_id,
            edit_id,
            locale,
        } => {
            exec::api_delete_with_etag(
                &format!("/applications/{app_id}/edits/{edit_id}/listings/{locale}"),
                &format!("/applications/{app_id}/edits/{edit_id}/listings/{locale}"),
                format,
                dry_run,
                timeout,
            )
            .await
        }
    }
}
