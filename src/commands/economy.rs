//! Economy commands: profile, daily, leaderboard, rank, shop, buy, give,
//! gamble, plus XP awarding and achievements.

use super::helpers::{color_for_level, human_duration, pretty_identity};
use super::Services;
use super::{COLOR_ACCENT, COLOR_OK, COLOR_WARN};
use chrono::Utc;
use foukoapi::{
    util::capitalize, util::progress_bar, Button, Ctx, Economy, Embed, Keyboard, Metric, Reply,
    Result,
};
use rand::Rng;

// ---------- XP + profile ----------------------------------------------------

pub(crate) async fn primary_id(ctx: &Ctx, svc: &Services) -> String {
    svc.accounts
        .primary_for(ctx.platform(), ctx.user_id())
        .await
        .unwrap_or_else(|_| format!("{}:{}", ctx.platform(), ctx.user_id()))
}

pub(crate) async fn award_xp(ctx: Ctx, svc: Services) -> Result<()> {
    // Only real typed messages earn XP - button presses also flow through
    // on_message (their callback id doubles as text), and clicking a menu
    // all day shouldn't level anyone up.
    if ctx.is_callback() || ctx.text().trim().is_empty() {
        return Ok(());
    }
    let _gain = svc.econ.add_xp(ctx.platform(), ctx.user_id(), 1).await?;
    // Remember the sender's name so the leaderboard can show it instead of
    // a raw id. Cheap and keeps the display current.
    if let Some(name) = ctx.user_name() {
        if let Err(e) = svc
            .econ
            .set_display_name(ctx.platform(), ctx.user_id(), name)
            .await
        {
            tracing::debug!(error = %e, "could not store display name");
        }
    }
    Ok(())
}

