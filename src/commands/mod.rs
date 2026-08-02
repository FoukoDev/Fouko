//! FoukoBot command set.

mod account;
mod economy;
mod fun;
pub(crate) mod helpers;
mod info;
mod tools;

use account::settings_cmd;
use economy::{
    achievements_cmd, award_xp, buy, daily, gamble, give, leaderboard, profile, rank, shop,
};
use fun::{cat, choose, coin, eight_ball, emoji, joke, menu, poll, reverse, roll, rps};
use helpers::i18n_map;
use info::{avatar, help, info, ping, server};
use tools::{calc, qr, remind, shorten, time, weather};

pub use tools::restore_reminders;

use foukoapi::{
    Accounts, AnyStorage, Bot, Ctx, Economy, Embed, I18n, Notifier, PlatformKind, Reply, Result,
    TextMatch,
};

/// Shared colour palette for bot-side embeds.
pub(crate) const COLOR_ACCENT: u32 = 0x7A5BE8;
pub(crate) const COLOR_OK: u32 = 0x43B581;
pub(crate) const COLOR_WARN: u32 = 0xF59F00;

/// Process start time, captured on first call. Feeds `/info`'s uptime.
pub(crate) static BOT_START: std::sync::OnceLock<std::time::Instant> = std::sync::OnceLock::new();

/// Services shared by every command.
#[derive(Clone)]
pub struct Services {
    pub storage: AnyStorage,
    pub accounts: Accounts,
    pub econ: Economy,
    pub i18n: I18n,
    pub notifier: Notifier,
    /// `None` when FOUKO_SECRET isn't set, disabling the AI feature.
    pub ai: Option<crate::ai::AiStore>,
}

impl Services {
    /// The caller's language, resolved through their linked accounts.
    /// Defaults to English.
    pub(crate) async fn lang(&self, ctx: &Ctx) -> String {
        self.accounts
            .lang_for(ctx.platform(), ctx.user_id())
            .await
            .unwrap_or_else(|_| "en".into())
    }

    /// Language of an arbitrary user, for messages delivered to someone
    /// other than the caller (reminders, invites). Defaults to English.
    pub(crate) async fn lang_of(&self, platform: PlatformKind, user_id: &str) -> String {
        self.accounts
            .lang_for(platform, user_id)
            .await
            .unwrap_or_else(|_| "en".into())
    }

    /// Translate `key` into the caller's language.
    pub(crate) async fn tr(&self, ctx: &Ctx, key: &str) -> String {
        let lang = self.lang(ctx).await;
        self.i18n.t(&lang, key)
    }

    /// Translate `key` and fill its `{}` placeholders.
    pub(crate) async fn trf(&self, ctx: &Ctx, key: &str, args: &[&str]) -> String {
        let lang = self.lang(ctx).await;
        self.i18n.tf(&lang, key, args)
    }

    /// Rate-limit an action that hits an external service. If the caller
    /// is still on cooldown, sends a short "slow down" reply and returns
    /// `true` so the handler can bail early; otherwise stamps the cooldown
    /// and returns `false`.
    pub(crate) async fn rate_limited(
        &self,
        ctx: &Ctx,
        action: &str,
        window_secs: i64,
    ) -> Result<bool> {
        let wait = self
            .econ
            .cooldown_remaining(ctx.platform(), ctx.user_id(), action, window_secs)
            .await;
        if wait > 0 {
            let lang = self.lang(ctx).await;
            let em = Embed::new()
                .title(self.i18n.t(&lang, "slow_down_title"))
                .description(self.i18n.tf(
                    &lang,
                    "rate_limited",
                    &[&helpers::human_duration(wait, &lang)],
                ))
                .color(COLOR_WARN);
            // A throttle notice is noise once it's served its purpose, so
            // let it clean itself up after a few seconds.
            ctx.reply_temporary(Reply::embed(em), 5).await?;
            return Ok(true);
        }
        self.econ
            .touch_cooldown(ctx.platform(), ctx.user_id(), action)
            .await?;
        Ok(false)
    }
}

