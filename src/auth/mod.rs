pub mod oauth;
pub mod store;

use anyhow::{bail, Result};

pub use oauth::login;
pub use store::{save_credentials, Credentials};

pub async fn get_token() -> Result<String> {
    // Priority 1: XINGU_TOKEN env var
    if let Ok(token) = std::env::var("XINGU_TOKEN") {
        return Ok(token);
    }

    // Priority 2: Cached token (if not expired)
    if let Some(cached) = store::load_cached_token()? {
        return Ok(cached);
    }

    // Priority 3: Auto-login with stored credentials
    fetch_fresh_token().await
}

/// Get a token with the Reporting API scope.
pub async fn get_reporting_token() -> Result<String> {
    // Priority 1: XINGU_TOKEN env var (user manages scope externally)
    if let Ok(token) = std::env::var("XINGU_TOKEN") {
        return Ok(token);
    }

    // Priority 2: Cached reporting token
    if let Some(cached) = store::load_cached_reporting_token()? {
        return Ok(cached);
    }

    // Priority 3: Fetch fresh reporting token
    let creds = store::load_credentials()?;
    match creds {
        Some(creds) => {
            let token =
                oauth::fetch_reporting_token(&creds.client_id, &creds.client_secret).await?;
            store::cache_reporting_token(&token)?;
            Ok(token)
        }
        None => {
            bail!(
                "No credentials found. Run `xingu auth setup` to configure your API credentials."
            );
        }
    }
}

/// Force a fresh token fetch, bypassing cache. Used by ApiClient on 401/403 retry.
pub async fn force_refresh() -> Result<String> {
    // Skip XINGU_TOKEN — if that's set, refreshing won't help
    fetch_fresh_token().await
}

async fn fetch_fresh_token() -> Result<String> {
    let creds = store::load_credentials()?;
    match creds {
        Some(creds) => {
            let token = oauth::fetch_token(&creds.client_id, &creds.client_secret).await?;
            store::cache_token(&token)?;
            Ok(token)
        }
        None => {
            bail!(
                "No credentials found. Run `xingu auth setup` to configure your API credentials."
            );
        }
    }
}
