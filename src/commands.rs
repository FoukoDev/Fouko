//! FoukoBot command set.

use chrono::Utc;
use foukoapi::{
    util::{capitalize, progress_bar, urlencode},
    Accounts, AnyStorage, Bot, Button, Ctx, Embed, Keyboard, PlatformKind, Reply, Result,
    TextMatch,
};
use rand::{seq::SliceRandom, Rng};
use std::collections::HashMap;

/// Services shared by every command.
#[derive(Clone)]
pub struct Services {
    pub storage: AnyStorage,
    pub accounts: Accounts,
    pub weather_key: Option<String>,
}

/// Wire every command into a [`Bot`].
pub fn register(bot: Bot, svc: Services) -> Bot {
    let s_help = svc.clone();
    let s_profile = svc.clone();
    let s_weather = svc.clone();
    let s_settings = svc.clone();
    let s_help_text = svc.clone();
    let s_about = svc.clone();
    let s_bot = svc.clone();
    let xp_svc = svc.clone();
    let accounts_for_bot = svc.accounts.clone();
    bot.with_accounts(accounts_for_bot)
        .on_message(move |ctx| {
            let s = xp_svc.clone();
            async move { award_xp(ctx, s).await }
        })
        // /start and /help: our own localised body, described for /help list.
        .command_described_i18n(
            "/start",
            i18n_map(&[
                ("en", "show this bot's command list"),
                ("ru", "показать список команд"),
            ]),
            {
                let s = s_help.clone();
                move |ctx| help(ctx, s.clone())
            },
        )
        .command_described_i18n(
            "/ping",
            i18n_map(&[("en", "health check"), ("ru", "проверка связи")]),
            ping,
        )
        .command_described_i18n(
            "/about",
            i18n_map(&[("en", "short info about the bot"), ("ru", "коротко о боте")]),
            move |ctx| about(ctx, s_about.clone()),
        )
        .command_described_i18n(
            "/bot",
            i18n_map(&[
                ("en", "full bot info: stack, features, links"),
                ("ru", "подробно о боте: стек, возможности, ссылки"),
            ]),
            move |ctx| bot_info(ctx, s_bot.clone()),
        )
        .command_described_i18n(
            "/time",
            i18n_map(&[
                ("en", "current time (optional UTC offset: /time +10 or /time -5:30)"),
                ("ru", "текущее время (можно с UTC-сдвигом: /time +10 или /time -5:30)"),
            ]),
            {
                let s = svc.clone();
                move |ctx| time(ctx, s.clone())
            },
        )
        .command_described_i18n(
            "/coin",
            i18n_map(&[("en", "flip a coin"), ("ru", "подбросить монетку")]),
            coin,
        )
        .command_described_i18n(
            "/roll",
            i18n_map(&[
                ("en", "roll dice, e.g. /roll 3d6"),
                ("ru", "бросить кубики, например /roll 3d6"),
            ]),
            roll,
        )
        .command_described_i18n(
            "/8ball",
            i18n_map(&[
                ("en", "ask a question, get an answer"),
                ("ru", "задай вопрос, получи ответ"),
            ]),
            eight_ball,
        )
        .command_described_i18n(
            "/cat",
            i18n_map(&[
                ("en", "random cat picture (Telegram / Discord only)"),
                ("ru", "случайный котик (только Telegram / Discord)"),
            ]),
            cat,
        )
        .only_on("/cat", &[PlatformKind::Telegram, PlatformKind::Discord])
        .command_described_i18n(
            "/menu",
            i18n_map(&[
                ("en", "interactive menu with buttons"),
                ("ru", "меню с кнопками"),
            ]),
            menu,
        )
        .command_described_i18n(
            "/profile",
            i18n_map(&[
                ("en", "show your XP, level and linked platforms"),
                ("ru", "твой опыт, уровень и связанные платформы"),
            ]),
            move |ctx| profile(ctx, s_profile.clone()),
        )
        .command_described_i18n(
            "/weather",
            i18n_map(&[
                ("en", "current weather for a city"),
                ("ru", "погода в городе"),
            ]),
            move |ctx| weather(ctx, s_weather.clone()),
        )
        .command_described_i18n(
            "/settings",
            i18n_map(&[
                ("en", "personal settings (DM only)"),
                ("ru", "личные настройки (только в ЛС)"),
            ]),
            move |ctx| settings_cmd(ctx, s_settings.clone()),
        )
        .command_described_i18n(
            "/joke",
            i18n_map(&[
                ("en", "a random programmer joke"),
                ("ru", "случайная шутка про программистов"),
            ]),
            joke,
        )
        .command_described_i18n(
            "/choose",
            i18n_map(&[
                ("en", "pick one at random: /choose a, b, c"),
                ("ru", "выбрать случайное: /choose а, б, в"),
            ]),
            choose,
        )
        .command_described_i18n(
            "/reverse",
            i18n_map(&[
                ("en", "reverse your text: /reverse hello"),
                ("ru", "перевернуть текст: /reverse привет"),
            ]),
            reverse,
        )
        .command_described_i18n(
            "/emoji",
            i18n_map(&[
                ("en", "a random cute emoji"),
                ("ru", "случайный милый эмодзи"),
            ]),
            emoji,
        )
        .command_described_i18n(
            "/qr",
            i18n_map(&[
                ("en", "generate a QR code, e.g. /qr hello"),
                ("ru", "сгенерировать QR-код, например /qr привет"),
            ]),
            qr,
        )
        .command_described_i18n(
            "/shorten",
            i18n_map(&[
                ("en", "shorten a URL: /shorten https://example.com/long"),
                ("ru", "сократить ссылку: /shorten https://example.com/long"),
            ]),
            shorten,
        )
        .command_described_i18n(
            "/stats",
            i18n_map(&[
                ("en", "bot uptime and runtime stats"),
                ("ru", "аптайм бота и статистика рантайма"),
            ]),
            stats,
        )
        .text_command("help", TextMatch::Exact, {
            let s = s_help_text.clone();
            move |ctx| help(ctx, s.clone())
        })
        .text_command("/", TextMatch::Exact, {
            let s = s_help_text.clone();
            move |ctx| help(ctx, s.clone())
        })
        .text_command("/?", TextMatch::Exact, {
            let s = s_help_text.clone();
            move |ctx| help(ctx, s.clone())
        })
        .text_command("ping", TextMatch::Exact, ping)
        .with_default_link_command()
        .with_default_lang_command(["en", "ru"])
        .with_default_help()
}