pub(crate) async fn profile(ctx: Ctx, svc: Services) -> Result<()> {
    // `/profile @someone` (or a bare id) shows that person's profile;
    // with no argument you get your own.
    let target = super::helpers::parse_target(ctx.args());
    let (uid, viewing_other) = match &target {
        Some(t) => (t.as_str(), t != ctx.user_id()),
        None => (ctx.user_id(), false),
    };

    let partner = svc.accounts.partner_for(ctx.platform(), uid).await?;
    let w = svc.econ.wallet(ctx.platform(), uid).await;
    let title = svc.econ.title(ctx.platform(), uid).await;
    let streak = daily_streak_of(&ctx, &svc, uid).await;

    // Someone who has never interacted has nothing to show.
    if viewing_other && w.xp == 0 && w.coins == 0 {
        let em = Embed::new()
            .description(svc.tr(&ctx, "profile_unknown").await)
            .color(COLOR_WARN);
        return ctx.reply_with(Reply::embed(em)).await;
    }

    // The viewer's language drives labels; the target's data fills them.
    let lang = svc.lang(&ctx).await;

    let mut platforms: Vec<String> = vec![capitalize(&ctx.platform().to_string())];
    if let Some(p) = partner.as_deref() {
        let other = p.split(':').next().unwrap_or(p);
        if !other.is_empty() {
            let pretty = capitalize(other);
            if !platforms.contains(&pretty) {
                platforms.push(pretty);
            }
        }
    }

    let (lower, upper) = Economy::level_bounds(w.level);
    let (progress, need, bar) = if upper > lower {
        let p = w.xp.saturating_sub(lower);
        let need = upper.saturating_sub(lower);
        (p, need, progress_bar(p, need, 14))
    } else {
        (0, 1, progress_bar(0, 1, 14))
    };

    let title_label = svc.tr(&ctx, "econ_profile_title").await;
    let level_label = svc.tr(&ctx, "econ_profile_level").await;
    let xp_label = svc.tr(&ctx, "econ_profile_xp").await;
    let coins_label = svc.tr(&ctx, "coins").await;
    let lang_label_str = svc.tr(&ctx, "econ_profile_lang").await;
    let platforms_label = svc.tr(&ctx, "econ_profile_platforms").await;
    let streak_label = svc.tr(&ctx, "daily_streak").await;
    let next_fmt = svc
        .trf(&ctx, "econ_profile_next", &[&(w.level + 1).to_string()])
        .await;
    let progress_line = format!("`{bar}` **{progress}/{need}** · {next_fmt}");
    let platforms_pretty = platforms.join(" · ");

    // A player's bought colour wins; otherwise fall back to the
    // level-based accent.
    let accent = svc
        .econ
        .color(ctx.platform(), uid)
        .await
        .unwrap_or_else(|| color_for_level(w.level));

    // Heading: for someone else's profile, lead with their name.
    let owner_name = if viewing_other {
        let primary = svc
            .accounts
            .primary_for(ctx.platform(), uid)
            .await
            .unwrap_or_else(|_| format!("{}:{uid}", ctx.platform()));
        svc.econ
            .display_name_of(&primary)
            .await
            .unwrap_or_else(|| super::helpers::pretty_identity(&primary))
    } else {
        String::new()
    };
    let mut heading = if viewing_other {
        format!("{title_label} - {owner_name}")
    } else {
        title_label.clone()
    };
    if let Some(t) = title.as_deref() {
        if !t.is_empty() {
            heading.push_str(&format!("\n_{t}_"));
        }
    }

    let mut em = Embed::new()
        .title(heading)
        .description(progress_line)
        .field_inline(level_label, format!("**{}**", w.level))
        .field_inline(xp_label, w.xp.to_string())
        .field_inline(coins_label, format!("\u{1FA99} {}", w.coins));
    if !viewing_other {
        em = em.field_inline(lang_label_str, lang.to_uppercase());
    }
    if streak > 0 {
        em = em.field_inline(streak_label, format!("\u{1F525} {streak}"));
    }
    em = em.field(platforms_label, platforms_pretty);

    // Badges earned, rendered as icons. Unknown ids (from a removed badge)
    // are skipped so old profiles never show a blank.
    let earned = svc.econ.achievements(ctx.platform(), uid).await;
    let icons: Vec<&str> = earned
        .iter()
        .filter_map(|id| achievement(id).map(|a| a.icon))
        .collect();
    if !icons.is_empty() {
        let label = svc.tr(&ctx, "econ_profile_badges").await;
        em = em.field(label, icons.join(" "));
    }

    // Account section: only on your own profile, and only the current
    // platform's id - a linked partner's id stays private.
    if !viewing_other {
        let account_label = svc.tr(&ctx, "econ_profile_account").await;
        let id_label = svc
            .trf(
                &ctx,
                "econ_profile_id",
                &[&capitalize(&ctx.platform().to_string())],
            )
            .await;
        let primary = svc
            .accounts
            .primary_for(ctx.platform(), uid)
            .await
            .unwrap_or_else(|_| format!("{}:{uid}", ctx.platform()));
        let mut account = format!("{id_label}: `{uid}`");
        if let Some(name) = svc.econ.display_name_of(&primary).await {
            account = format!("{name}\n{account}");
        }
        em = em.field(account_label, account);
    }

    em = em.color(accent);

    ctx.reply_with(Reply::embed(em)).await
}

// ---------- Achievements ----------------------------------------------------

/// A badge a player can earn. `id` is the stable key stored in the
/// economy; `name_key` points into the string catalogue.
struct Achievement {
    id: &'static str,
    icon: &'static str,
    name_key: &'static str,
}

/// The badge catalogue. Add rows freely; unknown ids stored on a profile
/// are simply ignored, so removing one never breaks old data.
fn achievements() -> &'static [Achievement] {
    &[
        Achievement {
            id: "first_daily",
            icon: "\u{1F305}",
            name_key: "ach_first_daily",
        },
        Achievement {
            id: "streak_7",
            icon: "\u{1F525}",
            name_key: "ach_streak_7",
        },
        Achievement {
            id: "high_roller",
            icon: "\u{1F3B0}",
            name_key: "ach_high_roller",
        },
        Achievement {
            id: "big_spender",
            icon: "\u{1F6CD}\u{FE0F}",
            name_key: "ach_big_spender",
        },
    ]
}

