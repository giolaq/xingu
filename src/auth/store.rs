use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

const TOKEN_TTL_SECS: u64 = 3500; // slightly less than 1hr
const KEYRING_SERVICE: &str = "xingu";
const KEYRING_USER: &str = "credentials";

#[derive(Serialize, Deserialize)]
pub struct Credentials {
    pub client_id: String,
    pub client_secret: String,
}

impl std::fmt::Debug for Credentials {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Credentials")
            .field("client_id", &self.client_id)
            .field("client_secret", &"[REDACTED]")
            .finish()
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct CachedToken {
    token: String,
    expires_at: u64,
}

fn config_dir() -> Result<PathBuf> {
    let dir = dirs::config_dir()
        .context("could not determine config directory")?
        .join("xingu");
    fs::create_dir_all(&dir).context("failed to create config directory")?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let perms = fs::Permissions::from_mode(0o700);
        fs::set_permissions(&dir, perms).ok();
    }

    Ok(dir)
}

fn credentials_path() -> Result<PathBuf> {
    Ok(config_dir()?.join("credentials.json"))
}

fn token_cache_path() -> Result<PathBuf> {
    Ok(config_dir()?.join("token_cache.json"))
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

pub fn save_credentials(creds: &Credentials) -> Result<()> {
    // Try keyring first
    let json = serde_json::to_string(creds)?;
    if save_to_keyring(&json).is_ok() {
        eprintln!("Credentials saved to OS keyring.");
        return Ok(());
    }

    // Fall back to file
    eprintln!("OS keyring unavailable. Saving credentials to file.");
    save_credentials_file(creds)
}

fn save_to_keyring(json: &str) -> Result<()> {
    let entry = keyring::Entry::new(KEYRING_SERVICE, KEYRING_USER)
        .context("failed to create keyring entry")?;
    entry
        .set_password(json)
        .context("failed to save to keyring")?;
    Ok(())
}

fn save_credentials_file(creds: &Credentials) -> Result<()> {
    let path = credentials_path()?;
    let json = serde_json::to_string_pretty(creds)?;
    fs::write(&path, json).context("failed to write credentials")?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let perms = fs::Permissions::from_mode(0o600);
        fs::set_permissions(&path, perms).ok();
    }

    Ok(())
}

pub fn load_credentials() -> Result<Option<Credentials>> {
    // Priority 1: env vars
    if let (Ok(id), Ok(secret)) = (
        std::env::var("XINGU_CLIENT_ID"),
        std::env::var("XINGU_CLIENT_SECRET"),
    ) {
        return Ok(Some(Credentials {
            client_id: id,
            client_secret: secret,
        }));
    }

    // Priority 2: keyring
    if let Some(creds) = load_from_keyring()? {
        return Ok(Some(creds));
    }

    // Priority 3: credentials file
    let path = credentials_path()?;
    if !path.exists() {
        return Ok(None);
    }

    let data = fs::read_to_string(&path).context("failed to read credentials")?;
    let creds: Credentials = serde_json::from_str(&data).context("failed to parse credentials")?;
    Ok(Some(creds))
}

fn load_from_keyring() -> Result<Option<Credentials>> {
    let entry = match keyring::Entry::new(KEYRING_SERVICE, KEYRING_USER) {
        Ok(e) => e,
        Err(_) => return Ok(None),
    };
    match entry.get_password() {
        Ok(json) => {
            let creds: Credentials =
                serde_json::from_str(&json).context("failed to parse keyring credentials")?;
            Ok(Some(creds))
        }
        Err(_) => Ok(None),
    }
}

pub fn cache_token(token: &str) -> Result<()> {
    let cached = CachedToken {
        token: token.to_string(),
        expires_at: now_secs() + TOKEN_TTL_SECS,
    };
    let path = token_cache_path()?;
    let json = serde_json::to_string(&cached)?;
    fs::write(&path, json).context("failed to cache token")?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let perms = fs::Permissions::from_mode(0o600);
        fs::set_permissions(&path, perms).ok();
    }

    Ok(())
}

pub fn load_cached_token() -> Result<Option<String>> {
    let path = token_cache_path()?;
    load_cached_token_from(&path)
}

/// Load and validate a cached token from a specific path. Extracted for testability.
fn load_cached_token_from(path: &std::path::Path) -> Result<Option<String>> {
    if !path.exists() {
        return Ok(None);
    }

    let data = fs::read_to_string(path).context("failed to read token cache")?;
    let cached: CachedToken = serde_json::from_str(&data).context("failed to parse token cache")?;

    if now_secs() < cached.expires_at {
        Ok(Some(cached.token))
    } else {
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;
    use std::env;

    #[test]
    #[serial]
    fn test_load_credentials_from_env() {
        env::set_var("XINGU_CLIENT_ID", "test-id");
        env::set_var("XINGU_CLIENT_SECRET", "test-secret");

        let creds = load_credentials().unwrap().unwrap();
        assert_eq!(creds.client_id, "test-id");
        assert_eq!(creds.client_secret, "test-secret");

        env::remove_var("XINGU_CLIENT_ID");
        env::remove_var("XINGU_CLIENT_SECRET");
    }

    #[test]
    fn test_token_cache_valid_token() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("token_cache.json");

        let cached = CachedToken {
            token: "valid-token-abc".to_string(),
            expires_at: now_secs() + 3600, // expires in 1 hour
        };
        fs::write(&path, serde_json::to_string(&cached).unwrap()).unwrap();

        let result = load_cached_token_from(&path).unwrap();
        assert_eq!(result, Some("valid-token-abc".to_string()));
    }

    #[test]
    fn test_token_cache_expired_token() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("token_cache.json");

        let cached = CachedToken {
            token: "expired-token".to_string(),
            expires_at: 0, // expired long ago
        };
        fs::write(&path, serde_json::to_string(&cached).unwrap()).unwrap();

        let result = load_cached_token_from(&path).unwrap();
        assert_eq!(result, None);
    }

    #[test]
    fn test_token_cache_missing_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("nonexistent.json");

        let result = load_cached_token_from(&path).unwrap();
        assert_eq!(result, None);
    }

    #[test]
    fn test_token_cache_corrupted_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("token_cache.json");

        fs::write(&path, "not valid json").unwrap();

        let result = load_cached_token_from(&path);
        assert!(result.is_err());
    }

    #[test]
    fn test_now_secs_is_reasonable() {
        let now = now_secs();
        // Should be after 2024-01-01
        assert!(now > 1704067200);
    }
}