/// Turn `[(lang, desc), ...]` into the HashMap that `command_described_i18n` wants.
fn i18n_map(pairs: &[(&str, &str)]) -> HashMap<String, String> {
    pairs
        .iter()
        .map(|(k, v)| ((*k).to_owned(), (*v).to_owned()))
        .collect()
}

async fn settings_cmd(ctx: Ctx, svc: Services) -> Result<()> {
    if !ctx.is_dm() {
        let em = Embed::new()
            .title("\u{1F512} Только в ЛС / DM Only")
            .description(
                "Эта команда работает только в личке с ботом.\nThis command only works in a private chat with the bot.",
            )
            .color(COLOR_WARN);
        return ctx.reply_with(Reply::embed(em)).await;
    }

    let lang = svc
        .accounts
        .lang_for(ctx.platform(), ctx.user_id())
        .await
        .unwrap_or_else(|_| "en".into());

    let partner = svc
        .accounts
        .partner_for(ctx.platform(), ctx.user_id())
        .await?;
    let primary = primary_id(&ctx, &svc).await;
    let primary_platform = primary.split(':').next().unwrap_or(&primary).to_owned();

    let linked = match &partner {
        Some(p) => {
            let me = ctx.platform().to_string();
            let other = p.split(':').next().unwrap_or(p);
            format!("{} \u{2194} {}", capitalize(&me), capitalize(other))
        }
        None => match lang.as_str() {
            "ru" => "нет".to_owned(),
            _ => "none".to_owned(),
        },
    };

    let (title, lang_label, platform_label, linked_label, footer) = match lang.as_str() {
        "ru" => (
            "\u{2699}\u{FE0F} Настройки",
            "Язык",
            "Основная платформа",
            "Связанные аккаунты",
            "Поменять язык: /lang   •   Привязать/отвязать: /link",
        ),
        _ => (
            "\u{2699}\u{FE0F} Settings",
            "Language",
            "Primary Platform",
            "Linked Accounts",
            "Change language: /lang   •   Link/unlink: /link",
        ),
    };

    let em = Embed::new()
        .title(title)
        .field_inline(lang_label, lang.to_uppercase())
        .field_inline(platform_label, capitalize(&primary_platform))
        .field(linked_label, linked)
        .footer(footer)
        .color(COLOR_ACCENT);
    ctx.reply_with(Reply::embed(em)).await
}

// ---------- XP + profile ----------------------------------------------------

const XP_KEY_PREFIX: &str = "foukobot:xp:";
const COINS_KEY_PREFIX: &str = "foukobot:coins:";

async fn primary_id(ctx: &Ctx, svc: &Services) -> String {
    svc.accounts
        .primary_for(ctx.platform(), ctx.user_id())
        .await
        .unwrap_or_else(|_| format!("{}:{}", ctx.platform(), ctx.user_id()))
}

async fn award_xp(ctx: Ctx, svc: Services) -> Result<()> {
    if ctx.text().trim().is_empty() {
        return Ok(());
    }
    let primary = primary_id(&ctx, &svc).await;
    let xp_key = format!("{XP_KEY_PREFIX}{primary}");
    let current_xp: u64 = svc
        .storage
        .get(&xp_key)
        .await?
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);
    let new_xp = current_xp.saturating_add(1);
    svc.storage.set(&xp_key, &new_xp.to_string()).await?;

    // One coin per 10 XP.
    if new_xp % 10 == 0 {
        let coins_key = format!("{COINS_KEY_PREFIX}{primary}");
        let current_coins: u64 = svc
            .storage
            .get(&coins_key)
            .await?
            .and_then(|s| s.parse().ok())
            .unwrap_or(0);
        let new_coins = current_coins.saturating_add(1);
        svc.storage.set(&coins_key, &new_coins.to_string()).await?;
    }
    Ok(())
}

fn level_for(xp: u64) -> u32 {
    // L*(L+1)*10 xp to reach level L: 20, 60, 120, 200 ...
    let mut level: u32 = 0;
    loop {
        let next = (level as u64 + 1) * (level as u64 + 2) * 10;
        if xp < next {
            return level;
        }
        level += 1;
        if level > 1000 {
            return level;
        }
    }
}

