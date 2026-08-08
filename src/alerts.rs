//! Owner alerts: operational DMs about crashes and floods.
//!
//! The owner configures OWNER_TG_ID / OWNER_DISCORD_ID (and optionally
//! OWNER_ALERT_PLATFORM) in .env. Alerts go straight to their DMs via
//! the notifier; if the preferred platform is down we try the other one.
//! These messages are for the operator, so they're plain English and
//! skip the i18n catalogue.

use foukoapi::{AnyStorage, Embed, Notifier, PlatformKind, Reply};

/// Accent for owner alerts - matches the warning tone used elsewhere.
const COLOR_ALERT: u32 = 0xF59F00;

/// Storage key holding the owner config marker of the last successful
/// greeting DM, so a plain restart doesn't spam the owner again.
const OWNER_SEEN_KEY: &str = "foukobot:owner:seen";

/// Owner ids plus the preferred delivery platform, read from env once.
#[derive(Clone)]
pub struct OwnerAlerts {
    tg_id: Option<String>,
    discord_id: Option<String>,
    /// Explicit preference from OWNER_ALERT_PLATFORM, if valid.
    preferred: Option<PlatformKind>,
}

impl OwnerAlerts {
    /// Read the owner config from env. Missing or empty ids just switch
    /// the feature off; a bad OWNER_ALERT_PLATFORM value is logged and
    /// ignored (the default preference order still applies).
    pub fn from_env() -> Self {
        let tg_id = non_empty("OWNER_TG_ID");
        let discord_id = non_empty("OWNER_DISCORD_ID");
        let preferred = match non_empty("OWNER_ALERT_PLATFORM").as_deref() {
            Some("telegram") => Some(PlatformKind::Telegram),
            Some("discord") => Some(PlatformKind::Discord),
            Some(other) => {
                tracing::warn!(
                    value = other,
                    "OWNER_ALERT_PLATFORM must be telegram or discord; ignoring"
                );
                None
            }
            None => None,
        };
        if tg_id.is_none() && discord_id.is_none() {
            tracing::info!("owner alerts disabled: no OWNER_TG_ID / OWNER_DISCORD_ID set");
        }
        Self {
            tg_id,
            discord_id,
            preferred,
        }
    }

    /// `true` when at least one owner id is configured.
    pub fn is_enabled(&self) -> bool {
        self.tg_id.is_some() || self.discord_id.is_some()
    }

    /// Short "telegram:123 + discord:456" line for the startup banner.
    /// Ids only - names are resolved later, once an adapter is up.
    pub fn summary(&self) -> Option<String> {
        let mut parts = Vec::new();
        if let Some(id) = &self.tg_id {
            parts.push(format!("telegram:{id}"));
        }
        if let Some(id) = &self.discord_id {
            parts.push(format!("discord:{id}"));
        }
        if parts.is_empty() {
            None
        } else {
            Some(parts.join(" + "))
        }
    }