fn achievement(id: &str) -> Option<&'static Achievement> {
    achievements().iter().find(|a| a.id == id)
}

/// `/achievements [@user]` - every badge in the game; the ones not yet
/// earned are shown dimmed. Works for other players too.
pub(crate) async fn achievements_cmd(ctx: Ctx, svc: Services) -> Result<()> {
    let target = super::helpers::parse_target(ctx.args());
    let (uid, viewing_other) = match &target {
        Some(t) => (t.as_str(), t != ctx.user_id()),
        None => (ctx.user_id(), false),
    };

    let lang = svc.lang(&ctx).await;
    let earned = svc.econ.achievements(ctx.platform(), uid).await;

    let mut body = String::new();
    let mut got = 0usize;
    for ach in achievements() {
        let name = svc.i18n.t(&lang, ach.name_key);
        if earned.iter().any(|e| e == ach.id) {
            got += 1;
            body.push_str(&format!("{} **{name}**\n", ach.icon));
        } else {
            // Not earned yet: dimmed - no icon, struck through.
            body.push_str(&format!("\u{2B1C} ~~{name}~~\n"));
        }
    }

    let title = if viewing_other {
        let primary = svc
            .accounts
            .primary_for(ctx.platform(), uid)
            .await
            .unwrap_or_else(|_| format!("{}:{uid}", ctx.platform()));
        let who = svc
            .econ
            .display_name_of(&primary)
            .await
            .unwrap_or_else(|| super::helpers::pretty_identity(&primary));
        svc.trf(&ctx, "ach_list_title_other", &[&who]).await
    } else {
        svc.tr(&ctx, "ach_list_title").await
    };

    let em = Embed::new()
        .title(title)
        .description(body.trim_end())
        .footer(format!("{got}/{}", achievements().len()))
        .color(COLOR_ACCENT);
    ctx.reply_with(Reply::embed(em)).await
}

/// Grant `id` to the caller and, if it's new, congratulate them. Safe to
/// call every time the trigger happens - the economy dedupes.
async fn award(ctx: &Ctx, svc: &Services, id: &str) -> Result<()> {
    let newly = svc
        .econ
        .grant_achievement(ctx.platform(), ctx.user_id(), id)
        .await?;
    if !newly {
        return Ok(());
    }
    let Some(ach) = achievement(id) else {
        return Ok(());
    };
    let name = svc.tr(ctx, ach.name_key).await;
    let em = Embed::new()
        .title(svc.tr(ctx, "ach_unlocked").await)
        .description(format!("{} **{name}**", ach.icon))
        .color(COLOR_OK);
    ctx.reply_with(Reply::embed(em)).await
}

// ---------- Daily reward ----------------------------------------------------

const DAILY_STREAK_PREFIX: &str = "foukobot:daily:streak:";
const DAILY_LAST_PREFIX: &str = "foukobot:daily:last:";
/// Base XP handed out by `/daily`, before the streak bonus.
const DAILY_BASE_XP: u64 = 25;
/// You have this long after the 24h mark to keep a streak alive.
const DAILY_WINDOW_SECS: i64 = 20 * 3600;

/// Daily streak for an arbitrary user id on the caller's platform.
async fn daily_streak_of(ctx: &Ctx, svc: &Services, uid: &str) -> u64 {
    let primary = svc
        .accounts
        .primary_for(ctx.platform(), uid)
        .await
        .unwrap_or_else(|_| format!("{}:{uid}", ctx.platform()));
    svc.storage
        .get(&format!("{DAILY_STREAK_PREFIX}{primary}"))
        .await
        .ok()
        .flatten()
        .and_then(|s| s.parse().ok())
        .unwrap_or(0)
}