async fn profile(ctx: Ctx, svc: Services) -> Result<()> {
    let primary = primary_id(&ctx, &svc).await;
    let partner = svc
        .accounts
        .partner_for(ctx.platform(), ctx.user_id())
        .await?;
    let xp_key = format!("{XP_KEY_PREFIX}{primary}");
    let xp: u64 = svc
        .storage
        .get(&xp_key)
        .await?
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);
    let coins_key = format!("{COINS_KEY_PREFIX}{primary}");
    let coins: u64 = svc
        .storage
        .get(&coins_key)
        .await?
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);
    let level = level_for(xp);

    let lang = svc
        .accounts
        .lang_for(ctx.platform(), ctx.user_id())
        .await
        .unwrap_or_else(|_| "en".into());

    let me_platform = ctx.platform().to_string();
    let mut platforms: Vec<String> = vec![capitalize(&me_platform)];
    if let Some(p) = partner.as_deref() {
        let other = p.split(':').next().unwrap_or(p);
        if !other.is_empty() {
            let pretty = capitalize(other);
            if !platforms.contains(&pretty) {
                platforms.push(pretty);
            }
        }
    }

    let lower = if level == 0 {
        0
    } else {
        (level as u64 - 1) * (level as u64) * 10
    };
    let upper = (level as u64) * (level as u64 + 1) * 10;
    let (progress, need, bar) = if upper > lower {
        let p = xp.saturating_sub(lower);
        let need = upper.saturating_sub(lower);
        (p, need, progress_bar(p, need, 14))
    } else {
        (0, 1, progress_bar(0, 1, 14))
    };

    let (title, level_label, xp_label, coins_label, lang_label_str, platforms_label, next_fmt) =
        match lang.as_str() {
            "ru" => (
                "\u{1F464} Профиль",
                "Уровень",
                "Опыт",
                "Монеты",
                "Язык",
                "Платформы",
                format!("до уровня {}", level + 1),
            ),
            _ => (
                "\u{1F464} Profile",
                "Level",
                "XP",
                "Coins",
                "Language",
                "Platforms",
                format!("to level {}", level + 1),
            ),
        };
    let progress_line = format!("`{bar}` **{progress}/{need}** · {next_fmt}");
    let platforms_pretty = platforms.join(" · ");

    let em = Embed::new()
        .title(title)
        .description(progress_line)
        .field_inline(level_label, format!("**{level}**"))
        .field_inline(xp_label, xp.to_string())
        .field_inline(coins_label, format!("\u{1FA99} {coins}"))
        .field_inline(lang_label_str, lang.to_uppercase())
        .field(platforms_label, platforms_pretty)
        .color(color_for_level(level));

    ctx.reply_with(Reply::embed(em)).await
}

/// Embed accent colour by level (cosmetic).
fn color_for_level(level: u32) -> u32 {
    match level {
        0..=2 => 0x7A5BE8,   // violet
        3..=6 => 0x5B8DEF,   // blue
        7..=14 => 0x00C2A8,  // teal
        15..=29 => 0xF59F00, // orange
        _ => 0xE02B6B,       // hot pink for the dedicated
    }
}

// ---------- Basic commands --------------------------------------------------

/// Shared colour palette for bot-side embeds.
const COLOR_ACCENT: u32 = 0x7A5BE8;
const COLOR_OK: u32 = 0x43B581;
const COLOR_WARN: u32 = 0xF59F00;

/// `/start` / `help` / `/` / `/?` — friendly nudge toward the full `/help`.
async fn help(ctx: Ctx, svc: Services) -> Result<()> {
    let lang = svc
        .accounts
        .lang_for(ctx.platform(), ctx.user_id())
        .await
        .unwrap_or_else(|_| "en".into());
    let (title, desc, handy_label) = match lang.as_str() {
        "ru" => (
            "\u{1F44B} FoukoBot",
            "Один бот — много платформ. Напиши `/help`, чтобы увидеть все команды.",
            "Попробуй",
        ),
        _ => (
            "\u{1F44B} FoukoBot",
            "One bot, every chat platform. Type `/help` to see every command.",
            "Try",
        ),
    };
    let em = Embed::new()
        .title(title)
        .description(desc)
        .field(handy_label, "`/menu` · `/profile` · `/bot` · `/link` · `/lang`")
        .footer("bot.fouko.xyz")
        .url("https://bot.fouko.xyz")
        .color(COLOR_ACCENT);
    ctx.reply_with(Reply::embed(em)).await
}

async fn ping(ctx: Ctx) -> Result<()> {
    let em = Embed::new()
        .title("\u{1F3D3} Pong")
        .description("Bot is alive and well.")
        .color(COLOR_OK);
    ctx.reply_with(Reply::embed(em)).await
}

/// `/about` — short info card.
async fn about(ctx: Ctx, svc: Services) -> Result<()> {
    let lang = svc
        .accounts
        .lang_for(ctx.platform(), ctx.user_id())
        .await
        .unwrap_or_else(|_| "en".into());
    let version = env!("CARGO_PKG_VERSION");
    let (title, desc, ver_label, lang_label, platforms_label, links_label, footer) =
        match lang.as_str() {
            "ru" => (
                "\u{1F31F} О боте",
                "Открытый кроссплатформенный бот на Rust. Один код — Telegram и Discord сразу.",
                "Версия",
                "Язык",
                "Платформы",
                "Ссылки",
                "Напиши /help, чтобы увидеть все команды",
            ),
            _ => (
                "\u{1F31F} About",
                "An open-source cross-platform bot written in Rust. One codebase, both Telegram and Discord.",
                "Version",
                "Language",
                "Platforms",
                "Links",
                "Type /help to see every command",
            ),
        };
    let em = Embed::new()
        .title(title)
        .url("https://bot.fouko.xyz")
        .description(desc)
        .field_inline(ver_label, format!("`{version}`"))
        .field_inline(lang_label, "Rust \u{1F980}")
        .field_inline(platforms_label, "Telegram · Discord")
        .field(
            links_label,
            "[bot.fouko.xyz](https://bot.fouko.xyz) · [api.fouko.xyz](https://api.fouko.xyz) · [GitHub](https://github.com/FoukoDev)",
        )
        .footer(footer)
        .color(COLOR_ACCENT);
    ctx.reply_with(Reply::embed(em)).await
}

