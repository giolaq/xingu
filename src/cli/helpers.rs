use anyhow::{Context, Result};
use clap::Args;
use std::path::PathBuf;

use crate::api::client::ApiClient;
use crate::output::{print_output, OutputFormat};

#[derive(Args)]
pub struct PublishArgs {
    /// Application ID
    pub app_id: String,
    /// Path to APK file
    #[arg(long)]
    pub file: PathBuf,
}

#[derive(Args)]
pub struct StatusArgs {
    /// Application ID
    pub app_id: String,
}

#[derive(Args)]
pub struct UpdateListingArgs {
    /// Application ID
    pub app_id: String,
    /// Locale code (e.g. en-US)
    #[arg(long)]
    pub locale: String,
    /// App title
    #[arg(long)]
    pub title: Option<String>,
    /// Short description
    #[arg(long)]
    pub short_description: Option<String>,
    /// Full description
    #[arg(long)]
    pub description: Option<String>,
    /// Recent changes / what's new
    #[arg(long)]
    pub recent_changes: Option<String>,
}

pub async fn publish(
    args: &PublishArgs,
    format: OutputFormat,
    dry_run: bool,
    timeout: u64,
) -> Result<()> {
    if dry_run {
        println!("POST /applications/{}/edits", args.app_id);
        println!(
            "POST /applications/{}/edits/<edit_id>/apks/upload (file: {})",
            args.app_id,
            args.file.display()
        );
        println!("POST /applications/{}/edits/<edit_id>/commit", args.app_id);
        return Ok(());
    }

    let client = ApiClient::new(timeout).await?;
    let mut edit_id: Option<String> = None;

    let result = async {
        // Step 1: Create edit
        eprintln!("Creating edit...");
        let edit_path = format!("/applications/{}/edits", args.app_id);
        let edit = client
            .post(&edit_path, &serde_json::json!({}))
            .await
            .context("failed to create edit")?;
        let id = edit["id"]
            .as_str()
            .context("edit response missing id")?
            .to_string();
        edit_id = Some(id.clone());
        eprintln!("Edit created: {id}");

        // Step 2: Upload APK
        eprintln!("Uploading APK...");
        let apk_path = format!("/applications/{}/edits/{}/apks/upload", args.app_id, id);
        let apk_result = client
            .upload_file(
                &apk_path,
                &args.file,
                "application/vnd.android.package-archive",
            )
            .await
            .context("failed to upload APK")?;
        eprintln!("APK uploaded successfully");

        // Step 3: Commit
        eprintln!("Committing edit...");
        let commit_path = format!("/applications/{}/edits/{}/commit", args.app_id, id);
        let commit_result = client
            .post(&commit_path, &serde_json::json!({}))
            .await
            .context("failed to commit edit")?;

        let output = serde_json::json!({
            "edit": edit,
            "apk": apk_result,
            "commit": commit_result,
        });
        print_output(&output, format);
        eprintln!("Published successfully!");
        Ok::<(), anyhow::Error>(())
    }
    .await;

    // Rollback on failure: attempt to delete the dangling edit
    if result.is_err() {
        if let Some(ref id) = edit_id {
            eprintln!("Attempting to clean up edit {id}...");
            let delete_path = format!("/applications/{}/edits/{}", args.app_id, id);
            match client.delete(&delete_path).await {
                Ok(_) => eprintln!("Edit {id} deleted."),
                Err(del_err) => {
                    eprintln!(
                        "Warning: Failed to delete edit {id}: {del_err:#}\n\
                         You can manually delete it with: xingu edits delete {} {id}",
                        args.app_id
                    );
                }
            }
        }
    }

    result
}

pub async fn status(
    args: &StatusArgs,
    format: OutputFormat,
    dry_run: bool,
    timeout: u64,
) -> Result<()> {
    if dry_run {
        println!("GET /applications/{}/edits", args.app_id);
        return Ok(());
    }

    let client = ApiClient::new(timeout).await?;

    let edit_path = format!("/applications/{}/edits", args.app_id);
    let edit = client
        .get(&edit_path)
        .await
        .context("failed to get active edit")?;

    let output = serde_json::json!({
        "appId": args.app_id,
        "activeEdit": edit,
    });
    print_output(&output, format);
    Ok(())
}

pub async fn update_listing(
    args: &UpdateListingArgs,
    format: OutputFormat,
    dry_run: bool,
    timeout: u64,
) -> Result<()> {
    if dry_run {
        println!("GET /applications/{}/edits", args.app_id);
        println!("GET + PUT listing for locale {}", args.locale);
        return Ok(());
    }

    let client = ApiClient::new(timeout).await?;

    // Get active edit
    let edit_path = format!("/applications/{}/edits", args.app_id);
    let edit = client
        .get(&edit_path)
        .await
        .context("failed to get active edit — create one first with `xingu edits create`")?;
    let edit_id = edit["id"].as_str().context("edit response missing id")?;

    // Get current listing (with ETag)
    let listing_path = format!(
        "/applications/{}/edits/{edit_id}/listings/{}",
        args.app_id, args.locale
    );
    let mut listing = client
        .get(&listing_path)
        .await
        .context("failed to get listing")?;

    // Merge updates
    if let Some(title) = &args.title {
        listing["title"] = serde_json::Value::String(title.clone());
    }
    if let Some(short) = &args.short_description {
        listing["shortDescription"] = serde_json::Value::String(short.clone());
    }
    if let Some(desc) = &args.description {
        listing["fullDescription"] = serde_json::Value::String(desc.clone());
    }
    if let Some(changes) = &args.recent_changes {
        listing["recentChanges"] = serde_json::Value::String(changes.clone());
    }

    let result = client
        .put(&listing_path, &listing)
        .await
        .context("failed to update listing")?;
    print_output(&result, format);
    Ok(())
}
