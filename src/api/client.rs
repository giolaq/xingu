use anyhow::{bail, Context, Result};
use bytes::Bytes;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE, IF_MATCH};
use reqwest::{Client, ClientBuilder, Method, Response, StatusCode};
use std::collections::HashMap;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use std::time::{Duration, Instant};

use crate::auth;

const BASE_URL: &str = "https://developer.amazon.com/api/appstore/v1";
const DOWNLOAD_BASE_URL: &str = "https://developer.amazon.com/api/appstore";
const MAX_429_RETRIES: u32 = 3;
const MAX_OUTER_ATTEMPTS: u32 = 2;

static VERBOSE: AtomicBool = AtomicBool::new(false);

pub fn set_verbose(v: bool) {
    VERBOSE.store(v, Ordering::Relaxed);
}

fn is_verbose() -> bool {
    VERBOSE.load(Ordering::Relaxed)
}

/// Result of a retried request, including optional ETag from response headers.
struct RetryResult {
    value: serde_json::Value,
    etag: Option<String>,
}

/// Validate that a custom base URL is safe to send credentials to.
/// Allows HTTPS amazon.com domains and localhost (for testing).
fn validate_base_url(url: &str) -> Result<()> {
    if url.starts_with("http://localhost") || url.starts_with("http://127.0.0.1") {
        return Ok(());
    }
    if !url.starts_with("https://") {
        bail!("XINGU_BASE_URL must use HTTPS (got: {url})");
    }
    // Extract host from URL
    let host = url
        .strip_prefix("https://")
        .unwrap()
        .split('/')
        .next()
        .unwrap_or("");
    if host.ends_with("amazon.com") || host.ends_with(".amazon.com") || host == "localhost" {
        Ok(())
    } else {
        bail!(
            "XINGU_BASE_URL must point to an amazon.com domain or localhost (got: {url}). \
             Bearer tokens will not be sent to untrusted hosts."
        );
    }
}

pub struct ApiClient {
    http: Client,
    token: Mutex<String>,
    base_url: String,
    download_base_url: String,
    etags: Mutex<HashMap<String, String>>,
}

impl ApiClient {
    pub async fn new(timeout_secs: u64) -> Result<Self> {
        let token = auth::get_token().await?;
        Self::with_token(token, timeout_secs)
    }

    pub async fn new_reporting(timeout_secs: u64) -> Result<Self> {
        let token = auth::get_reporting_token().await?;
        Self::with_token(token, timeout_secs)
    }

    fn with_token(token: String, timeout_secs: u64) -> Result<Self> {
        let base_url = match std::env::var("XINGU_BASE_URL") {
            Ok(url) => {
                validate_base_url(&url)?;
                url.clone()
            }
            Err(_) => BASE_URL.to_string(),
        };
        let download_base_url = match std::env::var("XINGU_BASE_URL") {
            Ok(url) => url,
            Err(_) => DOWNLOAD_BASE_URL.to_string(),
        };
        let http = ClientBuilder::new()
            .timeout(Duration::from_secs(timeout_secs))
            .build()
            .context("failed to create HTTP client")?;
        Ok(Self {
            http,
            token: Mutex::new(token),
            base_url,
            download_base_url,
            etags: Mutex::new(HashMap::new()),
        })
    }

    fn auth_headers(&self) -> HeaderMap {
        let mut headers = HeaderMap::new();
        let token = self.token.lock().unwrap().clone();
        let val = format!("Bearer {}", token);
        headers.insert(AUTHORIZATION, HeaderValue::from_str(&val).unwrap());
        headers
    }

    async fn refresh_token(&self) -> Result<()> {
        let new_token = auth::force_refresh().await?;
        *self.token.lock().unwrap() = new_token;
        Ok(())
    }

    fn url(&self, path: &str) -> String {
        format!("{}{}", self.base_url, path)
    }

    fn download_url(&self, path: &str) -> String {
        format!("{}{}", self.download_base_url, path)
    }

    fn store_etag(&self, path: &str, etag: &str) {
        self.etags
            .lock()
            .unwrap()
            .insert(path.to_string(), etag.to_string());
    }

    fn get_etag(&self, path: &str) -> Option<String> {
        self.etags.lock().unwrap().get(path).cloned()
    }

    pub fn copy_etag(&self, from: &str, to: &str) {
        if let Some(etag) = self.get_etag(from) {
            self.store_etag(to, &etag);
        }
    }

