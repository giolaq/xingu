use crate::api::models::TokenResponse;
use crate::auth::store;
use anyhow::{bail, Context, Result};
use std::time::Duration;

const TOKEN_URL: &str = "https://api.amazon.com/auth/O2/token";
const SCOPE: &str = "appstore::apps:readwrite";
const OAUTH_TIMEOUT_SECS: u64 = 30;

pub async fn fetch_token(client_id: &str, client_secret: &str) -> Result<String> {
    let client = reqwest::ClientBuilder::new()
        .timeout(Duration::from_secs(OAUTH_TIMEOUT_SECS))
        .build()
        .context("failed to create OAuth HTTP client")?;
    let params = [
        ("grant_type", "client_credentials"),
        ("client_id", client_id),
        ("client_secret", client_secret),
        ("scope", SCOPE),
    ];

    let resp = client
        .post(TOKEN_URL)
        .form(&params)
        .send()
        .await
        .context("failed to request OAuth token")?;

    let status = resp.status();
    let body = resp.text().await.context("failed to read token response")?;

    if !status.is_success() {
        bail!("OAuth token request failed ({}): {}", status, body);
    }

    let token_resp: TokenResponse =
        serde_json::from_str(&body).context("failed to parse token response")?;

    Ok(token_resp.access_token)
}

pub async fn login() -> Result<String> {
    let creds = store::load_credentials()?;
    match creds {
        Some(creds) => {
            let token = fetch_token(&creds.client_id, &creds.client_secret).await?;
            store::cache_token(&token)?;
            Ok(token)
        }
        None => {
            bail!("No credentials found. Run `xingu auth setup` first.");
        }
    }
}