pub(crate) async fn daily(ctx: Ctx, svc: Services) -> Result<()> {
    let lang = svc.lang(&ctx).await;
    let primary = primary_id(&ctx, &svc).await;
    let now = Utc::now().timestamp();

    // Same story as the shop: the claim check and the write aren't atomic,
    // so two rapid /daily calls could both pass the 24h check. A short
    // gate closes that window.
    if svc.rate_limited(&ctx, "daily_claim", 5).await? {
        return Ok(());
    }

    let last: i64 = svc
        .storage
        .get(&format!("{DAILY_LAST_PREFIX}{primary}"))
        .await?
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);
    let elapsed = now - last;

    // Once a day, on the dot: block if it's been under 24h since the last
    // claim.
    if last != 0 && elapsed < 24 * 3600 {
        let wait = 24 * 3600 - elapsed;
        let em = Embed::new()
            .title(svc.tr(&ctx, "daily_too_soon_title").await)
            .description(
                svc.trf(&ctx, "daily_too_soon_body", &[&human_duration(wait, &lang)])
                    .await,
            )
            .color(COLOR_WARN);
        return ctx.reply_with(Reply::embed(em)).await;
    }

    // Streak continues if the last claim was within a day-and-a-window,
    // otherwise it resets to 1.
    let prev_streak: u64 = svc
        .storage
        .get(&format!("{DAILY_STREAK_PREFIX}{primary}"))
        .await?
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);
    let streak = if last != 0 && elapsed <= 24 * 3600 + DAILY_WINDOW_SECS {
        prev_streak + 1
    } else {
        1
    };

    // Reward grows with the streak and gets a fat bonus every 7th day.
    let streak_bonus = (streak.min(30) - 1) * 5;
    let weekly_bonus = if streak % 7 == 0 { 100 } else { 0 };
    let reward = DAILY_BASE_XP + streak_bonus + weekly_bonus;

    let gain = svc
        .econ
        .add_xp(ctx.platform(), ctx.user_id(), reward)
        .await?;
    svc.storage
        .set(&format!("{DAILY_LAST_PREFIX}{primary}"), &now.to_string())
        .await?;
    svc.storage
        .set(
            &format!("{DAILY_STREAK_PREFIX}{primary}"),
            &streak.to_string(),
        )
        .await?;

    // Show the coins the grant actually minted, not an estimate: minting
    // depends on which XP boundaries the grant crossed.
    let coins = gain.coins_minted;
    let footer_key = if weekly_bonus > 0 {
        "daily_footer_weekly"
    } else {
        "daily_footer"
    };
    let em = Embed::new()
        .title(svc.tr(&ctx, "daily_title").await)
        .field_inline(
            svc.tr(&ctx, "daily_streak").await,
            format!("\u{1F525} {streak}"),
        )
        .field_inline(svc.tr(&ctx, "daily_reward").await, format!("+{reward} XP"))
        .field_inline(svc.tr(&ctx, "coins").await, format!("\u{1FA99} +{coins}"))
        .footer(svc.tr(&ctx, footer_key).await)
        .color(COLOR_OK);
    ctx.reply_with(Reply::embed(em)).await?;

    // Badges: first claim, and hitting a full week.
    award(&ctx, &svc, "first_daily").await?;
    if streak >= 7 {
        award(&ctx, &svc, "streak_7").await?;
    }
    Ok(())
}

// ---------- Leaderboard + rank ----------------------------------------------

