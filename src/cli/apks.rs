use anyhow::Result;
use clap::Subcommand;
use std::path::PathBuf;

use super::exec;
use crate::output::OutputFormat;

#[derive(Subcommand)]
pub enum ApksCommands {
    /// List APKs in an edit
    List {
        /// Application ID
        app_id: String,
        /// Edit ID
        edit_id: String,
    },
    /// Get details of a specific APK
    Get {
        /// Application ID
        app_id: String,
        /// Edit ID
        edit_id: String,
        /// APK ID
        apk_id: String,
    },
    /// Upload a new APK
    Upload {
        /// Application ID
        app_id: String,
        /// Edit ID
        edit_id: String,
        /// Path to APK file
        #[arg(long)]
        file: PathBuf,
    },
    /// Replace an existing APK (preserves device targeting)
    Replace {
        /// Application ID
        app_id: String,
        /// Edit ID
        edit_id: String,
        /// APK ID to replace
        apk_id: String,
        /// Path to new APK file
        #[arg(long)]
        file: PathBuf,
    },
    /// Delete an APK from an edit
    Delete {
        /// Application ID
        app_id: String,
        /// Edit ID
        edit_id: String,
        /// APK ID to delete
        apk_id: String,
    },
}

const APK_CONTENT_TYPE: &str = "application/vnd.android.package-archive";

pub async fn run(
    cmd: &ApksCommands,
    format: OutputFormat,
    dry_run: bool,
    timeout: u64,
) -> Result<()> {
    match cmd {
        ApksCommands::List { app_id, edit_id } => {
            exec::api_get(
                &format!("/applications/{app_id}/edits/{edit_id}/apks"),
                format,
                dry_run,
                timeout,
            )
            .await
        }
        ApksCommands::Get {
            app_id,
            edit_id,
            apk_id,
        } => {
            exec::api_get(
                &format!("/applications/{app_id}/edits/{edit_id}/apks/{apk_id}"),
                format,
                dry_run,
                timeout,
            )
            .await
        }
        ApksCommands::Upload {
            app_id,
            edit_id,
            file,
        } => {
            exec::api_upload(
                &format!("/applications/{app_id}/edits/{edit_id}/apks/upload"),
                file,
                APK_CONTENT_TYPE,
                format,
                dry_run,
                timeout,
            )
            .await
        }
        ApksCommands::Replace {
            app_id,
            edit_id,
            apk_id,
            file,
        } => {
            let etag_path = format!("/applications/{app_id}/edits/{edit_id}/apks/{apk_id}");
            exec::api_replace_with_etag(
                &format!("{}/replace", etag_path),
                &etag_path,
                file,
                APK_CONTENT_TYPE,
                format,
                dry_run,
                timeout,
            )
            .await
        }
        ApksCommands::Delete {
            app_id,
            edit_id,
            apk_id,
        } => {
            exec::api_delete_with_etag(
                &format!("/applications/{app_id}/edits/{edit_id}/apks/{apk_id}"),
                &format!("/applications/{app_id}/edits/{edit_id}/apks/{apk_id}"),
                format,
                dry_run,
                timeout,
            )
            .await
        }
    }
}
