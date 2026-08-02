//! FoukoBot entry point.

use foukoapi::banner::Tone;
use foukoapi::{bootstrap_env, open_storage, Accounts, Bot, Economy, EnvState, Platform};
use futures::FutureExt;
use std::time::{Duration, Instant};

mod ai;
mod alerts;
mod commands;
mod strings;

// "FOUKO" in ANSI Shadow; the framework paints the gradient.
const ART: [&str; 6] = [
    "███████╗ ██████╗ ██╗   ██╗██╗  ██╗ ██████╗ ",
    "██╔════╝██╔═══██╗██║   ██║██║ ██╔╝██╔═══██╗",
    "█████╗  ██║   ██║██║   ██║█████╔╝ ██║   ██║",
    "██╔══╝  ██║   ██║██║   ██║██╔═██╗ ██║   ██║",
    "██║     ╚██████╔╝╚██████╔╝██║  ██╗╚██████╔╝",
    "╚═╝      ╚═════╝  ╚═════╝ ╚═╝  ╚═╝ ╚═════╝ ",
];

/// Storage key that marks a run in progress. Present at startup means the
/// previous run didn't shut down cleanly.
const RUNSTATE_KEY: &str = "foukobot:runstate";

/// Minimum gap between "bot exited with an error" owner alerts, so a
/// flapping network doesn't turn into a DM flood.
const ERROR_ALERT_COOLDOWN: Duration = Duration::from_secs(600);

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let started = Instant::now();

    // First run: write a commented .env template and exit so the operator
    // can fill it in.
    if bootstrap_env()? == EnvState::Created {
        eprintln!(
            "wrote a fresh .env template. fill in TG_TOKEN / DISCORD_TOKEN / FOUKO_DB and run again."
        );
        return Ok(());
    }

    // Keep the file-logging guard alive for the whole process; dropping it
    // would flush and close the log writer.
    let _log_guard = init_logging();

    // --- Storage ------------------------------------------------------------
    // Auto-creates the SQLite file if FOUKO_DB is sqlite:..., falls back to
    // in-memory if the variable is missing. Storage, accounts and the
    // economy are built once and reused across restarts.
    let storage = open_storage()?;
    let accounts = Accounts::with_arc(storage.clone());
    let econ = Economy::new(accounts.clone());
    let notifier = foukoapi::Notifier::new();
    let owner_alerts = alerts::OwnerAlerts::from_env();

    // Unclean-shutdown marker: if the key survived from the previous run,
    // that run never reached its clean-exit path. Remember when it started
    // so the crash alert can say roughly when the bot went down.
    let crash_marker: Option<String> = storage.get(RUNSTATE_KEY).await.ok().flatten();
    if let Err(e) = storage
        .set(RUNSTATE_KEY, &chrono::Utc::now().timestamp().to_string())
        .await
    {
        tracing::warn!(error = %e, "could not write the runstate marker");
    }

    // The AI feature stores hosts, prompts and history encrypted, so it
    // needs a key from FOUKO_SECRET. Without one the feature is simply off
    // and the bot tells users so - everything else runs as normal.
    let ai = match foukoapi::Secret::from_env("FOUKO_SECRET") {
        Ok(secret) => Some(ai::AiStore::new(storage.clone(), secret)),
        Err(_) => {
            tracing::warn!("FOUKO_SECRET is not set; the AI feature is disabled");
            None
        }
    };

    let mut enabled: Vec<&str> = Vec::new();
    let tg_token = non_empty_env("TG_TOKEN");
    let discord_token = non_empty_env("DISCORD_TOKEN");
    if tg_token.is_some() {
        enabled.push("telegram");
    }
    if discord_token.is_some() {
        enabled.push("discord");
    }
    if enabled.is_empty() {
        anyhow::bail!(
            "no platform tokens are set. edit .env and set at least one of TG_TOKEN / DISCORD_TOKEN"
        );
    }

    let services = commands::Services {
        storage: storage.clone(),
        accounts,
        econ,
        i18n: strings::catalogue(),
        notifier: notifier.clone(),
        ai,
    };

    // Flood alarm threshold, shared by the banner and the on_flood hook.
    let flood_per_min: u32 = non_empty_env("FLOOD_ALERT_PER_MIN")
        .and_then(|s| s.parse().ok())
        .unwrap_or(300);

    let log_dir = std::env::var("FOUKO_LOG_DIR").unwrap_or_else(|_| "logs".to_owned());
    let owner_row = owner_alerts.summary();
    let mut startup = foukoapi::banner::Banner::new("FoukoBot", env!("CARGO_PKG_VERSION"))
        .art(&ART)
        .row("platforms", &enabled.join(" + "), Tone::Ok)
        .row("storage", &storage_desc(), Tone::Plain);
    startup = if services.ai.is_some() {
        startup.row("ai", "enabled (FOUKO_SECRET set)", Tone::Ok)
    } else {
        startup.row("ai", "disabled - set FOUKO_SECRET", Tone::Warn)
    };
    startup = match &owner_row {
        Some(desc) => startup.row("owner", &format!("{desc} (verifying...)"), Tone::Plain),
        None => startup.row("owner", "not set - alerts off", Tone::Warn),
    };
    startup
        .row(
            "flood gate",
            &format!("{flood_per_min} updates/min"),
            Tone::Plain,
        )
        .row(
            "logs",
            &format!("{log_dir} (terminal: info, file: debug)"),
            Tone::Plain,
        )
        .print();

    tracing::info!(platforms = ?enabled, "FoukoBot starting");

    // Reload reminders that were pending when we last stopped. This waits
    // for an adapter to come online (so the notifier can actually deliver)
    // and then re-arms every saved timer, once. The same readiness window
    // is a natural spot for the crash alert and the "online" line, so
    // they ride along.
    {
        let svc = services.clone();
        let notifier = notifier.clone();
        let owner = owner_alerts.clone();
        let storage = storage.clone();
        let enabled = enabled.clone();
        tokio::spawn(async move {
            for _ in 0..60 {
                if notifier.is_ready(foukoapi::PlatformKind::Telegram).await
                    || notifier.is_ready(foukoapi::PlatformKind::Discord).await
                {
                    // Sample every enabled platform for the ready line.
                    let mut statuses: Vec<(&str, bool)> = Vec::new();
                    for name in &enabled {
                        let kind = match *name {
                            "telegram" => foukoapi::PlatformKind::Telegram,
                            _ => foukoapi::PlatformKind::Discord,
                        };
                        statuses.push((*name, notifier.is_ready(kind).await));
                    }
                    foukoapi::banner::print_ready(&statuses, started.elapsed());
                    tracing::info!(
                        statuses = ?statuses,
                        startup_secs = started.elapsed().as_secs_f32(),
                        "adapters online"
                    );
                    // Confirm whose ids are in OWNER_* and, on a config
                    // change, DM the owner a one-time greeting.
                    owner.verify_and_greet(&notifier, &storage).await;
                    if let Some(marker) = &crash_marker {
                        let when = marker
                            .parse::<i64>()
                            .ok()
                            .and_then(|ts| chrono::DateTime::from_timestamp(ts, 0))
                            .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
                            .unwrap_or_else(|| "unknown time".to_owned());
                        owner
                            .notify_titled(
                                &notifier,
                                "\u{1F504} FoukoBot restarted",
                                &format!(
                                    "Bot restarted after an unclean shutdown. \
                                     Went down around {when} UTC, back up now."
                                ),
                            )
                            .await;
                    }
                    commands::restore_reminders(svc).await;
                    return;
                }
                tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            }
            tracing::warn!("no adapter came online in time; skipping reminder restore");
        });
    }

    // Supervisor loop: keep the bot alive no matter what. If an adapter
    // returns (network dropped, gateway closed) we log it and start over
    // after a short, capped backoff instead of letting the process die.
    // Nothing a user types can end this loop, so the bot can't be shut
    // down from chat.
    //
    // A real shutdown signal (Ctrl+C, or SIGTERM from systemd/docker) is
    // the one thing that *does* stop us, cleanly, so `docker stop` and
    // `systemctl stop` don't fight the restart logic.
    let mut backoff = Duration::from_secs(1);
    let max_backoff = Duration::from_secs(60);
    // Error alerts are throttled; panics are rare and always reported.
    let mut last_error_alert: Option<Instant> = None;
    loop {
        let bot = build_bot(
            &tg_token,
            &discord_token,
            services.clone(),
            &owner_alerts,
            flood_per_min,
        );

        tokio::select! {
            biased;

            _ = shutdown_signal() => {
                tracing::info!("shutdown signal received; stopping");
                break;
            }

            // Wrap the run future so a panic inside an adapter (teloxide,
            // for instance, panics when it can't reach Telegram at all)
            // turns into a restart instead of taking the whole process down.
            outcome = std::panic::AssertUnwindSafe(bot.run()).catch_unwind() => {
                match outcome {
                    Ok(Ok(())) => tracing::warn!("bot stopped cleanly; restarting"),
                    Ok(Err(e)) => {
                        tracing::error!(error = %e, "bot exited with an error; restarting");
                        // Throttled: transient network flaps restart the
                        // bot in a tight loop and would spam the owner.
                        let due = last_error_alert
                            .map(|t| t.elapsed() >= ERROR_ALERT_COOLDOWN)
                            .unwrap_or(true);
                        if due {
                            last_error_alert = Some(Instant::now());
                            owner_alerts
                                .notify(&notifier, &format!("Bot exited with an error and was restarted: {e}"))
                                .await;
                        }
                    }
                    Err(_) => {
                        tracing::error!("bot panicked; restarting");
                        owner_alerts
                            .notify_titled(
                                &notifier,
                                "\u{1F6A8} FoukoBot panic",
                                "Bot panicked and was restarted. Check the logs.",
                            )
                            .await;
                    }
                }
            }
        }

        tracing::info!(backoff_secs = backoff.as_secs(), "restarting after backoff");
        tokio::time::sleep(backoff).await;
        backoff = (backoff * 2).min(max_backoff);
    }

    // Clean exit: drop the marker so the next start doesn't cry crash.
    if let Err(e) = storage.del(RUNSTATE_KEY).await {
        tracing::warn!(error = %e, "could not clear the runstate marker");
    }

    Ok(())
}