/// `/bot` — richer info card aimed at curious users / server owners.
async fn bot_info(ctx: Ctx, svc: Services) -> Result<()> {
    let lang = svc
        .accounts
        .lang_for(ctx.platform(), ctx.user_id())
        .await
        .unwrap_or_else(|_| "en".into());
    let version = env!("CARGO_PKG_VERSION");
    let (title, tagline, about_label, about_text, stack_label, stack_text,
         features_label, features_text, links_label, links_text, footer) =
        match lang.as_str() {
            "ru" => (
                "\u{1F916} FoukoBot",
                "Один бот — сразу в Telegram и Discord.",
                "О боте",
                "Открытый проект на Rust. Уровни, монеты, связывание аккаунтов между платформами, настройки, красивые эмбеды — всё в одном боте.",
                "Стек",
                concat!(
                    "`Rust` · `tokio` · `teloxide` · `serenity` · `SQLite`\n",
                    "Ядро: [FoukoApi](https://api.fouko.xyz) — мини-фреймворк для таких ботов."
                ),
                "Возможности",
                concat!(
                    "• Синхронные профиль и XP между Telegram и Discord\n",
                    "• Связывание аккаунтов по коду (`/link`)\n",
                    "• Переключение языка на лету (`/lang`)\n",
                    "• Погода, время, кубики, монетка, 8-ball, кот, шутки и прочее\n",
                    "• Inline-кнопки в `/menu`"
                ),
                "Ссылки",
                "[Сайт](https://bot.fouko.xyz) · [API](https://api.fouko.xyz) · [GitHub](https://github.com/FoukoDev)",
                "/help — полный список команд",
            ),
            _ => (
                "\u{1F916} FoukoBot",
                "One bot, Telegram and Discord at the same time.",
                "About",
                "Open-source bot written in Rust. Levels, coins, cross-platform account linking, settings and pretty embeds — all in one place.",
                "Stack",
                concat!(
                    "`Rust` · `tokio` · `teloxide` · `serenity` · `SQLite`\n",
                    "Core: [FoukoApi](https://api.fouko.xyz) — a tiny framework made for bots like this."
                ),
                "Features",
                concat!(
                    "• Synced profile & XP between Telegram and Discord\n",
                    "• Code-based account linking (`/link`)\n",
                    "• On-the-fly language switch (`/lang`)\n",
                    "• Weather, time, dice, coin flip, 8-ball, cat, jokes and more\n",
                    "• Inline buttons via `/menu`"
                ),
                "Links",
                "[Website](https://bot.fouko.xyz) · [API](https://api.fouko.xyz) · [GitHub](https://github.com/FoukoDev)",
                "/help — full command list",
            ),
        };
    let em = Embed::new()
        .title(title)
        .url("https://bot.fouko.xyz")
        .description(tagline)
        .field(about_label, about_text)
        .field(stack_label, stack_text)
        .field(features_label, features_text)
        .field_inline("Version", format!("`{version}`"))
        .field_inline("Platforms", "Telegram · Discord")
        .field(links_label, links_text)
        .footer(footer)
        .color(COLOR_ACCENT);
    ctx.reply_with(Reply::embed(em)).await
}

/// `/time`, `/time +10`, `/time -5:30`.
async fn time(ctx: Ctx, svc: Services) -> Result<()> {
    let lang = svc
        .accounts
        .lang_for(ctx.platform(), ctx.user_id())
        .await
        .unwrap_or_else(|_| "en".into());
    let arg = ctx.args().trim();

    let (title, when_label, tz_label) = match lang.as_str() {
        "ru" => ("\u{1F550} Время", "Сейчас", "Часовой пояс"),
        _ => ("\u{1F550} Time", "Now", "Timezone"),
    };

    if arg.is_empty() {
        let now = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
        let em = Embed::new()
            .title(title)
            .field_inline(when_label, now)
            .field_inline(tz_label, "UTC")
            .color(COLOR_ACCENT);
        return ctx.reply_with(Reply::embed(em)).await;
    }

    match parse_utc_offset(arg) {
        Some(minutes) => {
            let shifted = Utc::now() + chrono::Duration::minutes(minutes as i64);
            let rendered = shifted.format("%Y-%m-%d %H:%M:%S").to_string();
            let suffix = format_offset(minutes);
            let em = Embed::new()
                .title(title)
                .field_inline(when_label, rendered)
                .field_inline(tz_label, format!("UTC{suffix}"))
                .color(COLOR_ACCENT);
            ctx.reply_with(Reply::embed(em)).await
        }
        None => {
            let desc = match lang.as_str() {
                "ru" => "не понял сдвиг. примеры: `/time +10`, `/time -5:30`, `/time 0`",
                _ => "couldn't parse the offset. examples: `/time +10`, `/time -5:30`, `/time 0`",
            };
            let em = Embed::new()
                .title(title)
                .description(desc)
                .color(COLOR_WARN);
            ctx.reply_with(Reply::embed(em)).await
        }
    }
}

/// Parse `+10`, `-5:30`, `0`, `+03:45` etc. into signed minutes, or `None`.
fn parse_utc_offset(spec: &str) -> Option<i32> {
    let s = spec.trim();
    if s.is_empty() {
        return None;
    }
    let (sign, rest) = match s.as_bytes()[0] {
        b'+' => (1_i32, &s[1..]),
        b'-' => (-1_i32, &s[1..]),
        _ => (1_i32, s),
    };
    let (hh, mm) = match rest.split_once(':') {
        Some((h, m)) => (h, m),
        None => (rest, "0"),
    };
    let hours: i32 = hh.parse().ok()?;
    let minutes: i32 = mm.parse().ok()?;
    if !(0..=23).contains(&hours) || !(0..=59).contains(&minutes) {
        return None;
    }
    let total = sign * (hours * 60 + minutes);
    if !(-12 * 60..=14 * 60).contains(&total) {
        return None;
    }
    Some(total)
}

/// Render a signed minute offset as `+10`, `-5:30`, `+00:00` etc.
fn format_offset(minutes: i32) -> String {
    let sign = if minutes < 0 { '-' } else { '+' };
    let m = minutes.abs();
    let hh = m / 60;
    let mm = m % 60;
    if mm == 0 {
        format!("{sign}{hh}")
    } else {
        format!("{sign}{hh}:{mm:02}")
    }
}

async fn coin(ctx: Ctx) -> Result<()> {
    let heads = {
        let mut rng = rand::thread_rng();
        rng.gen_bool(0.5)
    };
    let (side_en, side_ru, icon) = if heads {
        ("Heads", "Орёл", "\u{1FA99}")
    } else {
        ("Tails", "Решка", "\u{1F4B0}")
    };
    let em = Embed::new()
        .title(format!("{icon} Coin Flip"))
        .description(format!("**{side_en}** / **{side_ru}**"))
        .color(COLOR_ACCENT);
    ctx.reply_with(Reply::embed(em)).await
}

