//! General/info commands: `/start`, `/help`, `/ping`, `/info`, `/server`,
//! `/avatar`.

use super::helpers::uptime_string;
use super::Services;
use super::{COLOR_ACCENT, COLOR_OK, COLOR_WARN};
use foukoapi::{Ctx, Embed, PlatformKind, Reply, Result};

/// `/start` / `help` / `/` / `/?` - friendly nudge toward the full `/help`.
pub(crate) async fn help(ctx: Ctx, svc: Services) -> Result<()> {
    let em = Embed::new()
        .title(svc.tr(&ctx, "help_title").await)
        .description(svc.tr(&ctx, "help_intro").await)
        .field(
            svc.tr(&ctx, "help_try").await,
            "`/menu` · `/profile` · `/info` · `/link` · `/lang`",
        )
        .footer("bot.fouko.xyz")
        .url("https://bot.fouko.xyz")
        .color(COLOR_ACCENT);
    ctx.reply_with(Reply::embed(em)).await
}

pub(crate) async fn ping(ctx: Ctx, svc: Services) -> Result<()> {
    let em = Embed::new()
        .title(svc.tr(&ctx, "ping_title").await)
        .description(svc.tr(&ctx, "ping_body").await)
        .color(COLOR_OK);
    ctx.reply_with(Reply::embed(em)).await
}

/// `/info` - the one bot card: what it is, its stack and features, plus
/// live version / uptime / player count. Replaces the old `/about`,
/// `/stats` and `/bot`.
pub(crate) async fn info(ctx: Ctx, svc: Services) -> Result<()> {
    let version = env!("CARGO_PKG_VERSION");
    let uptime = uptime_string();
    let players = svc.econ.player_count().await;

    let em = Embed::new()
        .title("\u{1F916} FoukoBot")
        .url("https://bot.fouko.xyz")
        .description(svc.tr(&ctx, "info_tagline").await)
        .field(
            svc.tr(&ctx, "info_about_label").await,
            svc.tr(&ctx, "info_about_text").await,
        )
        .field(
            svc.tr(&ctx, "info_stack_label").await,
            svc.tr(&ctx, "info_stack_text").await,
        )
        .field(
            svc.tr(&ctx, "info_features_label").await,
            svc.tr(&ctx, "info_features_text").await,
        )
        .field_inline(svc.tr(&ctx, "info_version").await, format!("`{version}`"))
        .field_inline(svc.tr(&ctx, "info_uptime").await, uptime)
        .field_inline(svc.tr(&ctx, "info_players").await, players.to_string())
        .field(
            svc.tr(&ctx, "info_links_label").await,
            svc.tr(&ctx, "info_links_text").await,
        )
        .footer(svc.tr(&ctx, "info_footer").await)
        .color(COLOR_ACCENT);
    ctx.reply_with(Reply::embed(em)).await
}

/// `/server` - info about the current server (Discord) or chat (Telegram):
/// name, member count, description. Uses the cross-platform `chat_info`
/// lookup so one handler serves both.
pub(crate) async fn server(ctx: Ctx, svc: Services) -> Result<()> {
    let info = match ctx.chat_info().await {
        Ok(i) => i,
        Err(e) => {
            tracing::warn!(error = %e, "chat_info lookup failed");
            let em = Embed::new()
                .title(svc.tr(&ctx, "srv_title").await)
                .description(svc.tr(&ctx, "server_unavailable").await)
                .color(COLOR_WARN);
            return ctx.reply_with(Reply::embed(em)).await;
        }
    };

    // In a one-to-one chat there's no server to describe.
    if info.is_private {
        let em = Embed::new()
            .title(svc.tr(&ctx, "srv_title").await)
            .description(svc.tr(&ctx, "server_dm").await)
            .color(COLOR_WARN);
        return ctx.reply_with(Reply::embed(em)).await;
    }

    let title_key = match ctx.platform() {
        PlatformKind::Discord => "srv_title_discord",
        _ => "srv_title_chat",
    };
    let mut em = Embed::new()
        .title(svc.tr(&ctx, title_key).await)
        .color(COLOR_ACCENT);
    if let Some(name) = &info.title {
        em = em.field_inline(svc.tr(&ctx, "srv_name").await, name.clone());
    }
    if let Some(count) = info.member_count {
        em = em.field_inline(svc.tr(&ctx, "srv_members").await, count.to_string());
    }
    if let Some(desc) = &info.description {
        if !desc.is_empty() {
            em = em.field(svc.tr(&ctx, "srv_about").await, desc.clone());
        }
    }
    if let Some(icon) = &info.icon_url {
        em = em.thumbnail(icon.clone());
    }
    ctx.reply_with(Reply::embed(em)).await
}

/// `/avatar` - show a user's avatar (and banner, if any). Discord only:
/// Telegram doesn't expose a shareable avatar URL. With no argument it
/// shows the caller's; a mention or id targets someone else - but since
/// we only have the caller's rich user object in the event, targeting
/// others falls back to the caller with a note.
pub(crate) async fn avatar(ctx: Ctx, svc: Services) -> Result<()> {
    // "/avatar @someone" looks up that person over REST; bare "/avatar"
    // uses the data already on the interaction (no extra request). The
    // by-id path costs a Discord API call, so rate-limit it.
    let target = super::helpers::parse_target(ctx.args());
    let url = match &target {
        Some(uid) if uid != ctx.user_id() => {
            if svc.rate_limited(&ctx, "avatar_lookup", 3).await? {
                return Ok(());
            }
            ctx.avatar_url_of(uid).await
        }
        _ => ctx.avatar_url().await,
    };
    let url = match url {
        Ok(Some(u)) => u,
        other => {
            // "No avatar" and a failed REST call render the same to the
            // user, but the operator should see the difference.
            if let Err(e) = other {
                tracing::warn!(error = %e, "avatar lookup failed");
            }
            let em = Embed::new()
                .title(svc.tr(&ctx, "avatar_title").await)
                .description(svc.tr(&ctx, "avatar_unavailable").await)
                .color(COLOR_WARN);
            return ctx.reply_with(Reply::embed(em)).await;
        }
    };

    let title = svc.tr(&ctx, "avatar_title").await;
    let mut em = Embed::new().title(title).image(url).color(COLOR_ACCENT);

    // Banner is a separate REST call and we only know how to fetch the
    // caller's - showing your banner under someone else's avatar would
    // just be confusing, so skip it when a target was given.
    let viewing_other = target.as_deref().is_some_and(|t| t != ctx.user_id());
    if !viewing_other {
        if let Ok(Some(banner)) = ctx.banner_url().await {
            let label = svc.tr(&ctx, "avatar_banner").await;
            em = em.field(label, format!("[link]({banner})"));
        }
    }
    ctx.reply_with(Reply::embed(em)).await
}