/// Resolve when the process is asked to stop: Ctrl+C on any platform, plus
/// SIGTERM on Unix (what `systemctl stop` / `docker stop` send).
async fn shutdown_signal() {
    let ctrl_c = async {
        let _ = tokio::signal::ctrl_c().await;
    };

    #[cfg(unix)]
    let terminate = async {
        match tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate()) {
            Ok(mut sig) => {
                sig.recv().await;
            }
            Err(_) => std::future::pending::<()>().await,
        }
    };
    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {}
        _ = terminate => {}
    }
}

/// Assemble a fresh `Bot` with every command wired in. Called once per
/// supervisor iteration so a restart always starts from a clean builder.
fn build_bot(
    tg_token: &Option<String>,
    discord_token: &Option<String>,
    services: commands::Services,
    owner_alerts: &alerts::OwnerAlerts,
    flood_per_min: u32,
) -> Bot {
    let notifier = services.notifier.clone();
    let mut bot = Bot::new().with_notifier(notifier.clone());
    if let Some(token) = tg_token {
        bot = bot.add_platform(Platform::telegram(token.clone()));
    }
    if let Some(token) = discord_token {
        bot = bot.add_platform(Platform::discord(token.clone()));
    }

    // Flood alarm: if the update rate blows past the threshold, DM the
    // owner (at most once per 10 minutes - the cooldown is the
    // framework's job here).
    let per_minute = flood_per_min;
    let owner = owner_alerts.clone();
    bot = bot.on_flood(per_minute, Duration::from_secs(600), move |count| {
        let owner = owner.clone();
        let notifier = notifier.clone();
        async move {
            owner
                .notify_titled(
                    &notifier,
                    "\u{1F30A} FoukoBot flood warning",
                    &format!(
                        "Possible flood: {count} updates in the last minute \
                         (threshold {per_minute}). Rate limiting is on, but take a look."
                    ),
                )
                .await;
        }
    });

    commands::register(bot, services)
}