async fn roll(ctx: Ctx) -> Result<()> {
    let args = ctx.args();
    let spec = if args.is_empty() { "1d6" } else { args };
    match parse_dice(spec) {
        Some((count, sides)) if (1..=100).contains(&count) && (2..=1000).contains(&sides) => {
            let (rolls, total): (Vec<u32>, u32) = {
                let mut rng = rand::thread_rng();
                let r: Vec<u32> = (0..count).map(|_| rng.gen_range(1..=sides)).collect();
                let t: u32 = r.iter().sum();
                (r, t)
            };
            let joined = rolls
                .iter()
                .map(|r| format!("**{r}**"))
                .collect::<Vec<_>>()
                .join(" · ");
            let em = Embed::new()
                .title(format!("\u{1F3B2} Dice Roll — {count}d{sides}"))
                .field_inline("Total", format!("**{total}**"))
                .field_inline("Rolls", joined)
                .color(COLOR_ACCENT);
            ctx.reply_with(Reply::embed(em)).await
        }
        _ => {
            let em = Embed::new()
                .title("\u{2753} Bad Dice Spec")
                .description(
                    "Usage: `/roll NdM` where N ∈ 1..=100 and M ∈ 2..=1000.\nExample: `/roll 3d6`.",
                )
                .color(COLOR_WARN);
            ctx.reply_with(Reply::embed(em)).await
        }
    }
}

async fn eight_ball(ctx: Ctx) -> Result<()> {
    const ANSWERS: &[&str] = &[
        "Yes.", "No.", "Probably.", "Absolutely not.", "Ask again later.",
        "Signs point to yes.", "Outlook not so good.", "Without a doubt.",
        "Very doubtful.", "It is certain.", "Cannot predict now.", "Most likely.",
    ];
    let question = ctx.args().trim().to_owned();
    if question.is_empty() {
        let em = Embed::new()
            .title("\u{1F3B1} Magic 8-Ball")
            .description("Ask a question: `/8ball will it rain tomorrow?`")
            .color(COLOR_WARN);
        return ctx.reply_with(Reply::embed(em)).await;
    }
    let pick = {
        let mut rng = rand::thread_rng();
        ANSWERS.choose(&mut rng).copied().unwrap_or("...")
    };
    let em = Embed::new()
        .title("\u{1F3B1} Magic 8-Ball")
        .field("Question", question)
        .field("Answer", format!("**{pick}**"))
        .color(COLOR_ACCENT);
    ctx.reply_with(Reply::embed(em)).await
}

async fn cat(ctx: Ctx) -> Result<()> {
    let url = format!(
        "https://cataas.com/cat?ts={}",
        chrono::Utc::now().timestamp_millis()
    );
    let em = Embed::new()
        .title("\u{1F431} Random Cat")
        .description("A random fluffy friend for you.")
        .image(url)
        .color(COLOR_ACCENT);
    ctx.reply_with(Reply::embed(em)).await
}

// ---------- Fun extras ------------------------------------------------------

async fn joke(ctx: Ctx) -> Result<()> {
    // Short, SFW, cross-culture safe.
    const JOKES: &[&str] = &[
        "Why do programmers prefer dark mode? Because light attracts bugs.",
        "There are 10 kinds of people — those who understand binary and those who don't.",
        "I told my computer I needed a break and it said \"No problem, I'll go to sleep.\"",
        "Debugging: being the detective in a crime movie where you are also the murderer.",
        "A SQL query walks into a bar, sees two tables and asks: \"Can I join you?\"",
        "Why don't scientists trust atoms? Because they make up everything.",
        "How many programmers does it take to change a lightbulb? None, that's a hardware problem.",
        "I'd tell you a UDP joke, but you might not get it.",
        "There is no place like 127.0.0.1.",
        "Real programmers count from 0.",
    ];
    let pick = {
        let mut rng = rand::thread_rng();
        JOKES.choose(&mut rng).copied().unwrap_or("No joke today \u{1F61E}")
    };
    let em = Embed::new()
        .title("\u{1F3AD} Joke")
        .description(pick)
        .color(COLOR_ACCENT);
    ctx.reply_with(Reply::embed(em)).await
}

async fn choose(ctx: Ctx) -> Result<()> {
    // Comma-separated first, whitespace as fallback.
    let raw = ctx.args().trim();
    if raw.is_empty() {
        let em = Embed::new()
            .title("\u{1F3B2} Choose")
            .description("Give me a few options, comma- or space-separated.\nExample: `/choose pizza, sushi, burger`")
            .color(COLOR_WARN);
        return ctx.reply_with(Reply::embed(em)).await;
    }
    let mut options: Vec<&str> = raw
        .split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .collect();
    if options.len() < 2 {
        options = raw.split_whitespace().collect();
    }
    if options.len() < 2 {
        let em = Embed::new()
            .title("\u{1F3B2} Choose")
            .description("Need at least two options to choose from.")
            .color(COLOR_WARN);
        return ctx.reply_with(Reply::embed(em)).await;
    }
    let pick = {
        let mut rng = rand::thread_rng();
        options.choose(&mut rng).copied().unwrap_or("?")
    };
    let em = Embed::new()
        .title("\u{1F3B2} Choose")
        .field("Options", options.join(" · "))
        .field("Pick", format!("**{pick}**"))
        .color(COLOR_ACCENT);
    ctx.reply_with(Reply::embed(em)).await
}

async fn reverse(ctx: Ctx) -> Result<()> {
    let text = ctx.args();
    if text.trim().is_empty() {
        let em = Embed::new()
            .title("\u{1F501} Reverse")
            .description("Give me some text: `/reverse hello`")
            .color(COLOR_WARN);
        return ctx.reply_with(Reply::embed(em)).await;
    }
    let reversed: String = text.chars().rev().collect();
    let em = Embed::new()
        .title("\u{1F501} Reverse")
        .field("Input", text.to_string())
        .field("Output", reversed)
        .color(COLOR_ACCENT);
    ctx.reply_with(Reply::embed(em)).await
}