pub(crate) async fn leaderboard(ctx: Ctx, svc: Services) -> Result<()> {
    // /leaderboard coins ranks on coins; anything else (or nothing) is XP.
    let metric = if ctx.args().trim().eq_ignore_ascii_case("coins") {
        Metric::Coins
    } else {
        Metric::Xp
    };
    let top = svc.econ.leaderboard(metric, 10).await;

    if top.is_empty() {
        let em = Embed::new()
            .title(svc.tr(&ctx, "lb_title").await)
            .description(svc.tr(&ctx, "lb_empty").await)
            .color(COLOR_ACCENT);
        return ctx.reply_with(Reply::embed(em)).await;
    }

    let unit = match metric {
        Metric::Coins => "\u{1FA99}",
        Metric::Xp => "XP",
    };
    let medals = ["\u{1F947}", "\u{1F948}", "\u{1F949}"];
    let mut body = String::new();
    for row in &top {
        let place = medals
            .get(row.position)
            .map(|m| (*m).to_string())
            .unwrap_or_else(|| format!("#{}", row.position + 1));
        // Prefer a real display name; fall back to a shortened id only if
        // we've never seen the person send a message.
        let who = match svc.econ.display_name_of(&row.primary).await {
            Some(name) => name,
            None => pretty_identity(&row.primary),
        };
        body.push_str(&format!("{place} **{who}** - {} {unit}\n", row.value));
    }

    // Footer hints at the other ranking.
    let (title_key, footer_key) = match metric {
        Metric::Coins => ("lb_coins_title", "lb_footer_by_xp"),
        Metric::Xp => ("lb_xp_title", "lb_footer_by_coins"),
    };
    let em = Embed::new()
        .title(svc.tr(&ctx, title_key).await)
        .description(body.trim_end())
        .footer(svc.tr(&ctx, footer_key).await)
        .color(COLOR_ACCENT);
    ctx.reply_with(Reply::embed(em)).await
}

pub(crate) async fn rank(ctx: Ctx, svc: Services) -> Result<()> {
    // `/rank @someone` looks up their position; bare `/rank` is yours.
    let target = super::helpers::parse_target(ctx.args());
    let uid = target.as_deref().unwrap_or_else(|| ctx.user_id());

    let total = svc.econ.player_count().await;
    let xp_rank = svc.econ.rank_of(ctx.platform(), uid, Metric::Xp).await;
    let coin_rank = svc.econ.rank_of(ctx.platform(), uid, Metric::Coins).await;

    let Some(xp_rank) = xp_rank else {
        let em = Embed::new()
            .title(svc.tr(&ctx, "rank_title").await)
            .description(svc.tr(&ctx, "rank_none").await)
            .color(COLOR_WARN);
        return ctx.reply_with(Reply::embed(em)).await;
    };

    let mut em = Embed::new()
        .title(svc.tr(&ctx, "rank_title").await)
        .field_inline(
            svc.tr(&ctx, "rank_by_xp").await,
            format!("#{} / {total}", xp_rank.position + 1),
        );
    if let Some(cr) = coin_rank {
        em = em.field_inline(
            svc.tr(&ctx, "rank_by_coins").await,
            format!("#{} / {total}", cr.position + 1),
        );
    }
    em = em.color(COLOR_ACCENT);
    ctx.reply_with(Reply::embed(em)).await
}

// ---------- Shop ------------------------------------------------------------

/// What a shop item does when bought.
enum ShopEffect {
    Title(&'static str),
    Color(u32),
}

struct ShopItem {
    id: &'static str,
    price: u64,
    name_key: &'static str,
    effect: ShopEffect,
}

/// The full catalogue. Prices are in coins.
fn shop_items() -> &'static [ShopItem] {
    &[
        ShopItem {
            id: "title_novice",
            price: 10,
            name_key: "shop_item_title_novice",
            effect: ShopEffect::Title("\u{1F331} Novice"),
        },
        ShopItem {
            id: "title_regular",
            price: 50,
            name_key: "shop_item_title_regular",
            effect: ShopEffect::Title("\u{2615} Regular"),
        },
        ShopItem {
            id: "title_legend",
            price: 250,
            name_key: "shop_item_title_legend",
            effect: ShopEffect::Title("\u{1F451} Legend"),
        },
        ShopItem {
            id: "color_teal",
            price: 40,
            name_key: "shop_item_color_teal",
            effect: ShopEffect::Color(0x00C2A8),
        },
        ShopItem {
            id: "color_gold",
            price: 120,
            name_key: "shop_item_color_gold",
            effect: ShopEffect::Color(0xF5B301),
        },
        ShopItem {
            id: "color_crimson",
            price: 120,
            name_key: "shop_item_color_crimson",
            effect: ShopEffect::Color(0xE02B4B),
        },
    ]
}

