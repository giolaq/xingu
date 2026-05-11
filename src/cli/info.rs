use anyhow::Result;
use serde_json::json;

use crate::auth::store::config_dir;
use crate::output::{print_output, OutputFormat};

pub async fn run(format: OutputFormat) -> Result<()> {
    let cfg = config_dir()?;
    let creds = cfg.join("credentials.json");
    let token = cfg.join("token_cache.json");
    let reporting = cfg.join("reporting_token_cache.json");

    let skills_dir = dirs::home_dir()
        .map(|h| h.join(".xingu").join("skills"))
        .unwrap_or_default();

    let info = json!({
        "version": env!("CARGO_PKG_VERSION"),
        "config_dir": cfg,
        "credentials_file": creds.exists().then_some(creds),
        "token_cached": token.exists(),
        "reporting_token_cached": reporting.exists(),
        "base_url_override": std::env::var("XINGU_BASE_URL").ok(),
        "xingu_token_env": std::env::var("XINGU_TOKEN").is_ok(),
        "skills_dir": skills_dir,
    });

    print_output(&info, format);
    Ok(())
}