async fn emoji(ctx: Ctx) -> Result<()> {
    const POOL: &[&str] = &[
        "\u{1F436}", "\u{1F431}", "\u{1F98A}", "\u{1F43C}", "\u{1F428}", "\u{1F981}",
        "\u{1F42F}", "\u{1F984}", "\u{1F98B}", "\u{1F33B}", "\u{1F337}", "\u{1F352}",
        "\u{1F34A}", "\u{1F355}", "\u{1F366}", "\u{1F31F}", "\u{2728}", "\u{1F308}",
        "\u{1F31E}", "\u{1F319}", "\u{1F680}", "\u{1F3AE}", "\u{1F3B2}", "\u{1F3A8}",
    ];
    let pick = {
        let mut rng = rand::thread_rng();
        POOL.choose(&mut rng).copied().unwrap_or("\u{2728}")
    };
    let em = Embed::new()
        .title(format!("{pick} Random Emoji"))
        .description("A random cute emoji for you.")
        .color(COLOR_ACCENT);
    ctx.reply_with(Reply::embed(em)).await
}

/// `/qr TEXT` — renders a QR code pointing at TEXT using goqr.me.
async fn qr(ctx: Ctx) -> Result<()> {
    let text = ctx.args().trim();
    if text.is_empty() {
        let em = Embed::new()
            .title("\u{1F4F1} QR Code")
            .description("Give me something to encode: `/qr https://example.com`")
            .color(COLOR_WARN);
        return ctx.reply_with(Reply::embed(em)).await;
    }
    if text.len() > 800 {
        let em = Embed::new()
            .title("\u{1F4F1} QR Code")
            .description("Text is too long for a QR code (max 800 chars).")
            .color(COLOR_WARN);
        return ctx.reply_with(Reply::embed(em)).await;
    }
    let url = format!(
        "https://api.qrserver.com/v1/create-qr-code/?size=400x400&data={}",
        urlencode(text)
    );
    let em = Embed::new()
        .title("\u{1F4F1} QR Code")
        .description(format!("Encoded: `{}`", text.chars().take(80).collect::<String>()))
        .image(url)
        .color(COLOR_ACCENT);
    ctx.reply_with(Reply::embed(em)).await
}

/// `/shorten URL` — asks is.gd for a short alias.
async fn shorten(ctx: Ctx) -> Result<()> {
    let raw = ctx.args().trim();
    if raw.is_empty() {
        let em = Embed::new()
            .title("\u{1F517} Shorten")
            .description("Usage: `/shorten https://example.com/long/path`")
            .color(COLOR_WARN);
        return ctx.reply_with(Reply::embed(em)).await;
    }
    if !(raw.starts_with("http://") || raw.starts_with("https://")) {
        let em = Embed::new()
            .title("\u{1F517} Shorten")
            .description("URL must start with `http://` or `https://`.")
            .color(COLOR_WARN);
        return ctx.reply_with(Reply::embed(em)).await;
    }
    let api = format!(
        "https://is.gd/create.php?format=simple&url={}",
        urlencode(raw)
    );
    match reqwest::get(&api).await {
        Ok(r) if r.status().is_success() => match r.text().await {
            Ok(body) => {
                let short = body.trim();
                if short.starts_with("http") {
                    let em = Embed::new()
                        .title("\u{1F517} Shortened")
                        .field("Original", format!("`{raw}`"))
                        .field("Short", format!("**{short}**"))
                        .color(COLOR_ACCENT);
                    ctx.reply_with(Reply::embed(em)).await
                } else {
                    let em = Embed::new()
                        .title("\u{274C} Shorten Failed")
                        .description(format!("Provider said: `{short}`"))
                        .color(COLOR_WARN);
                    ctx.reply_with(Reply::embed(em)).await
                }
            }
            Err(_) => {
                let em = Embed::new()
                    .title("\u{274C} Shorten Failed")
                    .description("Could not read provider response.")
                    .color(COLOR_WARN);
                ctx.reply_with(Reply::embed(em)).await
            }
        },
        _ => {
            let em = Embed::new()
                .title("\u{274C} Shorten Failed")
                .description("Provider is unreachable right now.")
                .color(COLOR_WARN);
            ctx.reply_with(Reply::embed(em)).await
        }
    }
}

/// Process start time, captured on first call. Used by `/stats`.
static BOT_START: std::sync::OnceLock<std::time::Instant> = std::sync::OnceLock::new();

/// `/stats` — uptime + build info.
async fn stats(ctx: Ctx) -> Result<()> {
    let start = *BOT_START.get_or_init(std::time::Instant::now);
    let up = start.elapsed();
    let total_secs = up.as_secs();
    let days = total_secs / 86_400;
    let hours = (total_secs % 86_400) / 3_600;
    let minutes = (total_secs % 3_600) / 60;
    let seconds = total_secs % 60;
    let uptime = if days > 0 {
        format!("{days}d {hours}h {minutes}m {seconds}s")
    } else if hours > 0 {
        format!("{hours}h {minutes}m {seconds}s")
    } else if minutes > 0 {
        format!("{minutes}m {seconds}s")
    } else {
        format!("{seconds}s")
    };
    let em = Embed::new()
        .title("\u{1F4CA} Stats")
        .field_inline("Uptime", uptime)
        .field_inline("Version", format!("`{}`", env!("CARGO_PKG_VERSION")))
        .field_inline("Platform", capitalize(&ctx.platform().to_string()))
        .footer("FoukoBot · bot.fouko.xyz")
        .color(COLOR_ACCENT);
    ctx.reply_with(Reply::embed(em)).await
}

// ---------- Menu (buttons demo) ---------------------------------------------