pub(crate) async fn shop(ctx: Ctx, svc: Services) -> Result<()> {
    // Button press: `shop:<invoker>:<item_id>` - buy that item. Guard so
    // only the person who opened the shop can spend their coins.
    if let Some(data) = ctx.callback_data() {
        if let Some(rest) = data.strip_prefix("shop:") {
            let mut parts = rest.splitn(2, ':');
            let invoker = parts.next().unwrap_or("");
            let item_id = parts.next().unwrap_or("");
            if invoker != ctx.user_id() {
                return ctx
                    .reply_temporary(svc.tr(&ctx, "not_your_button").await, 5)
                    .await;
            }
            return purchase(&ctx, &svc, item_id).await;
        }
    }

    let lang = svc.lang(&ctx).await;
    let balance = svc.econ.coins(ctx.platform(), ctx.user_id()).await;

    let mut body = String::new();
    for item in shop_items() {
        let name = svc.i18n.t(&lang, item.name_key);
        body.push_str(&format!("**{name}** · \u{1FA99} {}\n", item.price));
    }

    // One button per item, chunked two per row for a tidy grid.
    let invoker = ctx.user_id().to_owned();
    let mut kb = Keyboard::new();
    for chunk in shop_items().chunks(2) {
        let row: Vec<Button> = chunk
            .iter()
            .map(|item| {
                let name = svc.i18n.t(&lang, item.name_key);
                Button::callback(
                    format!("{name} · {}", item.price),
                    format!("shop:{invoker}:{}", item.id),
                )
            })
            .collect();
        kb = kb.row(row);
    }

    let em = Embed::new()
        .title(svc.tr(&ctx, "shop_title").await)
        .description(body.trim_end())
        .field(
            svc.tr(&ctx, "shop_balance").await,
            format!("\u{1FA99} {balance}"),
        )
        .footer(svc.tr(&ctx, "shop_footer").await)
        .color(COLOR_ACCENT);
    ctx.reply_with(Reply::embed(em).keyboard(kb)).await
}

pub(crate) async fn buy(ctx: Ctx, svc: Services) -> Result<()> {
    let wanted = ctx.args().trim().to_ascii_lowercase();
    if wanted.is_empty() {
        let em = Embed::new()
            .title(svc.tr(&ctx, "buy_title").await)
            .description(svc.tr(&ctx, "buy_usage").await)
            .color(COLOR_WARN);
        return ctx.reply_with(Reply::embed(em)).await;
    }
    purchase(&ctx, &svc, &wanted).await
}

/// Shared purchase logic used by both the `/buy` command and the shop's
/// inline buttons.
async fn purchase(ctx: &Ctx, svc: &Services, item_id: &str) -> Result<()> {
    // A double-tap on the shop button lands here twice before the first
    // deduction commits (balance check and spend aren't atomic), so gate
    // purchases behind a short cooldown.
    if svc.rate_limited(ctx, "shop_buy", 3).await? {
        return Ok(());
    }

    let Some(item) = shop_items().iter().find(|i| i.id == item_id) else {
        let em = Embed::new()
            .title(svc.tr(ctx, "shop_unknown_title").await)
            .description(svc.tr(ctx, "shop_unknown_body").await)
            .color(COLOR_WARN);
        return ctx.reply_with(Reply::embed(em)).await;
    };

    let balance = svc.econ.coins(ctx.platform(), ctx.user_id()).await;
    if balance < item.price {
        let em = Embed::new()
            .title(svc.tr(ctx, "shop_not_enough_title").await)
            .description(
                svc.trf(
                    ctx,
                    "shop_not_enough_body",
                    &[&item.price.to_string(), &balance.to_string()],
                )
                .await,
            )
            .color(COLOR_WARN);
        return ctx.reply_with(Reply::embed(em)).await;
    }

    svc.econ
        .add_coins(ctx.platform(), ctx.user_id(), -(item.price as i64))
        .await?;
    match &item.effect {
        ShopEffect::Title(t) => {
            svc.econ.set_title(ctx.platform(), ctx.user_id(), t).await?;
        }
        ShopEffect::Color(c) => {
            svc.econ
                .set_color(ctx.platform(), ctx.user_id(), *c)
                .await?;
        }
    }

    let name = svc.tr(ctx, item.name_key).await;
    let em = Embed::new()
        .title(svc.tr(ctx, "shop_purchased_title").await)
        .description(svc.trf(ctx, "shop_purchased_body", &[&name]).await)
        .footer(svc.tr(ctx, "shop_purchased_footer").await)
        .color(COLOR_OK);
    ctx.reply_with(Reply::embed(em)).await?;
    award(ctx, svc, "big_spender").await?;
    Ok(())
}