    /// Startup verification: log who the configured ids belong to, and DM
    /// the owner a one-time "alerts armed" greeting. The greeting is sent
    /// only when the owner config changed since the last successful send
    /// (tracked via `foukobot:owner:seen` in storage), so restarts stay
    /// quiet. Call after at least one adapter is ready.
    pub async fn verify_and_greet(&self, notifier: &Notifier, storage: &AnyStorage) {
        if !self.is_enabled() {
            return;
        }

        // Resolve each configured id to a display name so the operator
        // can see at a glance whose id ended up in .env. Adapters come up
        // at their own pace (Discord's gateway takes a few seconds longer
        // than Telegram), so give each platform a short grace period
        // instead of failing the lookup on whichever was slower.
        for (platform, id) in [
            (PlatformKind::Telegram, self.tg_id.as_deref()),
            (PlatformKind::Discord, self.discord_id.as_deref()),
        ] {
            let Some(id) = id else { continue };
            for _ in 0..30 {
                if notifier.is_dm_ready(platform).await {
                    break;
                }
                tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            }
            match notifier.user_name(platform, id).await {
                Ok(Some(name)) => {
                    // Pretty line for the terminal, plain fact for the file.
                    foukoapi::banner::print_check("owner", &format!("{platform}  {name} ({id})"));
                    tracing::info!("owner verified: {platform} = {name} ({id})");
                }
                Ok(None) => {
                    let hint = match platform {
                        PlatformKind::Telegram => {
                            " (the bot can only see users who have messaged it at least once)"
                        }
                        _ => "",
                    };
                    foukoapi::banner::print_warn(
                        "owner",
                        &format!("id {id} on {platform} matches no known user"),
                    );
                    tracing::warn!("owner id {id} on {platform} matches no known user{hint}");
                }
                Err(e) => {
                    foukoapi::banner::print_warn(
                        "owner",
                        &format!("id {id} on {platform}: verification failed"),
                    );
                    tracing::warn!(%platform, id, error = %e, "owner id verification failed");
                }
            }
        }

        // One-time greeting, keyed on the exact owner config. Same marker
        // as last time means the owner already got their DM - skip.
        let marker = format!(
            "tg={};ds={};platform={}",
            self.tg_id.as_deref().unwrap_or(""),
            self.discord_id.as_deref().unwrap_or(""),
            self.preferred.map(|p| p.to_string()).unwrap_or_default(),
        );
        match storage.get(OWNER_SEEN_KEY).await {
            Ok(Some(seen)) if seen == marker => return,
            Ok(_) => {}
            Err(e) => {
                tracing::warn!(error = %e, "could not read the owner-seen marker");
                return;
            }
        }
        let delivered = self
            .notify_titled(
                notifier,
                "\u{2705} Owner alerts armed",
                "This account now receives FoukoBot owner alerts (crashes, floods). \
                 If this wasn't you, remove OWNER_* from .env.",
            )
            .await;
        // Persist only after a successful send, so a failed delivery is
        // retried on the next restart.
        if delivered {
            if let Err(e) = storage.set(OWNER_SEEN_KEY, &marker).await {
                tracing::warn!(error = %e, "could not save the owner-seen marker");
            }
        } else {
            tracing::warn!("owner greeting not delivered; will retry on next restart");
        }
    }

    /// The owner id for `platform`, if configured.
    fn id_for(&self, platform: PlatformKind) -> Option<&str> {
        match platform {
            PlatformKind::Telegram => self.tg_id.as_deref(),
            PlatformKind::Discord => self.discord_id.as_deref(),
            _ => None,
        }
    }

    /// Delivery order: the explicit preference first (when its id is set
    /// and its adapter can DM), then whichever of telegram / discord is
    /// ready, then anything with an id as a last-ditch attempt.
    async fn targets(&self, notifier: &Notifier) -> Vec<(PlatformKind, String)> {
        let mut order: Vec<PlatformKind> = Vec::new();
        if let Some(p) = self.preferred {
            order.push(p);
        }
        for p in [PlatformKind::Telegram, PlatformKind::Discord] {
            if !order.contains(&p) {
                order.push(p);
            }
        }

        let mut ready = Vec::new();
        let mut cold = Vec::new();
        for p in order {
            let Some(id) = self.id_for(p) else { continue };
            if notifier.is_dm_ready(p).await {
                ready.push((p, id.to_owned()));
            } else {
                cold.push((p, id.to_owned()));
            }
        }
        // Not-yet-ready platforms go last: send_dm will fail fast there,
        // but trying costs nothing and covers a racy adapter start.
        ready.extend(cold);
        ready
    }

    /// DM the owner an alert with the default title. No-op when disabled.
    pub async fn notify(&self, notifier: &Notifier, text: &str) {
        self.notify_titled(notifier, "\u{26A0}\u{FE0F} FoukoBot alert", text)
            .await;
    }

    /// DM the owner an alert with a custom title. Tries each configured
    /// platform in preference order until one delivery succeeds. Returns
    /// `true` when some platform accepted the message.
    pub async fn notify_titled(&self, notifier: &Notifier, title: &str, text: &str) -> bool {
        if !self.is_enabled() {
            return false;
        }
        let targets = self.targets(notifier).await;
        if targets.is_empty() {
            tracing::warn!("owner alert dropped: no owner id configured");
            return false;
        }
        let em = Embed::new()
            .title(title)
            .description(text)
            .color(COLOR_ALERT);
        for (platform, id) in targets {
            match notifier
                .send_dm(platform, id, Reply::embed(em.clone()))
                .await
            {
                Ok(()) => {
                    tracing::info!(%platform, "owner alert delivered");
                    return true;
                }
                Err(e) => {
                    tracing::warn!(%platform, error = %e, "owner alert failed; trying next")
                }
            }
        }
        tracing::error!("owner alert could not be delivered on any platform");
        false
    }
}

/// Read an env var, returning `None` when it's missing or empty.
fn non_empty(key: &str) -> Option<String> {
    std::env::var(key)
        .ok()
        .map(|s| s.trim().to_owned())
        .filter(|s| !s.is_empty())
}