/// `/menu` — callback ids look like `menu:<invoker_id>:<action>`.
/// Only the user who ran `/menu` is allowed to tap its buttons.
async fn menu(ctx: Ctx) -> Result<()> {
    if let Some(data) = ctx.callback_data() {
        if let Some(rest) = data.strip_prefix("menu:") {
            let mut parts = rest.splitn(2, ':');
            let invoker = parts.next().unwrap_or("");
            let action = parts.next().unwrap_or("").to_ascii_lowercase();
            if invoker != ctx.user_id() {
                return ctx
                    .reply("This menu isn't for you — run /menu to get your own.")
                    .await;
            }
            return run_menu_action(ctx, &action).await;
        }
    }
    let invoker = ctx.user_id().to_owned();
    let kb = menu_keyboard(&invoker);
    let em = Embed::new()
        .title("\u{1F4CB} Menu")
        .description("Pick something to try:")
        .color(COLOR_ACCENT);
    ctx.reply_with(Reply::embed(em).keyboard(kb)).await
}

fn menu_keyboard(invoker: &str) -> Keyboard {
    Keyboard::new()
        .row([
            Button::callback("\u{1F3B0} Coin", format!("menu:{invoker}:coin")),
            Button::callback("\u{1F3B2} Roll", format!("menu:{invoker}:roll")),
        ])
        .row([
            Button::callback("\u{1F431} Cat", format!("menu:{invoker}:cat")),
            Button::callback("\u{1F550} Time", format!("menu:{invoker}:time")),
        ])
        .row([Button::url("\u{1F310} bot.fouko.xyz", "https://bot.fouko.xyz")])
}

async fn run_menu_action(ctx: Ctx, action: &str) -> Result<()> {
    let (title, body) = match action {
        "coin" => {
            let heads = {
                let mut rng = rand::thread_rng();
                rng.gen_bool(0.5)
            };
            let side = if heads { "Heads \u{1FA99}" } else { "Tails \u{1F4B0}" };
            ("\u{1F3B0} Coin Flip".to_owned(), format!("Result: **{side}**"))
        }
        "roll" => {
            let n = {
                let mut rng = rand::thread_rng();
                rng.gen_range(1..=6)
            };
            ("\u{1F3B2} Dice Roll".to_owned(), format!("d6 → **{n}**"))
        }
        "cat" => {
            let url = format!(
                "https://cataas.com/cat?ts={}",
                chrono::Utc::now().timestamp_millis()
            );
            ("\u{1F431} Random Cat".to_owned(), url)
        }
        "time" => {
            let now = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
            ("\u{1F550} Time (UTC)".to_owned(), now)
        }
        _ => ("Menu".to_owned(), "Unknown action".to_owned()),
    };
    let invoker = ctx.user_id().to_owned();
    let kb = menu_keyboard(&invoker);
    let em = Embed::new()
        .title(title)
        .description(body)
        .footer("Pick another action below")
        .color(COLOR_ACCENT);
    ctx.edit_reply(Reply::embed(em).keyboard(kb)).await
}

// ---------- Weather ---------------------------------------------------------

async fn weather(ctx: Ctx, svc: Services) -> Result<()> {
    let lang = svc
        .accounts
        .lang_for(ctx.platform(), ctx.user_id())
        .await
        .unwrap_or_else(|_| "en".into());
    let city = ctx.args().trim();
    if city.is_empty() {
        let (title, desc) = match lang.as_str() {
            "ru" => (
                "\u{1F324}\u{FE0F} Погода",
                "Напиши название города: `/weather Berlin`",
            ),
            _ => (
                "\u{1F324}\u{FE0F} Weather",
                "Give me a city name: `/weather Berlin`",
            ),
        };
        let em = Embed::new().title(title).description(desc).color(COLOR_WARN);
        return ctx.reply_with(Reply::embed(em)).await;
    }
    let Some(key) = svc.weather_key.as_ref() else {
        let (title, desc) = match lang.as_str() {
            "ru" => (
                "\u{1F324}\u{FE0F} Погода",
                "Погода не настроена на этом боте (нужен OPEN_WEATHER_KEY в .env).",
            ),
            _ => (
                "\u{1F324}\u{FE0F} Weather",
                "Weather is not configured on this bot (set OPEN_WEATHER_KEY in .env).",
            ),
        };
        let em = Embed::new().title(title).description(desc).color(COLOR_WARN);
        return ctx.reply_with(Reply::embed(em)).await;
    };
    match fetch_weather_embed(city, key, &lang).await {
        Ok(em) => ctx.reply_with(Reply::embed(em)).await,
        Err(e) => {
            tracing::warn!(error = %e, "weather fetch failed");
            let (title, prefix) = match lang.as_str() {
                "ru" => ("\u{1F324}\u{FE0F} Не получилось", "Погода:"),
                _ => ("\u{1F324}\u{FE0F} Weather Failed", "Weather:"),
            };
            let em = Embed::new()
                .title(title)
                .description(format!("{prefix} {e}"))
                .color(COLOR_WARN);
            ctx.reply_with(Reply::embed(em)).await
        }
    }
}

/// Weather lookup errors. API key is intentionally stripped from messages
/// so the bot never leaks it back into chat.
#[derive(Debug)]
enum WeatherError {
    Unauthorized,
    CityNotFound,
    Http(u16),
    Network,
    BadJson,
}

impl std::fmt::Display for WeatherError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unauthorized => f.write_str(
                "the weather provider rejected the API key (401). \
                 ask the bot owner to set a valid OPEN_WEATHER_KEY.",
            ),
            Self::CityNotFound => f.write_str("city not found, try a different spelling"),
            Self::Http(code) => write!(f, "weather provider returned HTTP {code}"),
            Self::Network => f.write_str("network error reaching the weather provider"),
            Self::BadJson => f.write_str("weather provider sent an unexpected response"),
        }
    }
}

impl std::error::Error for WeatherError {}

/// Resolve `city` and return a ready-to-send [`Embed`].
async fn fetch_weather_embed(
    city: &str,
    key: &str,
    lang: &str,
) -> std::result::Result<Embed, WeatherError> {
    // Geocoder first: gives canonical English name ("Moscow, RU") even
    // when the user typed in another language. Falls back to `?q=` if
    // the geocoder has no entry.
    if let Some((lat, lon, nice_name)) = geocode_city(city, key).await? {
        return fetch_weather_by_coords_embed(lat, lon, &nice_name, key, lang).await;
    }
    fetch_weather_by_q_embed(city, key, lang).await
}