// ---------- Give (coin transfer) --------------------------------------------

pub(crate) async fn give(ctx: Ctx, svc: Services) -> Result<()> {
    let args = ctx.args().trim();

    // Expect "<target> <amount>". Target is a raw user id or a mention we
    // strip down to digits, since that's all we can resolve without a
    // per-platform name lookup.
    let mut parts = args.split_whitespace();
    let target_raw = parts.next().unwrap_or("");
    let amount_raw = parts.next().unwrap_or("");
    let target = target_raw
        .trim_start_matches("<@")
        .trim_start_matches('!')
        .trim_end_matches('>')
        .trim_start_matches('@');

    // Missing pieces get the usage text; a present-but-broken amount
    // ("-5", "10abc", overflow) gets its own message so the user knows
    // what exactly went wrong.
    if target.is_empty() || amount_raw.is_empty() {
        let em = Embed::new()
            .title(svc.tr(&ctx, "give_title").await)
            .description(svc.tr(&ctx, "give_usage").await)
            .color(COLOR_WARN);
        return ctx.reply_with(Reply::embed(em)).await;
    }
    let amount: u64 = match amount_raw.parse() {
        Ok(n) if n > 0 => n,
        _ => {
            let em = Embed::new()
                .title(svc.tr(&ctx, "give_title").await)
                .description(svc.tr(&ctx, "amount_invalid").await)
                .color(COLOR_WARN);
            return ctx.reply_with(Reply::embed(em)).await;
        }
    };

    if target == ctx.user_id() {
        let em = Embed::new()
            .title(svc.tr(&ctx, "give_title").await)
            .description(svc.tr(&ctx, "give_self").await)
            .color(COLOR_WARN);
        return ctx.reply_with(Reply::embed(em)).await;
    }

    // Guard against fat-fingered ids: only send to someone who has already
    // shown up in the economy, so coins don't vanish into a typo'd wallet.
    let recipient = svc.econ.wallet(ctx.platform(), target).await;
    if recipient.xp == 0 && recipient.coins == 0 {
        let em = Embed::new()
            .title(svc.tr(&ctx, "give_title").await)
            .description(svc.tr(&ctx, "give_unknown").await)
            .color(COLOR_WARN);
        return ctx.reply_with(Reply::embed(em)).await;
    }

    match svc
        .econ
        .transfer(
            (ctx.platform(), ctx.user_id()),
            (ctx.platform(), target),
            amount,
        )
        .await
    {
        Ok(remaining) => {
            let em = Embed::new()
                .title(svc.tr(&ctx, "give_sent_title").await)
                .description(
                    svc.trf(
                        &ctx,
                        "give_sent_body",
                        &[&amount.to_string(), &remaining.to_string()],
                    )
                    .await,
                )
                .color(COLOR_OK);
            ctx.reply_with(Reply::embed(em)).await
        }
        Err(_) => {
            let em = Embed::new()
                .title(svc.tr(&ctx, "give_title").await)
                .description(svc.tr(&ctx, "give_failed").await)
                .color(COLOR_WARN);
            ctx.reply_with(Reply::embed(em)).await
        }
    }
}

// ---------- Gamble ----------------------------------------------------------

/// Largest single bet `/gamble` accepts, to keep the economy sane.
const GAMBLE_MAX_BET: u64 = 500;

