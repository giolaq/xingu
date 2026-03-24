use anyhow::Result;
use clap::Subcommand;

use super::exec;
use crate::output::OutputFormat;

#[derive(Subcommand)]
pub enum TargetingCommands {
    /// Get device targeting for an APK
    Get {
        /// Application ID
        app_id: String,
        /// Edit ID
        edit_id: String,
        /// APK ID
        apk_id: String,
    },
    /// Update device targeting for an APK
    Update {
        /// Application ID
        app_id: String,
        /// Edit ID
        edit_id: String,
        /// APK ID
        apk_id: String,
        /// JSON data for targeting update
        #[arg(long)]
        json: String,
    },
}

pub async fn run(
    cmd: &TargetingCommands,
    format: OutputFormat,
    dry_run: bool,
    timeout: u64,
) -> Result<()> {
    let path = |app_id: &str, edit_id: &str, apk_id: &str| {
        format!("/applications/{app_id}/edits/{edit_id}/apks/{apk_id}/targeting")
    };

    match cmd {
        TargetingCommands::Get {
            app_id,
            edit_id,
            apk_id,
        } => exec::api_get(&path(app_id, edit_id, apk_id), format, dry_run, timeout).await,
        TargetingCommands::Update {
            app_id,
            edit_id,
            apk_id,
            json,
        } => {
            let body: serde_json::Value = serde_json::from_str(json)?;
            exec::api_put_with_etag(
                &path(app_id, edit_id, apk_id),
                &body,
                format,
                dry_run,
                timeout,
            )
            .await
        }
    }
}