/// Read an env var, returning `None` when it's missing or empty.
fn non_empty_env(key: &str) -> Option<String> {
    std::env::var(key).ok().filter(|s| !s.is_empty())
}

/// Human line for the banner: what FOUKO_DB resolves to.
fn storage_desc() -> String {
    match foukoapi::DbUrl::from_env() {
        Ok(foukoapi::DbUrl::Sqlite(path)) => format!("sqlite ({})", path.display()),
        Ok(foukoapi::DbUrl::Memory) => "in-memory (not persisted)".to_owned(),
        Ok(foukoapi::DbUrl::External(url)) => format!("external ({url})"),
        Err(_) => "unknown".to_owned(),
    }
}

/// Set up logging: pretty lines to stdout plus a daily-rotating file under
/// `logs/`. Returns the appender guard, which must stay alive for logs to
/// keep flushing. The log directory is created if missing; if that fails
/// we fall back to stdout-only so the bot still runs.
fn init_logging() -> Option<tracing_appender::non_blocking::WorkerGuard> {
    use tracing_subscriber::{fmt, prelude::*, EnvFilter};

    // Default filter hides serenity's INFO chatter (shard queuer, runner
    // spans) but keeps its warnings. RUST_LOG, when set, wins entirely.
    let filter = || {
        EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| EnvFilter::new("info,serenity=warn,tracing=warn"))
    };

    // Terminal layer: for eyes - no targets, short local time. The file
    // layer below keeps full timestamps and targets for post-mortems.
    let terminal = || {
        fmt::layer()
            .with_target(false)
            .with_timer(fmt::time::ChronoLocal::new("%H:%M:%S".to_owned()))
    };

    let log_dir = std::env::var("FOUKO_LOG_DIR").unwrap_or_else(|_| "logs".to_owned());
    match std::fs::create_dir_all(&log_dir) {
        Ok(()) => {
            let file_appender = tracing_appender::rolling::daily(&log_dir, "foukobot.log");
            let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);
            tracing_subscriber::registry()
                .with(filter())
                .with(terminal())
                .with(fmt::layer().with_ansi(false).with_writer(non_blocking))
                .init();
            tracing::info!(dir = %log_dir, "file logging enabled");
            Some(guard)
        }
        Err(e) => {
            tracing_subscriber::registry()
                .with(filter())
                .with(terminal())
                .init();
            tracing::warn!(error = %e, "could not open log dir; logging to stdout only");
            None
        }
    }
}