pub(crate) async fn gamble(ctx: Ctx, svc: Services) -> Result<()> {
    let lang = svc.lang(&ctx).await;

    // Validate the bet before anything touches the cooldown, so a typo'd
    // `/gamble abc` doesn't burn the window.
    let bet_raw = ctx.args().trim();
    if bet_raw.is_empty() {
        let em = Embed::new()
            .title(svc.tr(&ctx, "gamble_title").await)
            .description(svc.tr(&ctx, "gamble_prompt").await)
            .color(COLOR_WARN);
        return ctx.reply_with(Reply::embed(em)).await;
    }
    let bet: u64 = match bet_raw.parse() {
        Ok(n) if n > 0 => n,
        _ => {
            let em = Embed::new()
                .title(svc.tr(&ctx, "gamble_title").await)
                .description(svc.tr(&ctx, "amount_invalid").await)
                .color(COLOR_WARN);
            return ctx.reply_with(Reply::embed(em)).await;
        }
    };

    // Cap the stake so one lucky (or rich) player can't warp the coin
    // leaderboard in a couple of taps.
    if bet > GAMBLE_MAX_BET {
        let em = Embed::new()
            .title(svc.tr(&ctx, "gamble_title").await)
            .description(
                svc.trf(&ctx, "gamble_max", &[&GAMBLE_MAX_BET.to_string()])
                    .await,
            )
            .color(COLOR_WARN);
        return ctx.reply_with(Reply::embed(em)).await;
    }

    // Light rate limit so nobody spams a fortune (or a loss) in a second.
    let wait = svc
        .econ
        .cooldown_remaining(ctx.platform(), ctx.user_id(), "gamble", 10)
        .await;
    if wait > 0 {
        let em = Embed::new()
            .title(svc.tr(&ctx, "gamble_slow_title").await)
            .description(
                svc.trf(&ctx, "gamble_wait", &[&human_duration(wait, &lang)])
                    .await,
            )
            .color(COLOR_WARN);
        return ctx.reply_with(Reply::embed(em)).await;
    }
    // Stamp the cooldown before touching balances: two overlapping calls
    // must not both slip past the balance check, or a double-tap turns
    // into a free bet (the second debit clamps at zero but the win pays).
    svc.econ
        .touch_cooldown(ctx.platform(), ctx.user_id(), "gamble")
        .await?;

    let balance = svc.econ.coins(ctx.platform(), ctx.user_id()).await;
    if balance < bet {
        let em = Embed::new()
            .title(svc.tr(&ctx, "gamble_not_enough_title").await)
            .description(svc.trf(&ctx, "gamble_short", &[&balance.to_string()]).await)
            .color(COLOR_WARN);
        return ctx.reply_with(Reply::embed(em)).await;
    }

    // 45% to win: pays even money. Slight house edge keeps the economy
    // from inflating.
    let win = {
        let mut rng = rand::thread_rng();
        rng.gen_bool(0.45)
    };
    let delta = if win { bet as i64 } else { -(bet as i64) };
    let new_balance = svc
        .econ
        .add_coins(ctx.platform(), ctx.user_id(), delta)
        .await?;

    let (title, body) = if win {
        (
            svc.tr(&ctx, "gamble_win_title").await,
            svc.trf(
                &ctx,
                "gamble_win_body",
                &[&bet.to_string(), &new_balance.to_string()],
            )
            .await,
        )
    } else {
        (
            svc.tr(&ctx, "gamble_lose_title").await,
            svc.trf(
                &ctx,
                "gamble_lose_body",
                &[&bet.to_string(), &new_balance.to_string()],
            )
            .await,
        )
    };
    let em =
        Embed::new()
            .title(title)
            .description(body)
            .color(if win { COLOR_OK } else { COLOR_WARN });
    ctx.reply_with(Reply::embed(em)).await?;

    // Landing a big win earns a badge.
    if win && bet >= 100 {
        award(&ctx, &svc, "high_roller").await?;
    }
    Ok(())
}
