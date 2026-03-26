use anyhow::Result;
use std::path::Path;

use crate::api::client::ApiClient;
use crate::output::{print_output, OutputFormat};

pub async fn api_get(path: &str, format: OutputFormat, dry_run: bool, timeout: u64) -> Result<()> {
    if dry_run {
        println!("GET {path}");
        return Ok(());
    }
    let client = ApiClient::new(timeout).await?;
    let result = client.get(path).await?;
    print_output(&result, format);
    Ok(())
}

pub async fn api_post(
    path: &str,
    body: &serde_json::Value,
    format: OutputFormat,
    dry_run: bool,
    timeout: u64,
) -> Result<()> {
    if dry_run {
        println!("POST {path}");
        return Ok(());
    }
    let client = ApiClient::new(timeout).await?;
    let result = client.post(path, body).await?;
    print_output(&result, format);
    Ok(())
}

pub async fn api_put_with_etag(
    path: &str,
    body: &serde_json::Value,
    format: OutputFormat,
    dry_run: bool,
    timeout: u64,
) -> Result<()> {
    if dry_run {
        println!("PUT {path}");
        return Ok(());
    }
    let client = ApiClient::new(timeout).await?;
    // GET to capture ETag (stored automatically), then PUT
    let _ = client.get(path).await?;
    let result = client.put(path, body).await?;
    print_output(&result, format);
    Ok(())
}

/// DELETE that first GETs an ETag from `etag_path`, then deletes `path`.
pub async fn api_delete_with_etag(
    path: &str,
    etag_path: &str,
    format: OutputFormat,
    dry_run: bool,
    timeout: u64,
) -> Result<()> {
    if dry_run {
        println!("DELETE {path}");
        return Ok(());
    }
    let client = ApiClient::new(timeout).await?;
    let _ = client.get(etag_path).await?;
    // Copy ETag from etag_path to path so delete can find it
    client.copy_etag(etag_path, path);
    let result = client.delete(path).await?;
    print_output(&result, format);
    Ok(())
}

/// POST that first GETs an ETag from `etag_path`, then posts to `path`.
pub async fn api_post_with_etag(
    path: &str,
    etag_path: &str,
    format: OutputFormat,
    dry_run: bool,
    timeout: u64,
) -> Result<()> {
    if dry_run {
        println!("POST {path}");
        return Ok(());
    }
    let client = ApiClient::new(timeout).await?;
    let _ = client.get(etag_path).await?;
    // Copy ETag from etag_path to path so post can find it
    client.copy_etag(etag_path, path);
    let result = client.post(path, &serde_json::json!({})).await?;
    print_output(&result, format);
    Ok(())
}

pub async fn api_upload(
    path: &str,
    file_path: &Path,
    content_type: &str,
    format: OutputFormat,
    dry_run: bool,
    timeout: u64,
) -> Result<()> {
    if dry_run {
        println!("POST {path} (file: {})", file_path.display());
        return Ok(());
    }
    let client = ApiClient::new(timeout).await?;
    let result = client.upload_file(path, file_path, content_type).await?;
    print_output(&result, format);
    Ok(())
}

/// Upload that first GETs an ETag from `etag_path`, then uploads to `path`.
pub async fn api_upload_with_etag(
    path: &str,
    etag_path: &str,
    file_path: &Path,
    content_type: &str,
    format: OutputFormat,
    dry_run: bool,
    timeout: u64,
) -> Result<()> {
    if dry_run {
        println!("POST {path} (file: {})", file_path.display());
        return Ok(());
    }
    let client = ApiClient::new(timeout).await?;
    let _ = client.get(etag_path).await?;
    client.copy_etag(etag_path, path);
    let result = client.upload_file(path, file_path, content_type).await?;
    print_output(&result, format);
    Ok(())
}

#[allow(dead_code)]
pub async fn api_replace(
    path: &str,
    file_path: &Path,
    content_type: &str,
    format: OutputFormat,
    dry_run: bool,
    timeout: u64,
) -> Result<()> {
    if dry_run {
        println!("PUT {path} (file: {})", file_path.display());
        return Ok(());
    }
    let client = ApiClient::new(timeout).await?;
    let result = client.replace_file(path, file_path, content_type).await?;
    print_output(&result, format);
    Ok(())
}

/// Replace that first GETs an ETag from `etag_path`, then replaces at `path`.
pub async fn api_replace_with_etag(
    path: &str,
    etag_path: &str,
    file_path: &Path,
    content_type: &str,
    format: OutputFormat,
    dry_run: bool,
    timeout: u64,
) -> Result<()> {
    if dry_run {
        println!("PUT {path} (file: {})", file_path.display());
        return Ok(());
    }
    let client = ApiClient::new(timeout).await?;
    let _ = client.get(etag_path).await?;
    client.copy_etag(etag_path, path);
    let result = client.replace_file(path, file_path, content_type).await?;
    print_output(&result, format);
    Ok(())
}

/// Detect content type from file extension
pub fn content_type_for_image(path: &Path) -> &'static str {
    match path.extension().and_then(|e| e.to_str()) {
        Some("png") => "image/png",
        Some("jpg" | "jpeg") => "image/jpeg",
        Some("gif") => "image/gif",
        Some("webp") => "image/webp",
        _ => "image/png", // default fallback
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_content_type_detection() {
        assert_eq!(
            content_type_for_image(&PathBuf::from("icon.png")),
            "image/png"
        );
        assert_eq!(
            content_type_for_image(&PathBuf::from("screen.jpg")),
            "image/jpeg"
        );
        assert_eq!(
            content_type_for_image(&PathBuf::from("screen.jpeg")),
            "image/jpeg"
        );
        assert_eq!(
            content_type_for_image(&PathBuf::from("anim.gif")),
            "image/gif"
        );
        assert_eq!(
            content_type_for_image(&PathBuf::from("hero.webp")),
            "image/webp"
        );
        assert_eq!(
            content_type_for_image(&PathBuf::from("unknown.bmp")),
            "image/png"
        );
        assert_eq!(content_type_for_image(&PathBuf::from("noext")), "image/png");
    }
}
