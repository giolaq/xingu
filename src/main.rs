mod api;
mod auth;
mod cli;
mod output;

use anyhow::{bail, Result};
use clap::Parser;
use std::process;

use cli::{AuthCommands, Commands};
use output::OutputFormat;

#[derive(Parser)]
#[command(
    name = "xingu",
    about = "Amazon AppStore CLI — for humans and agents",
    version,
    propagate_version = true
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Output format
    #[arg(long, global = true, default_value = "json")]
    output: OutputFormat,

    /// Preview the request without executing
    #[arg(long, global = true, default_value_t = false)]
    dry_run: bool,

    /// Show verbose HTTP details
    #[arg(long, global = true, default_value_t = false)]
    verbose: bool,

    /// Request timeout in seconds
    #[arg(long, global = true, default_value_t = 30)]
    timeout: u64,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    // Wire up verbose flag
    api::client::set_verbose(cli.verbose);

    let result = run(cli).await;

    match result {
        Ok(()) => process::exit(0),
        Err(e) => {
            let exit_code = classify_error(&e);
            eprintln!("Error: {e:#}");
            process::exit(exit_code);
        }
    }
}

async fn run(cli: Cli) -> Result<()> {
    let format = cli.output;
    let dry_run = cli.dry_run;
    let timeout = cli.timeout;

    match &cli.command {
        Commands::Auth { command } => run_auth(command).await,
        Commands::Apps { command } => cli::apps::run(command, format, dry_run, timeout).await,
        Commands::Edits { command } => cli::edits::run(command, format, dry_run, timeout).await,
        Commands::Apks { command } => cli::apks::run(command, format, dry_run, timeout).await,
        Commands::Listings { command } => {
            cli::listings::run(command, format, dry_run, timeout).await
        }
        Commands::Details { command } => cli::details::run(command, format, dry_run, timeout).await,
        Commands::Images { command } => cli::images::run(command, format, dry_run, timeout).await,
        Commands::Videos { command } => cli::videos::run(command, format, dry_run, timeout).await,
        Commands::Availability { command } => {
            cli::availability::run(command, format, dry_run, timeout).await
        }
        Commands::Targeting { command } => {
            cli::targeting::run(command, format, dry_run, timeout).await
        }
        Commands::Reports { command } => {
            cli::reports::run(command, format, dry_run, timeout).await
        }
        Commands::Publish(args) => cli::helpers::publish(args, format, dry_run, timeout).await,
        Commands::Status(args) => cli::helpers::status(args, format, dry_run, timeout).await,
        Commands::UpdateListing(args) => {
            cli::helpers::update_listing(args, format, dry_run, timeout).await
        }
    }
}

async fn run_auth(cmd: &AuthCommands) -> Result<()> {
    match cmd {
        AuthCommands::Setup => {
            println!("Amazon AppStore API Credentials Setup");
            println!("=====================================");
            println!();
            println!("You need a Security Profile from the Amazon Developer Console.");
            println!("Go to: Tools & Services > API Access > Create a new security profile");
            println!();

            print!("Client ID: ");
            use std::io::Write;
            std::io::stdout().flush()?;
            let mut client_id = String::new();
            std::io::stdin().read_line(&mut client_id)?;
            let client_id = client_id.trim().to_string();

            if client_id.is_empty() {
                bail!("Client ID cannot be empty.");
            }

            print!("Client Secret: ");
            std::io::stdout().flush()?;
            let mut client_secret = String::new();
            std::io::stdin().read_line(&mut client_secret)?;
            let client_secret = client_secret.trim().to_string();

            if client_secret.is_empty() {
                bail!("Client Secret cannot be empty.");
            }

            let creds = auth::Credentials {
                client_id,
                client_secret,
            };
            auth::save_credentials(&creds)?;
            println!("Credentials saved. Run `xingu auth login` to test.");
            Ok(())
        }
        AuthCommands::Login => {
            auth::login().await?;
            eprintln!("Login successful. Token cached (expires in ~1 hour).");
            Ok(())
        }
        AuthCommands::Token => {
            let token = auth::get_token().await?;
            println!("{token}");
            Ok(())
        }
    }
}

fn classify_error(e: &anyhow::Error) -> i32 {
    let msg = format!("{e:#}");
    if msg.contains("Authentication failed") || msg.contains("OAuth token") {
        2 // auth error
    } else if msg.contains("Rate limited") || msg.contains("API error") {
        1 // API error
    } else if msg.contains("failed to parse") || msg.contains("cannot be empty") {
        3 // validation error
    } else if msg.contains("HTTP request failed") || msg.contains("timeout") {
        4 // network error
    } else {
        1 // generic
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_classify_auth_error() {
        let e = anyhow::anyhow!("Authentication failed (401). Run `xingu auth login`");
        assert_eq!(classify_error(&e), 2);
    }

    #[test]
    fn test_classify_api_error() {
        let e = anyhow::anyhow!("API error (500): internal server error");
        assert_eq!(classify_error(&e), 1);
    }

    #[test]
    fn test_classify_rate_limit() {
        let e = anyhow::anyhow!("Rate limited (429): too many requests");
        assert_eq!(classify_error(&e), 1);
    }

    #[test]
    fn test_classify_validation_error() {
        let e = anyhow::anyhow!("Client ID cannot be empty");
        assert_eq!(classify_error(&e), 3);
    }

    #[test]
    fn test_classify_network_error() {
        let e = anyhow::anyhow!("HTTP request failed: connection refused");
        assert_eq!(classify_error(&e), 4);
    }

    #[test]
    fn test_classify_generic_error() {
        let e = anyhow::anyhow!("something unexpected");
        assert_eq!(classify_error(&e), 1);
    }
}