async fn fetch_weather_by_q_embed(
    city: &str,
    key: &str,
    lang: &str,
) -> std::result::Result<Embed, WeatherError> {
    let url = format!(
        "https://api.openweathermap.org/data/2.5/weather?q={}&appid={}&units=metric&lang={}",
        urlencode(city),
        key,
        urlencode(lang)
    );
    let resp = reqwest::get(&url).await.map_err(|_| WeatherError::Network)?;
    match resp.status().as_u16() {
        200 => format_weather_embed(resp, lang, None).await,
        401 => Err(WeatherError::Unauthorized),
        404 => Err(WeatherError::CityNotFound),
        code => Err(WeatherError::Http(code)),
    }
}

async fn fetch_weather_by_coords_embed(
    lat: f64,
    lon: f64,
    nice_name: &str,
    key: &str,
    lang: &str,
) -> std::result::Result<Embed, WeatherError> {
    let url = format!(
        "https://api.openweathermap.org/data/2.5/weather?lat={lat}&lon={lon}&appid={}&units=metric&lang={}",
        key,
        urlencode(lang)
    );
    let resp = reqwest::get(&url).await.map_err(|_| WeatherError::Network)?;
    match resp.status().as_u16() {
        200 => format_weather_embed(resp, lang, Some(nice_name.to_owned())).await,
        401 => Err(WeatherError::Unauthorized),
        404 => Err(WeatherError::CityNotFound),
        code => Err(WeatherError::Http(code)),
    }
}

async fn format_weather_embed(
    resp: reqwest::Response,
    lang: &str,
    override_name: Option<String>,
) -> std::result::Result<Embed, WeatherError> {
    #[derive(serde::Deserialize)]
    struct Resp {
        name: String,
        weather: Vec<WeatherItem>,
        main: MainInfo,
        wind: Wind,
        sys: Option<Sys>,
    }
    #[derive(serde::Deserialize)]
    struct WeatherItem {
        description: String,
        icon: Option<String>,
    }
    #[derive(serde::Deserialize)]
    struct MainInfo {
        temp: f64,
        feels_like: f64,
        humidity: f64,
    }
    #[derive(serde::Deserialize)]
    struct Wind {
        speed: f64,
    }
    #[derive(serde::Deserialize)]
    struct Sys {
        country: Option<String>,
    }

    let parsed: Resp = resp.json().await.map_err(|_| WeatherError::BadJson)?;
    let name = match override_name {
        Some(n) => n,
        None => match parsed.sys.as_ref().and_then(|s| s.country.as_deref()) {
            Some(cc) if !cc.is_empty() => format!("{}, {cc}", parsed.name),
            _ => parsed.name.clone(),
        },
    };
    let first = parsed.weather.first();
    let cond = first.map(|w| w.description.as_str()).unwrap_or("?");
    let icon_url = first
        .and_then(|w| w.icon.as_deref())
        .map(|i| format!("https://openweathermap.org/img/wn/{i}@2x.png"));

    let (title_prefix, temp_label, feels_label, humid_label, wind_label, wind_unit) =
        match lang {
            "ru" => (
                "\u{1F324}\u{FE0F}",
                "Температура",
                "Ощущается",
                "Влажность",
                "Ветер",
                "м/с",
            ),
            _ => (
                "\u{1F324}\u{FE0F}",
                "Temperature",
                "Feels Like",
                "Humidity",
                "Wind",
                "m/s",
            ),
        };

    let mut em = Embed::new()
        .title(format!("{title_prefix} {name}"))
        .description(cond.to_string())
        .field_inline(temp_label, format!("{:.1}\u{00B0}C", parsed.main.temp))
        .field_inline(feels_label, format!("{:.1}\u{00B0}C", parsed.main.feels_like))
        .field_inline(humid_label, format!("{}%", parsed.main.humidity))
        .field_inline(
            wind_label,
            format!("{:.1} {wind_unit}", parsed.wind.speed),
        )
        .color(COLOR_ACCENT);
    if let Some(icon) = icon_url {
        em = em.thumbnail(icon);
    }
    Ok(em)
}

/// Ask OpenWeather's geocoding API for candidates matching `city`.
/// Returns `(lat, lon, canonical_name)` of the top match, or `None`
/// when nothing close was found.
async fn geocode_city(
    city: &str,
    key: &str,
) -> std::result::Result<Option<(f64, f64, String)>, WeatherError> {
    #[derive(serde::Deserialize)]
    struct GeoHit {
        lat: f64,
        lon: f64,
        name: String,
        country: Option<String>,
    }
    let url = format!(
        "https://api.openweathermap.org/geo/1.0/direct?q={}&limit=1&appid={}",
        urlencode(city),
        key
    );
    let resp = reqwest::get(&url).await.map_err(|_| WeatherError::Network)?;
    match resp.status().as_u16() {
        200 => {
            let hits: Vec<GeoHit> = resp.json().await.map_err(|_| WeatherError::BadJson)?;
            let Some(hit) = hits.into_iter().next() else {
                return Ok(None);
            };
            let pretty = match hit.country {
                Some(c) if !c.is_empty() => format!("{}, {c}", hit.name),
                _ => hit.name,
            };
            Ok(Some((hit.lat, hit.lon, pretty)))
        }
        401 => Err(WeatherError::Unauthorized),
        _ => Ok(None),
    }
}

// ---------- Helpers ---------------------------------------------------------

fn parse_dice(spec: &str) -> Option<(u32, u32)> {
    let s = spec.trim().to_ascii_lowercase();
    let (n_str, m_str) = s.split_once('d')?;
    let count: u32 = if n_str.is_empty() { 1 } else { n_str.parse().ok()? };
    let sides: u32 = m_str.parse().ok()?;
    Some((count, sides))
}