    fn log_request(method: &str, url: &str) {
        if is_verbose() {
            eprintln!("[verbose] --> {} {}", method, url);
        }
    }

    fn log_response(method: &str, url: &str, status: StatusCode, elapsed: Duration) {
        if is_verbose() {
            eprintln!(
                "[verbose] <-- {} {} {} ({:.0?})",
                method, url, status, elapsed
            );
        }
    }

    fn extract_etag(resp: &Response) -> Option<String> {
        resp.headers()
            .get("ETag")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string())
    }

    /// Parse response body. Returns (status, parsed_value) for both success and error cases.
    /// Error bodies are wrapped in a JSON object so callers can extract the message.
    pub(crate) fn check_response_sync(status: StatusCode, body: &str) -> Result<serde_json::Value> {
        if status.is_success() {
            if body.is_empty() {
                return Ok(serde_json::Value::Null);
            }
            serde_json::from_str(body).context("failed to parse response JSON")
        } else {
            Ok(serde_json::json!({ "error": body, "status": status.as_u16() }))
        }
    }

    /// Convert a non-success status + parsed value into a typed error.
    pub(crate) fn result_from_status(
        status: StatusCode,
        value: serde_json::Value,
    ) -> Result<serde_json::Value> {
        if status.is_success() {
            Ok(value)
        } else {
            let body = value["error"].as_str().unwrap_or("unknown error");
            match status {
                StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN => {
                    bail!(
                        "Authentication failed ({}). Run `xingu auth login` to refresh your token.\n{}",
                        status,
                        body
                    );
                }
                StatusCode::TOO_MANY_REQUESTS => {
                    bail!("Rate limited (429): {}", body);
                }
                _ => {
                    bail!("API error ({}): {}", status, body);
                }
            }
        }
    }

    /// Core retry engine. Handles 429 backoff (up to 3 retries) and one 401/403 token refresh.
    /// Returns the parsed value and any ETag from the final successful response.
    ///
    /// ```text
    ///   OUTER (max 2 attempts) ◄─── auth refresh re-enters here
    ///   └─ INNER FOR 0..=3    ◄─── 429 retries
    ///       ├─ 429? sleep + continue
    ///       ├─ 401? refresh + break to outer
    ///       └─ other? return result
    /// ```
    async fn execute_with_retry<F, Fut>(
        &self,
        method: &str,
        url: &str,
        make_request: F,
    ) -> Result<RetryResult>
    where
        F: Fn(HeaderMap) -> Fut,
        Fut: std::future::Future<Output = Result<Response>>,
    {
        let mut auth_retried = false;

        for _outer in 0..MAX_OUTER_ATTEMPTS {
            for attempt in 0..=MAX_429_RETRIES {
                let start = Instant::now();
                Self::log_request(method, url);

                let resp = make_request(self.auth_headers()).await?;
                let elapsed = start.elapsed();
                let status = resp.status();
                Self::log_response(method, url, status, elapsed);

                // Capture ETag before consuming body
                let etag = Self::extract_etag(&resp);

                // Handle 429 with backoff
                if status == StatusCode::TOO_MANY_REQUESTS && attempt < MAX_429_RETRIES {
                    let wait = Duration::from_secs(2u64.pow(attempt));
                    if is_verbose() {
                        eprintln!("[verbose] Rate limited. Retrying in {:.0?}...", wait);
                    }
                    tokio::time::sleep(wait).await;
                    continue;
                }

                // Handle 401/403 with one token refresh
                if (status == StatusCode::UNAUTHORIZED || status == StatusCode::FORBIDDEN)
                    && !auth_retried
                {
                    auth_retried = true;
                    if is_verbose() {
                        eprintln!("[verbose] Auth failed. Refreshing token and retrying...");
                    }
                    if self.refresh_token().await.is_ok() {
                        break; // break inner loop, re-enter outer loop with new token
                    }
                }

                // Parse and return
                let body = resp.text().await.context("failed to read response body")?;
                let value = Self::check_response_sync(status, &body)?;
                let value = Self::result_from_status(status, value)?;
                return Ok(RetryResult { value, etag });
            }
            // Inner loop ended via break (auth refresh) — outer loop continues
        }

        bail!("Request failed after retries")
    }

    pub async fn get(&self, path: &str) -> Result<serde_json::Value> {
        let url = self.url(path);
        let result = self
            .execute_with_retry("GET", &url, |headers| {
                let http = self.http.clone();
                let url = url.clone();
                async move {
                    http.get(&url)
                        .headers(headers)
                        .send()
                        .await
                        .context("HTTP request failed")
                }
            })
            .await?;

        // Store ETag if present (for subsequent PUT with If-Match)
        if let Some(etag) = &result.etag {
            self.store_etag(path, etag);
        }
        Ok(result.value)
    }

    /// GET that returns the raw response body as a string (for Reporting API S3 URLs).
    pub async fn get_raw(&self, path: &str) -> Result<String> {
        let url = self.download_url(path);
        Self::log_request("GET", &url);

        let start = Instant::now();
        let resp = self
            .http
            .get(&url)
            .headers(self.auth_headers())
            .send()
            .await
            .context("HTTP request failed")?;

        let status = resp.status();
        Self::log_response("GET", &url, status, start.elapsed());

        let body = resp.text().await.context("failed to read response body")?;

        if !status.is_success() {
            if status == StatusCode::UNAUTHORIZED || status == StatusCode::FORBIDDEN {
                bail!(
                    "Authentication failed ({}). The Reporting API may require a separate security profile with scope `adx_reporting::appstore:marketer`.\n{}",
                    status,
                    body
                );
            }
            bail!("API error ({}): {}", status, body);
        }

        Ok(body)
    }

    pub async fn post(&self, path: &str, body: &serde_json::Value) -> Result<serde_json::Value> {
        let url = self.url(path);
        let body = body.clone();
        let etag = self.get_etag(path);
        let result = self
            .execute_with_retry("POST", &url, |headers| {
                let http = self.http.clone();
                let url = url.clone();
                let body = body.clone();
                let etag = etag.clone();
                async move {
                    let mut req = http.post(&url).headers(headers).json(&body);
                    if let Some(etag) = etag {
                        req = req.header(IF_MATCH, etag);
                    }
                    req.send().await.context("HTTP request failed")
                }
            })
            .await?;
        Ok(result.value)
    }

    pub async fn put(&self, path: &str, body: &serde_json::Value) -> Result<serde_json::Value> {
        let url = self.url(path);
        let body = body.clone();
        let etag = self.get_etag(path);
        let result = self
            .execute_with_retry("PUT", &url, |headers| {
                let http = self.http.clone();
                let url = url.clone();
                let body = body.clone();
                let etag = etag.clone();
                async move {
                    let mut req = http.put(&url).headers(headers).json(&body);
                    if let Some(etag) = etag {
                        req = req.header(IF_MATCH, etag);
                    }
                    req.send().await.context("HTTP request failed")
                }
            })
            .await?;
        Ok(result.value)
    }

    pub async fn delete(&self, path: &str) -> Result<serde_json::Value> {
        let url = self.url(path);
        let etag = self.get_etag(path);
        let result = self
            .execute_with_retry("DELETE", &url, |headers| {
                let http = self.http.clone();
                let url = url.clone();
                let etag = etag.clone();
                async move {
                    let mut req = http.delete(&url).headers(headers);
                    if let Some(etag) = etag {
                        req = req.header(IF_MATCH, etag);
                    }
                    req.send().await.context("HTTP request failed")
                }
            })
            .await?;
        Ok(result.value)
    }

    /// Send a file via POST or PUT with retry support.
    /// Uses `Bytes` internally to avoid cloning the file buffer on retries.
    async fn send_file(
        &self,
        method: Method,
        path: &str,
        file_path: &Path,
        content_type: &str,
    ) -> Result<serde_json::Value> {
        let file_bytes: Bytes = tokio::fs::read(file_path)
            .await
            .context("failed to read file")?
            .into();

        if file_bytes.is_empty() {
            bail!("File is empty: {}", file_path.display());
        }

        let url = self.url(path);
        let content_type = content_type.to_string();
        let etag = self.get_etag(path);
        let method_str = method.to_string();

        let result = self
            .execute_with_retry(&method_str, &url, |mut headers| {
                let http = self.http.clone();
                let url = url.clone();
                let file_bytes = file_bytes.clone(); // O(1) — Bytes is reference-counted
                let content_type = content_type.clone();
                let etag = etag.clone();
                let method = method.clone();
                async move {
                    headers.insert(CONTENT_TYPE, HeaderValue::from_str(&content_type).unwrap());
                    let mut req = http.request(method, &url).headers(headers).body(file_bytes);
                    if let Some(etag) = etag {
                        req = req.header(IF_MATCH, etag);
                    }
                    req.send().await.context("HTTP request failed")
                }
            })
            .await?;
        Ok(result.value)
    }

    pub async fn upload_file(
        &self,
        path: &str,
        file_path: &Path,
        content_type: &str,
    ) -> Result<serde_json::Value> {
        self.send_file(Method::POST, path, file_path, content_type)
            .await
    }

    pub async fn replace_file(
        &self,
        path: &str,
        file_path: &Path,
        content_type: &str,
    ) -> Result<serde_json::Value> {
        self.send_file(Method::PUT, path, file_path, content_type)
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_response_sync_success() {
        let body = r#"{"id": "123", "status": "DRAFT"}"#;
        let result = ApiClient::check_response_sync(StatusCode::OK, body).unwrap();
        assert_eq!(result["id"], "123");
        assert_eq!(result["status"], "DRAFT");
    }

    #[test]
    fn test_check_response_sync_empty_success() {
        let result = ApiClient::check_response_sync(StatusCode::NO_CONTENT, "").unwrap();
        assert_eq!(result, serde_json::Value::Null);
    }

    #[test]
    fn test_check_response_sync_error_wraps_body() {
        let result =
            ApiClient::check_response_sync(StatusCode::BAD_REQUEST, "invalid field").unwrap();
        assert_eq!(result["error"], "invalid field");
        assert_eq!(result["status"], 400);
    }

    #[test]
    fn test_check_response_sync_malformed_json() {
        let result = ApiClient::check_response_sync(StatusCode::OK, "not json");
        assert!(result.is_err());
        let msg = format!("{}", result.unwrap_err());
        assert!(msg.contains("failed to parse response JSON"));
    }

    #[test]
    fn test_result_from_status_success() {
        let val = serde_json::json!({"ok": true});
        let result = ApiClient::result_from_status(StatusCode::OK, val.clone()).unwrap();
        assert_eq!(result, val);
    }

    #[test]
    fn test_result_from_status_401() {
        let val = serde_json::json!({"error": "bad token", "status": 401});
        let result = ApiClient::result_from_status(StatusCode::UNAUTHORIZED, val);
        assert!(result.is_err());
        let msg = format!("{}", result.unwrap_err());
        assert!(msg.contains("Authentication failed"));
    }

    #[test]
    fn test_result_from_status_429() {
        let val = serde_json::json!({"error": "too fast", "status": 429});
        let result = ApiClient::result_from_status(StatusCode::TOO_MANY_REQUESTS, val);
        assert!(result.is_err());
        let msg = format!("{}", result.unwrap_err());
        assert!(msg.contains("Rate limited"));
    }

    #[test]
    fn test_result_from_status_500() {
        let val = serde_json::json!({"error": "internal", "status": 500});
        let result = ApiClient::result_from_status(StatusCode::INTERNAL_SERVER_ERROR, val);
        assert!(result.is_err());
        let msg = format!("{}", result.unwrap_err());
        assert!(msg.contains("API error (500 Internal Server Error)"));
    }

    #[test]
    fn test_validate_base_url_default_amazon() {
        assert!(validate_base_url("https://developer.amazon.com/api/appstore/v1").is_ok());
    }

    #[test]
    fn test_validate_base_url_localhost_http() {
        assert!(validate_base_url("http://localhost:8080/api").is_ok());
    }

    #[test]
    fn test_validate_base_url_localhost_127() {
        assert!(validate_base_url("http://127.0.0.1:9999").is_ok());
    }

    #[test]
    fn test_validate_base_url_rejects_http_non_localhost() {
        let result = validate_base_url("http://evil.com/steal");
        assert!(result.is_err());
        assert!(format!("{}", result.unwrap_err()).contains("must use HTTPS"));
    }

    #[test]
    fn test_validate_base_url_rejects_non_amazon_https() {
        let result = validate_base_url("https://evil.com/api");
        assert!(result.is_err());
        assert!(format!("{}", result.unwrap_err()).contains("amazon.com"));
    }

    #[test]
    fn test_validate_base_url_rejects_subdomain_trick() {
        let result = validate_base_url("https://amazon.com.evil.com/api");
        assert!(result.is_err());
    }
}
