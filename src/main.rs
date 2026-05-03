//! FoukoBot entry point.

use foukoapi::{bootstrap_env, open_storage, Accounts, Bot, EnvState, Platform};

mod commands;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // First run: write a commented .env template and exit so the operator
    // can fill it in.
    if bootstrap_env()? == EnvState::Created {
        eprintln!(
            "wrote a fresh .env template. fill in TG_TOKEN / DISCORD_TOKEN / FOUKO_DB and run again."
        );
        return Ok(());
    }

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,foukoapi=info,foukobot=debug".into()),
        )
        .init();

    // --- Storage ------------------------------------------------------------
    // Auto-creates the SQLite file if FOUKO_DB is sqlite:..., falls back to
    // in-memory if the variable is missing.
    let storage = open_storage()?;
    let accounts = Accounts::with_arc(storage.clone());

    // --- Bot + platforms ----------------------------------------------------
    let mut bot = Bot::new();
    let mut enabled: Vec<&str> = Vec::new();

    if let Ok(token) = std::env::var("TG_TOKEN") {
        if !token.is_empty() {
            bot = bot.add_platform(Platform::telegram(token));
            enabled.push("telegram");
        }
    }
    if let Ok(token) = std::env::var("DISCORD_TOKEN") {
        if !token.is_empty() {
            bot = bot.add_platform(Platform::discord(token));
            enabled.push("discord");
        }
    }
    if enabled.is_empty() {
        anyhow::bail!(
            "no platform tokens are set. edit .env and set at least one of TG_TOKEN / DISCORD_TOKEN"
        );
    }

    tracing::info!(platforms = ?enabled, "FoukoBot starting");

    let services = commands::Services {
        storage,
        accounts,
        weather_key: std::env::var("OPEN_WEATHER_KEY").ok().filter(|s| !s.is_empty()),
    };

    commands::register(bot, services).run().await?;
    Ok(())
}