/// Wire every command into a [`Bot`].
pub fn register(bot: Bot, svc: Services) -> Bot {
    let s_help = svc.clone();
    let s_profile = svc.clone();
    let s_weather = svc.clone();
    let s_settings = svc.clone();
    let s_joke = svc.clone();
    let s_menu = svc.clone();
    let s_rps = svc.clone();
    let s_help_text = svc.clone();
    let s_about = svc.clone();
    let s_server = svc.clone();
    let s_avatar = svc.clone();
    let s_8ball = svc.clone();
    let s_cat = svc.clone();
    let s_qr = svc.clone();
    let s_shorten = svc.clone();
    let s_daily = svc.clone();
    let s_lb = svc.clone();
    let s_rank = svc.clone();
    let s_ach = svc.clone();
    let s_shop = svc.clone();
    let s_buy = svc.clone();
    let s_give = svc.clone();
    let s_gamble = svc.clone();
    let s_poll = svc.clone();
    let s_remind = svc.clone();
    let s_rl = svc.clone();
    let s_ai = svc.clone();
    let ai_passive_svc = svc.clone();
    let xp_svc = svc.clone();
    let accounts_for_bot = svc.accounts.clone();
    bot.with_accounts(accounts_for_bot)
        .on_message(move |ctx| {
            let s = xp_svc.clone();
            async move { award_xp(ctx, s).await }
        })
        .on_message(move |ctx| {
            let s = ai_passive_svc.clone();
            async move { crate::ai::command::ai_passive(ctx, s).await }
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
            {
                let s = svc.clone();
                move |ctx| ping(ctx, s.clone())
            },
        )
        .command_described_i18n(
            "/info",
            i18n_map(&[
                ("en", "about the bot: stack, features, version, uptime"),
                ("ru", "о боте: стек, возможности, версия, аптайм"),
            ]),
            move |ctx| info(ctx, s_about.clone()),
        )
        .command_described_i18n(
            "/server",
            i18n_map(&[
                ("en", "info about this server or chat"),
                ("ru", "инфо об этом сервере или чате"),
            ]),
            move |ctx| server(ctx, s_server.clone()),
        )
        .command_described_i18n(
            "/avatar",
            i18n_map(&[
                ("en", "show your avatar (Discord only)"),
                ("ru", "показать твой аватар (только Discord)"),
            ]),
            move |ctx| avatar(ctx, s_avatar.clone()),
        )
        .only_on("/avatar", &[PlatformKind::Discord])
        .command_described_i18n(
            "/time",
            i18n_map(&[
                (
                    "en",
                    "current time (optional UTC offset: /time +10 or /time -5:30)",
                ),
                (
                    "ru",
                    "текущее время (можно с UTC-сдвигом: /time +10 или /time -5:30)",
                ),
            ]),
            {
                let s = svc.clone();
                move |ctx| time(ctx, s.clone())
            },
        )
        .command_described_i18n(
            "/coin",
            i18n_map(&[("en", "flip a coin"), ("ru", "подбросить монетку")]),
            {
                let s = svc.clone();
                move |ctx| coin(ctx, s.clone())
            },
        )
        .command_described_i18n(
            "/roll",
            i18n_map(&[
                ("en", "roll dice, e.g. /roll 3d6"),
                ("ru", "бросить кубики, например /roll 3d6"),
            ]),
            {
                let s = svc.clone();
                move |ctx| roll(ctx, s.clone())
            },
        )
        .command_described_i18n(
            "/8ball",
            i18n_map(&[
                ("en", "ask a question, get an answer"),
                ("ru", "задай вопрос, получи ответ"),
            ]),
            move |ctx| eight_ball(ctx, s_8ball.clone()),
        )
        .command_described_i18n(
            "/cat",
            i18n_map(&[
                ("en", "random cat picture (Telegram / Discord only)"),
                ("ru", "случайный котик (только Telegram / Discord)"),
            ]),
            move |ctx| cat(ctx, s_cat.clone()),
        )
        .only_on("/cat", &[PlatformKind::Telegram, PlatformKind::Discord])
        .command_described_i18n(
            "/menu",
            i18n_map(&[
                ("en", "interactive menu with buttons"),
                ("ru", "меню с кнопками"),
            ]),
            move |ctx| menu(ctx, s_menu.clone()),
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
                ("en", "a joke - pick a theme with buttons"),
                ("ru", "шутка - тему можно выбрать кнопками"),
            ]),
            move |ctx| joke(ctx, s_joke.clone()),
        )
        .command_described_i18n(
            "/choose",
            i18n_map(&[
                ("en", "pick one at random: /choose a, b, c"),
                ("ru", "выбрать случайное: /choose а, б, в"),
            ]),
            {
                let s = svc.clone();
                move |ctx| choose(ctx, s.clone())
            },
        )
        .command_described_i18n(
            "/reverse",
            i18n_map(&[
                ("en", "reverse your text: /reverse hello"),
                ("ru", "перевернуть текст: /reverse привет"),
            ]),
            {
                let s = svc.clone();
                move |ctx| reverse(ctx, s.clone())
            },
        )
        .command_described_i18n(
            "/emoji",
            i18n_map(&[
                ("en", "a random cute emoji"),
                ("ru", "случайный милый эмодзи"),
            ]),
            {
                let s = svc.clone();
                move |ctx| emoji(ctx, s.clone())
            },
        )
        .command_described_i18n(
            "/qr",
            i18n_map(&[
                ("en", "generate a QR code, e.g. /qr hello"),
                ("ru", "сгенерировать QR-код, например /qr привет"),
            ]),
            move |ctx| qr(ctx, s_qr.clone()),
        )
        .command_described_i18n(
            "/shorten",
            i18n_map(&[
                ("en", "shorten a URL: /shorten https://example.com/long"),
                ("ru", "сократить ссылку: /shorten https://example.com/long"),
            ]),
            move |ctx| shorten(ctx, s_shorten.clone()),
        )
        .command_described_i18n(
            "/daily",
            i18n_map(&[
                ("en", "claim your daily reward and keep a streak"),
                ("ru", "ежедневная награда и стрик"),
            ]),
            move |ctx| daily(ctx, s_daily.clone()),
        )
        .command_described_i18n(
            "/leaderboard",
            i18n_map(&[
                ("en", "top players by XP (or /leaderboard coins)"),
                ("ru", "топ по опыту (или /leaderboard coins)"),
            ]),
            move |ctx| leaderboard(ctx, s_lb.clone()),
        )
        .command_described_i18n(
            "/rank",
            i18n_map(&[
                ("en", "your (or @user's) leaderboard position"),
                ("ru", "твоё место в рейтинге (или @юзера)"),
            ]),
            move |ctx| rank(ctx, s_rank.clone()),
        )
        .command_described_i18n(
            "/achievements",
            i18n_map(&[
                ("en", "all badges - yours or @user's"),
                ("ru", "все достижения - твои или @юзера"),
            ]),
            move |ctx| achievements_cmd(ctx, s_ach.clone()),
        )
        .command_described_i18n(
            "/shop",
            i18n_map(&[
                ("en", "browse titles and profile colours"),
                ("ru", "магазин титулов и цветов профиля"),
            ]),
            move |ctx| shop(ctx, s_shop.clone()),
        )
        .command_described_i18n(
            "/buy",
            i18n_map(&[
                ("en", "buy a shop item: /buy title_legend"),
                ("ru", "купить товар: /buy title_legend"),
            ]),
            move |ctx| buy(ctx, s_buy.clone()),
        )
        .command_described_i18n(
            "/give",
            i18n_map(&[
                ("en", "send coins to someone: /give <id> <amount>"),
                ("ru", "отправить монеты: /give <id> <сумма>"),
            ]),
            move |ctx| give(ctx, s_give.clone()),
        )
        .command_described_i18n(
            "/gamble",
            i18n_map(&[
                ("en", "bet coins on a coin flip: /gamble 20"),
                ("ru", "поставить монеты: /gamble 20"),
            ]),
            move |ctx| gamble(ctx, s_gamble.clone()),
        )
        .command_described_i18n(
            "/rps",
            i18n_map(&[
                ("en", "rock paper scissors"),
                ("ru", "камень-ножницы-бумага"),
            ]),
            move |ctx| rps(ctx, s_rps.clone()),
        )
        .command_described_i18n(
            "/poll",
            i18n_map(&[
                ("en", "start a poll: /poll Q? | A | B"),
                ("ru", "опрос: /poll Вопрос? | А | Б"),
            ]),
            move |ctx| poll(ctx, s_poll.clone()),
        )
        .command_described_i18n(
            "/remind",
            i18n_map(&[
                ("en", "set a reminder: /remind 10m text"),
                ("ru", "напоминание: /remind 10m текст"),
            ]),
            move |ctx| remind(ctx, s_remind.clone()),
        )
        .command_described_i18n(
            "/calc",
            i18n_map(&[
                ("en", "evaluate an expression: /calc 2*(3+4)"),
                ("ru", "посчитать выражение: /calc 2*(3+4)"),
            ]),
            {
                let s = svc.clone();
                move |ctx| calc(ctx, s.clone())
            },
        )
        .command_described_i18n(
            "/ai",
            i18n_map(&[
                (
                    "en",
                    "your private AI - hosts, chats, prompts; sub-commands: /ai help",
                ),
                (
                    "ru",
                    "твой личный ИИ - хосты, чаты, промпты; подкоманды: /ai help",
                ),
            ]),
            move |ctx| crate::ai::command::ai(ctx, s_ai.clone()),
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
        .text_command("ping", TextMatch::Exact, {
            let s = svc.clone();
            move |ctx| ping(ctx, s.clone())
        })
        .with_default_link_command()
        .with_default_lang_command(crate::strings::SUPPORTED.iter().copied())
        // Group commands for a tidy /help. Headings are plain labels; the
        // built-in /help lays them out as sections.
        .category("/start", "\u{2139}\u{FE0F} General")
        .category("/help", "\u{2139}\u{FE0F} General")
        .category("/info", "\u{2139}\u{FE0F} General")
        .category("/server", "\u{2139}\u{FE0F} General")
        .category("/ping", "\u{2139}\u{FE0F} General")
        .category("/profile", "\u{1F4B0} Economy")
        .category("/daily", "\u{1F4B0} Economy")
        .category("/leaderboard", "\u{1F4B0} Economy")
        .category("/rank", "\u{1F4B0} Economy")
        .category("/achievements", "\u{1F4B0} Economy")
        .category("/shop", "\u{1F4B0} Economy")
        .category("/buy", "\u{1F4B0} Economy")
        .category("/give", "\u{1F4B0} Economy")
        .category("/gamble", "\u{1F4B0} Economy")
        .category("/coin", "\u{1F3B2} Fun")
        .category("/roll", "\u{1F3B2} Fun")
        .category("/8ball", "\u{1F3B2} Fun")
        .category("/rps", "\u{1F3B2} Fun")
        .category("/poll", "\u{1F3B2} Fun")
        .category("/cat", "\u{1F3B2} Fun")
        .category("/joke", "\u{1F3B2} Fun")
        .category("/choose", "\u{1F3B2} Fun")
        .category("/emoji", "\u{1F3B2} Fun")
        .category("/menu", "\u{1F3B2} Fun")
        .category("/avatar", "\u{1F3B2} Fun")
        .category("/time", "\u{1F6E0}\u{FE0F} Tools")
        .category("/weather", "\u{1F6E0}\u{FE0F} Tools")
        .category("/qr", "\u{1F6E0}\u{FE0F} Tools")
        .category("/shorten", "\u{1F6E0}\u{FE0F} Tools")
        .category("/calc", "\u{1F6E0}\u{FE0F} Tools")
        .category("/remind", "\u{1F6E0}\u{FE0F} Tools")
        .category("/reverse", "\u{1F6E0}\u{FE0F} Tools")
        .category("/link", "\u{1F517} Account")
        .category("/lang", "\u{1F517} Account")
        .category("/settings", "\u{1F517} Account")
        .category("/ai", "\u{1F916} AI")
        // These take a person: Discord shows its native member picker.
        .user_option("/profile")
        .user_option("/rank")
        .user_option("/achievements")
        .user_option("/give")
        .user_option("/avatar")
        .with_default_help()
        .on_rate_limited(move |ctx| {
            let svc = s_rl.clone();
            async move { rate_limit_notice(ctx, svc).await }
        })
}

/// Shown when a user trips the global rate limit. To avoid answering every
/// message of a flood (which would be its own spam), we only send the
/// notice once every 10 seconds per user and stay silent in between.
async fn rate_limit_notice(ctx: Ctx, svc: Services) -> Result<()> {
    let quiet = svc
        .econ
        .cooldown_remaining(ctx.platform(), ctx.user_id(), "ratelimit_notice", 10)
        .await;
    if quiet > 0 {
        return Ok(());
    }
    svc.econ
        .touch_cooldown(ctx.platform(), ctx.user_id(), "ratelimit_notice")
        .await?;
    let em = Embed::new()
        .title(svc.tr(&ctx, "slow_down_title").await)
        .description(svc.tr(&ctx, "flood_notice").await)
        .color(COLOR_WARN);
    ctx.reply_temporary(Reply::embed(em), 8).await
}
