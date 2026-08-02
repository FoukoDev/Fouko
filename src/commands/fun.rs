//! Fun commands: coin, dice, 8-ball, rock-paper-scissors, polls, cat, joke,
//! choose, reverse, emoji, and the button menu.

use super::helpers::parse_dice;
use super::Services;
use super::{COLOR_ACCENT, COLOR_OK, COLOR_WARN};
use chrono::Utc;
use foukoapi::{util::progress_bar, Button, Ctx, Embed, Keyboard, Reply, Result};
use rand::{seq::SliceRandom, Rng};
use unicode_segmentation::UnicodeSegmentation;

pub(crate) async fn coin(ctx: Ctx, svc: Services) -> Result<()> {
    let heads = {
        let mut rng = rand::thread_rng();
        rng.gen_bool(0.5)
    };
    let (side_key, icon) = if heads {
        ("coin_heads", "\u{1FA99}")
    } else {
        ("coin_tails", "\u{1F4B0}")
    };
    let em = Embed::new()
        .title(svc.trf(&ctx, "coin_title", &[icon]).await)
        .description(format!("**{}**", svc.tr(&ctx, side_key).await))
        .color(COLOR_ACCENT);
    ctx.reply_with(Reply::embed(em)).await
}

pub(crate) async fn roll(ctx: Ctx, svc: Services) -> Result<()> {
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
                .title(
                    svc.trf(
                        &ctx,
                        "roll_title",
                        &[&count.to_string(), &sides.to_string()],
                    )
                    .await,
                )
                .field_inline(svc.tr(&ctx, "roll_total").await, format!("**{total}**"))
                .field_inline(svc.tr(&ctx, "roll_rolls").await, joined)
                .color(COLOR_ACCENT);
            ctx.reply_with(Reply::embed(em)).await
        }
        _ => {
            let em = Embed::new()
                .title(svc.tr(&ctx, "roll_bad_title").await)
                .description(svc.tr(&ctx, "roll_usage").await)
                .color(COLOR_WARN);
            ctx.reply_with(Reply::embed(em)).await
        }
    }
}

pub(crate) async fn eight_ball(ctx: Ctx, svc: Services) -> Result<()> {
    let question = ctx.args().trim().to_owned();
    if question.is_empty() {
        let em = Embed::new()
            .title(svc.tr(&ctx, "8ball_title").await)
            .description(svc.tr(&ctx, "8ball_prompt").await)
            .color(COLOR_WARN);
        return ctx.reply_with(Reply::embed(em)).await;
    }
    // Answers are stored as a `|`-separated list per language, so the
    // reply lands in the user's own tongue.
    let answers = svc.tr(&ctx, "8ball_answers").await;
    let options: Vec<&str> = answers.split('|').map(str::trim).collect();
    let pick = {
        let mut rng = rand::thread_rng();
        options.choose(&mut rng).copied().unwrap_or("...")
    };
    let em = Embed::new()
        .title(svc.tr(&ctx, "8ball_title").await)
        .field(svc.tr(&ctx, "8ball_question").await, question)
        .field(svc.tr(&ctx, "8ball_answer").await, format!("**{pick}**"))
        .color(COLOR_ACCENT);
    ctx.reply_with(Reply::embed(em)).await
}

pub(crate) async fn cat(ctx: Ctx, svc: Services) -> Result<()> {
    if svc.rate_limited(&ctx, "cat", 3).await? {
        return Ok(());
    }
    let url = format!(
        "https://cataas.com/cat?ts={}",
        chrono::Utc::now().timestamp_millis()
    );
    let em = Embed::new()
        .title(svc.tr(&ctx, "cat_title").await)
        .description(svc.tr(&ctx, "cat_body").await)
        .image(url)
        .color(COLOR_ACCENT);
    ctx.reply_with(Reply::embed(em)).await
}

// ---------- Fun extras ------------------------------------------------------

/// Joke categories JokeAPI knows. First entry doubles as the default.
const JOKE_CATEGORIES: &[&str] = &["Any", "Programming", "Pun", "Misc", "Spooky", "Christmas"];

/// Languages JokeAPI can serve. Anything else falls back to English.
const JOKE_LANGS: &[&str] = &["en", "de", "es", "fr", "cs", "pt"];

/// `/joke [category]` - a joke from JokeAPI in the user's language (when
/// the API supports it). Category buttons let you pick a theme; the
/// callback is `joke:<invoker>:<category>`.
pub(crate) async fn joke(ctx: Ctx, svc: Services) -> Result<()> {
    // Category button: only for whoever opened the card.
    if let Some(data) = ctx.callback_data() {
        if let Some(rest) = data.strip_prefix("joke:") {
            let mut parts = rest.splitn(2, ':');
            let invoker = parts.next().unwrap_or("");
            let category = parts.next().unwrap_or("Any");
            if invoker != ctx.user_id() {
                return ctx
                    .reply_temporary(svc.tr(&ctx, "not_your_button").await, 5)
                    .await;
            }
            return send_joke(&ctx, &svc, category, true).await;
        }
        return Ok(());
    }

    // Typed category, e.g. `/joke pun`. Unknown input just means "Any".
    let wanted = ctx.args().trim();
    let category = JOKE_CATEGORIES
        .iter()
        .find(|c| c.eq_ignore_ascii_case(wanted))
        .copied()
        .unwrap_or("Any");
    send_joke(&ctx, &svc, category, false).await
}

/// Fetch and show one joke, with theme buttons for the next round.
async fn send_joke(ctx: &Ctx, svc: &Services, category: &str, edit: bool) -> Result<()> {
    if svc.rate_limited(ctx, "joke", 3).await? {
        return Ok(());
    }

    // Russian speakers get jokes from a Russian service; everyone else
    // goes to JokeAPI in their language when it speaks it.
    let user_lang = svc.lang(ctx).await;
    let text = if user_lang == "ru" || user_lang == "uk" {
        fetch_joke_ru(category).await
    } else {
        let lang = JOKE_LANGS
            .iter()
            .find(|l| **l == user_lang)
            .copied()
            .unwrap_or("en");
        fetch_joke(category, lang).await
    };

    let text = match text {
        Some(j) => j,
        None => {
            let em = Embed::new()
                .description(svc.tr(ctx, "joke_unavailable").await)
                .color(COLOR_WARN);
            return ctx.reply_with(Reply::embed(em)).await;
        }
    };

    let invoker = ctx.user_id().to_owned();
    let mut kb = Keyboard::new();
    for chunk in JOKE_CATEGORIES.chunks(3) {
        let row: Vec<Button> = chunk
            .iter()
            .map(|c| {
                let mark = if c.eq_ignore_ascii_case(category) {
                    "\u{2B50} "
                } else {
                    ""
                };
                Button::callback(format!("{mark}{c}"), format!("joke:{invoker}:{c}"))
            })
            .collect();
        kb = kb.row(row);
    }

    let em = Embed::new()
        .title(svc.tr(ctx, "joke_title").await)
        .description(text)
        .color(COLOR_ACCENT);
    if edit {
        ctx.edit_reply(Reply::embed(em).keyboard(kb)).await
    } else {
        ctx.reply_with(Reply::embed(em).keyboard(kb)).await
    }
}

/// One safe-mode joke from JokeAPI, or `None` when the API is unreachable
/// or answers with an error.
async fn fetch_joke(category: &str, lang: &str) -> Option<String> {
    #[derive(serde::Deserialize)]
    struct JokeResp {
        #[serde(default)]
        error: bool,
        #[serde(default)]
        joke: Option<String>,
        #[serde(default)]
        setup: Option<String>,
        #[serde(default)]
        delivery: Option<String>,
    }

    let url = format!("https://v2.jokeapi.dev/joke/{category}?safe-mode&lang={lang}");
    let client = reqwest::Client::builder()
        .user_agent("FoukoBot/1.0")
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .ok()?;
    let resp = client.get(&url).send().await.ok()?;
    if !resp.status().is_success() {
        return None;
    }
    let parsed: JokeResp = resp.json().await.ok()?;
    if parsed.error {
        return None;
    }
    // Single-liner, or setup + punchline on two lines.
    if let Some(j) = parsed.joke {
        return Some(j);
    }
    match (parsed.setup, parsed.delivery) {
        (Some(s), Some(d)) => Some(format!("{s}\n\n||{d}||")),
        _ => None,
    }
}

/// One joke in Russian from rzhunemogu.ru. The site answers Windows-1251
/// JSON with unescaped newlines, so we decode the bytes ourselves and pull
/// the content field out by hand instead of serde. Categories map roughly:
/// programming jokes aren't a thing there, so anything non-default becomes
/// an aphorism/story pick.
///
/// TODO: the site is http-only and ancient; keep an eye out for a better
/// russian-language joke source.
async fn fetch_joke_ru(category: &str) -> Option<String> {
    // CType: 1 = joke, 4 = aphorism, 11 = one-liner.
    let ctype = match category {
        "Pun" => 11,
        "Misc" => 4,
        _ => 1,
    };
    let url = format!("http://rzhunemogu.ru/RandJSON.aspx?CType={ctype}");
    let client = reqwest::Client::builder()
        .user_agent("FoukoBot/1.0")
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .ok()?;
    let resp = client.get(&url).send().await.ok()?;
    if !resp.status().is_success() {
        return None;
    }
    let raw = resp.bytes().await.ok()?;
    let (decoded, _, _) = encoding_rs::WINDOWS_1251.decode(&raw);

    // The body looks like {"content":"..."} but the text may contain raw
    // newlines and quotes, which breaks strict JSON parsers - cut the
    // field out manually and normalise the line endings.
    let text = decoded
        .trim()
        .strip_prefix("{\"content\":\"")?
        .strip_suffix("\"}")?
        .replace("\r\n", "\n")
        .trim()
        .to_owned();
    if text.is_empty() {
        None
    } else {
        Some(text)
    }
}

pub(crate) async fn choose(ctx: Ctx, svc: Services) -> Result<()> {
    // Comma-separated first, whitespace as fallback.
    let raw = ctx.args().trim();
    if raw.is_empty() {
        let em = Embed::new()
            .title(svc.tr(&ctx, "choose_title").await)
            .description(svc.tr(&ctx, "choose_usage").await)
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
            .title(svc.tr(&ctx, "choose_title").await)
            .description(svc.tr(&ctx, "choose_need_two").await)
            .color(COLOR_WARN);
        return ctx.reply_with(Reply::embed(em)).await;
    }
    let pick = {
        let mut rng = rand::thread_rng();
        options.choose(&mut rng).copied().unwrap_or("?")
    };
    let em = Embed::new()
        .title(svc.tr(&ctx, "choose_title").await)
        .field(svc.tr(&ctx, "choose_options").await, options.join(" · "))
        .field(svc.tr(&ctx, "choose_pick").await, format!("**{pick}**"))
        .color(COLOR_ACCENT);
    ctx.reply_with(Reply::embed(em)).await
}

pub(crate) async fn reverse(ctx: Ctx, svc: Services) -> Result<()> {
    let text = ctx.args();
    if text.trim().is_empty() {
        let em = Embed::new()
            .title(svc.tr(&ctx, "reverse_title").await)
            .description(svc.tr(&ctx, "reverse_usage").await)
            .color(COLOR_WARN);
        return ctx.reply_with(Reply::embed(em)).await;
    }
    // Reverse by grapheme clusters so emoji, flags and combining marks
    // survive the flip instead of being torn apart char-by-char.
    let reversed: String = UnicodeSegmentation::graphemes(text, true).rev().collect();
    let em = Embed::new()
        .title(svc.tr(&ctx, "reverse_title").await)
        .field(svc.tr(&ctx, "reverse_input").await, text.to_string())
        .field(svc.tr(&ctx, "reverse_output").await, reversed)
        .color(COLOR_ACCENT);
    ctx.reply_with(Reply::embed(em)).await
}

const EMOJI_POOL: &[&str] = &[
    "\u{1F436}",
    "\u{1F431}",
    "\u{1F98A}",
    "\u{1F43C}",
    "\u{1F428}",
    "\u{1F981}",
    "\u{1F42F}",
    "\u{1F984}",
    "\u{1F98B}",
    "\u{1F33B}",
    "\u{1F337}",
    "\u{1F352}",
    "\u{1F34A}",
    "\u{1F355}",
    "\u{1F366}",
    "\u{1F31F}",
    "\u{2728}",
    "\u{1F308}",
    "\u{1F31E}",
    "\u{1F319}",
    "\u{1F680}",
    "\u{1F3AE}",
    "\u{1F3B2}",
    "\u{1F3A8}",
];

pub(crate) fn random_emoji() -> &'static str {
    let mut rng = rand::thread_rng();
    EMOJI_POOL.choose(&mut rng).copied().unwrap_or("\u{2728}")
}

pub(crate) async fn emoji(ctx: Ctx, svc: Services) -> Result<()> {
    let pick = random_emoji();
    let em = Embed::new()
        .title(svc.trf(&ctx, "emoji_title", &[pick]).await)
        .description(svc.tr(&ctx, "emoji_body").await)
        .color(COLOR_ACCENT);
    ctx.reply_with(Reply::embed(em)).await
}

// ---------- Menu (buttons demo) ---------------------------------------------

/// `/menu` - callback ids look like `menu:<invoker_id>:<action>`.
/// Only the user who ran `/menu` is allowed to tap its buttons.
pub(crate) async fn menu(ctx: Ctx, svc: Services) -> Result<()> {
    if let Some(data) = ctx.callback_data() {
        if let Some(rest) = data.strip_prefix("menu:") {
            let mut parts = rest.splitn(2, ':');
            let invoker = parts.next().unwrap_or("");
            let action = parts.next().unwrap_or("").to_ascii_lowercase();
            if invoker != ctx.user_id() {
                return ctx
                    .reply_temporary(svc.tr(&ctx, "not_your_button").await, 5)
                    .await;
            }
            return run_menu_action(ctx, &svc, &action).await;
        }
    }
    let invoker = ctx.user_id().to_owned();
    let kb = menu_keyboard(&invoker);
    let em = Embed::new()
        .title(svc.tr(&ctx, "menu_title").await)
        .description(svc.tr(&ctx, "menu_body").await)
        .color(COLOR_ACCENT);
    ctx.reply_with(Reply::embed(em).keyboard(kb)).await
}

fn menu_keyboard(invoker: &str) -> Keyboard {
    Keyboard::new()
        .row([
            Button::callback("\u{1F3B0} Coin", format!("menu:{invoker}:coin")),
            Button::callback("\u{1F3B2} Roll", format!("menu:{invoker}:roll")),
            Button::callback("\u{1F3B1} 8-ball", format!("menu:{invoker}:8ball")),
        ])
        .row([
            Button::callback("\u{1F431} Cat", format!("menu:{invoker}:cat")),
            Button::callback("\u{1F3AD} Joke", format!("menu:{invoker}:joke")),
            Button::callback("\u{2728} Emoji", format!("menu:{invoker}:emoji")),
        ])
        .row([
            Button::callback("\u{1F550} Time", format!("menu:{invoker}:time")),
            Button::callback("\u{1FAA8} RPS", format!("menu:{invoker}:rps")),
        ])
        .row([Button::url(
            "\u{1F310} bot.fouko.xyz",
            "https://bot.fouko.xyz",
        )])
}

async fn run_menu_action(ctx: Ctx, svc: &Services, action: &str) -> Result<()> {
    // The cat action carries a picture; the rest are plain text. We build
    // the embed differently so the image renders on both platforms (a real
    // embed image on Discord, a link preview on Telegram) instead of a bare
    // URL sitting in the body.
    let mut image: Option<String> = None;
    let (title, body) = match action {
        "coin" => {
            let heads = {
                let mut rng = rand::thread_rng();
                rng.gen_bool(0.5)
            };
            let side = if heads {
                format!("{} \u{1FA99}", svc.tr(&ctx, "coin_heads").await)
            } else {
                format!("{} \u{1F4B0}", svc.tr(&ctx, "coin_tails").await)
            };
            (
                svc.trf(&ctx, "coin_title", &["\u{1F3B0}"]).await,
                svc.trf(&ctx, "menu_coin_result", &[&side]).await,
            )
        }
        "roll" => {
            let n = {
                let mut rng = rand::thread_rng();
                rng.gen_range(1..=6)
            };
            (
                svc.tr(&ctx, "menu_roll_title").await,
                format!("d6 → **{n}**"),
            )
        }
        "cat" => {
            if svc.rate_limited(&ctx, "cat", 3).await? {
                return Ok(());
            }
            image = Some(format!(
                "https://cataas.com/cat?ts={}",
                chrono::Utc::now().timestamp_millis()
            ));
            (
                svc.tr(&ctx, "cat_title").await,
                svc.tr(&ctx, "cat_body").await,
            )
        }
        "time" => {
            let now = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
            (svc.tr(&ctx, "menu_time_title").await, now)
        }
        "8ball" => {
            // Same localised pool the /8ball command draws from.
            let answers = svc.tr(&ctx, "8ball_answers").await;
            let options: Vec<&str> = answers.split('|').map(str::trim).collect();
            let pick = {
                let mut rng = rand::thread_rng();
                options.choose(&mut rng).copied().unwrap_or("...")
            };
            (svc.tr(&ctx, "8ball_title").await, format!("**{pick}**"))
        }
        "joke" => {
            if svc.rate_limited(&ctx, "joke", 3).await? {
                return Ok(());
            }
            let pick = match fetch_joke("Any", "en").await {
                Some(j) => j,
                None => svc.tr(&ctx, "joke_napping").await,
            };
            (svc.tr(&ctx, "joke_title").await, pick)
        }
        "emoji" => {
            let pick = random_emoji();
            (svc.trf(&ctx, "emoji_title", &[pick]).await, pick.to_owned())
        }
        "rps" => {
            // A quick round against the bot with a random move for the
            // player too - the full game with move buttons lives in /rps.
            const MOVES: [&str; 3] = ["rock", "paper", "scissors"];
            let (you, bot_move) = {
                let mut rng = rand::thread_rng();
                (MOVES[rng.gen_range(0..3)], MOVES[rng.gen_range(0..3)])
            };
            let verdict_key = if you == bot_move {
                "rps_draw"
            } else if matches!(
                (you, bot_move),
                ("rock", "scissors") | ("paper", "rock") | ("scissors", "paper")
            ) {
                "rps_win"
            } else {
                "rps_lose"
            };
            let move_name = |m: &str| match m {
                "rock" => "rps_rock",
                "paper" => "rps_paper",
                _ => "rps_scissors",
            };
            let you_name = svc.tr(&ctx, move_name(you)).await;
            let bot_name = svc.tr(&ctx, move_name(bot_move)).await;
            let verdict = svc.tr(&ctx, verdict_key).await;
            (
                svc.tr(&ctx, "rps_title").await,
                svc.trf(&ctx, "menu_rps_body", &[&you_name, &bot_name, &verdict])
                    .await,
            )
        }
        _ => (
            svc.tr(&ctx, "menu_title").await,
            svc.tr(&ctx, "menu_unknown_action").await,
        ),
    };
    let invoker = ctx.user_id().to_owned();
    let kb = menu_keyboard(&invoker);
    let mut em = Embed::new()
        .title(title)
        .description(body)
        .footer(svc.tr(&ctx, "menu_footer").await)
        .color(COLOR_ACCENT);
    if let Some(url) = image {
        em = em.image(url);
    }
    ctx.edit_reply(Reply::embed(em).keyboard(kb)).await
}

// ---------- Rock paper scissors ---------------------------------------------

/// `/rps` - plays on buttons. Callback ids: `rps:<invoker>:<choice>`.
pub(crate) async fn rps(ctx: Ctx, svc: Services) -> Result<()> {
    if let Some(data) = ctx.callback_data() {
        if let Some(rest) = data.strip_prefix("rps:") {
            let mut parts = rest.splitn(2, ':');
            let invoker = parts.next().unwrap_or("").to_owned();
            let choice = parts.next().unwrap_or("").to_owned();
            if invoker != ctx.user_id() {
                return ctx
                    .reply_temporary(svc.tr(&ctx, "not_your_button").await, 5)
                    .await;
            }
            return rps_result(ctx, &svc, &choice).await;
        }
    }
    let invoker = ctx.user_id().to_owned();
    let kb = Keyboard::new().row([
        Button::callback(
            format!("\u{1FAA8} {}", svc.tr(&ctx, "rps_rock").await),
            format!("rps:{invoker}:rock"),
        ),
        Button::callback(
            format!("\u{1F4C4} {}", svc.tr(&ctx, "rps_paper").await),
            format!("rps:{invoker}:paper"),
        ),
        Button::callback(
            format!("\u{2702}\u{FE0F} {}", svc.tr(&ctx, "rps_scissors").await),
            format!("rps:{invoker}:scissors"),
        ),
    ]);
    let em = Embed::new()
        .title(svc.tr(&ctx, "rps_title").await)
        .description(svc.tr(&ctx, "rps_prompt").await)
        .color(COLOR_ACCENT);
    ctx.reply_with(Reply::embed(em).keyboard(kb)).await
}

async fn rps_result(ctx: Ctx, svc: &Services, choice: &str) -> Result<()> {
    const MOVES: [&str; 3] = ["rock", "paper", "scissors"];
    let player = match choice {
        "rock" | "paper" | "scissors" => choice,
        _ => "rock",
    };
    let bot_move = {
        let mut rng = rand::thread_rng();
        MOVES.choose(&mut rng).copied().unwrap_or("rock")
    };

    let outcome = if player == bot_move {
        "draw"
    } else if (player == "rock" && bot_move == "scissors")
        || (player == "paper" && bot_move == "rock")
        || (player == "scissors" && bot_move == "paper")
    {
        "win"
    } else {
        "lose"
    };

    let icon = |m: &str| match m {
        "rock" => "\u{1FAA8}",
        "paper" => "\u{1F4C4}",
        _ => "\u{2702}\u{FE0F}",
    };
    let move_key = |m: &str| match m {
        "rock" => "rps_rock",
        "paper" => "rps_paper",
        _ => "rps_scissors",
    };
    let (verdict_key, color) = match outcome {
        "win" => ("rps_win", COLOR_OK),
        "lose" => ("rps_lose", COLOR_WARN),
        _ => ("rps_draw", COLOR_ACCENT),
    };
    let player_name = svc.tr(&ctx, move_key(player)).await;
    let bot_name = svc.tr(&ctx, move_key(bot_move)).await;
    let em = Embed::new()
        .title(svc.tr(&ctx, "rps_title").await)
        .field_inline(
            svc.tr(&ctx, "rps_you").await,
            format!("{} {player_name}", icon(player)),
        )
        .field_inline(
            svc.tr(&ctx, "rps_bot").await,
            format!("{} {bot_name}", icon(bot_move)),
        )
        .description(format!("**{}**", svc.tr(&ctx, verdict_key).await))
        .color(color);
    ctx.edit_reply(Reply::embed(em)).await
}

// ---------- Poll ------------------------------------------------------------

const POLL_PREFIX: &str = "foukobot:poll:";
const POLL_VOTE_PREFIX: &str = "foukobot:pollvote:";
const POLL_MAX_OPTIONS: usize = 5;

/// Serialises poll vote updates: the tally is a read-modify-write over
/// storage, so two simultaneous button presses could otherwise lose one
/// of the votes. A single global lock is plenty at chat-poll scale.
static POLL_VOTE_LOCK: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

/// A poll's persisted state: the question, its options and a tally per
/// option. Stored as `question\n<count>\t<option>\n...` so we don't pull
/// in a JSON dependency for a handful of fields.
pub(crate) struct PollState {
    question: String,
    options: Vec<String>,
    counts: Vec<u64>,
}

impl PollState {
    fn serialize(&self) -> String {
        let mut out = self.question.clone();
        for (count, opt) in self.counts.iter().zip(&self.options) {
            out.push('\n');
            out.push_str(&format!("{count}\t{opt}"));
        }
        out
    }

    fn parse(blob: &str) -> Option<Self> {
        let mut lines = blob.lines();
        let question = lines.next()?.to_owned();
        let mut options = Vec::new();
        let mut counts = Vec::new();
        for line in lines {
            let (count, opt) = line.split_once('\t')?;
            counts.push(count.parse().unwrap_or(0));
            options.push(opt.to_owned());
        }
        if options.is_empty() {
            return None;
        }
        Some(Self {
            question,
            options,
            counts,
        })
    }

    /// Total votes cast so far.
    fn total(&self) -> u64 {
        self.counts.iter().sum()
    }

    /// Render the embed body: each option with its share as a small bar.
    /// `votes_line` is the pre-translated "N votes" footer, if any.
    fn body(&self, votes_line: Option<&str>) -> String {
        let total = self.total();
        let mut out = String::new();
        for (i, (opt, count)) in self.options.iter().zip(&self.counts).enumerate() {
            let bar = progress_bar(*count, total.max(1), 12);
            out.push_str(&format!("{}. **{opt}** - `{bar}` {count}\n", i + 1));
        }
        if let Some(line) = votes_line {
            out.push_str(&format!("\n_{line}_"));
        }
        out
    }
}

/// The localised "1 vote" / "N votes" line, or `None` when nobody voted.
async fn poll_votes_line(ctx: &Ctx, svc: &Services, state: &PollState) -> Option<String> {
    match state.total() {
        0 => None,
        1 => Some(svc.tr(ctx, "poll_vote_one").await),
        n => Some(svc.trf(ctx, "poll_votes", &[&n.to_string()]).await),
    }
}

/// `/poll question | option a | option b`. Buttons carry
/// `poll:<id>:<idx>`; tallies live in storage so votes actually count and
/// nobody votes twice.
pub(crate) async fn poll(ctx: Ctx, svc: Services) -> Result<()> {
    // A button press: record the vote and redraw the poll message.
    if let Some(data) = ctx.callback_data() {
        if let Some(rest) = data.strip_prefix("poll:") {
            return poll_vote(&ctx, &svc, rest).await;
        }
    }

    let raw = ctx.args().trim();
    let mut parts: Vec<&str> = raw
        .split('|')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .collect();
    if parts.len() < 3 {
        let em = Embed::new()
            .title(svc.tr(&ctx, "poll_title").await)
            .description(svc.tr(&ctx, "poll_usage").await)
            .color(COLOR_WARN);
        return ctx.reply_with(Reply::embed(em)).await;
    }
    // Too many options: tell the user instead of silently dropping the
    // extras.
    if parts.len() - 1 > POLL_MAX_OPTIONS {
        let em = Embed::new()
            .title(svc.tr(&ctx, "poll_title").await)
            .description(
                svc.trf(&ctx, "poll_too_many", &[&POLL_MAX_OPTIONS.to_string()])
                    .await,
            )
            .color(COLOR_WARN);
        return ctx.reply_with(Reply::embed(em)).await;
    }
    // Each poll writes rows to storage, so don't let one user machine-gun
    // them into the database.
    if svc.rate_limited(&ctx, "poll_new", 30).await? {
        return Ok(());
    }
    let question = parts.remove(0).to_owned();

    // Sanitise: the storage format is line/tab-delimited, so strip both
    // from user text (a Discord slash arg can carry real newlines), and
    // keep sizes sane.
    let question: String = question
        .replace(['\n', '\t'], " ")
        .chars()
        .take(200)
        .collect();
    let options: Vec<String> = parts
        .iter()
        .map(|s| s.replace(['\n', '\t'], " ").chars().take(80).collect())
        .collect();

    // A short id keyed off the clock is plenty for a chat poll. set_nx
    // guards the freak case of two polls in the same millisecond: instead
    // of silently overwriting, nudge the id until it lands on a free slot.
    let state = PollState {
        question: question.clone(),
        options,
        counts: vec![0; parts.len()],
    };
    let mut poll_id = String::new();
    for bump in 0..16i64 {
        let candidate = format!("{:x}", Utc::now().timestamp_millis() + bump);
        if svc
            .storage
            .set_nx(&format!("{POLL_PREFIX}{candidate}"), &state.serialize())
            .await?
        {
            poll_id = candidate;
            break;
        }
    }
    if poll_id.is_empty() {
        // 16 straight collisions means the clock is stuck; give up quietly.
        return Ok(());
    }

    // Housekeeping: polls have no natural end, so whenever someone makes a
    // new one, sweep out those older than a week along with their votes.
    // Piggybacking on creation keeps it cheap - no background task needed.
    sweep_old_polls(&svc).await;

    let row: Vec<Button> = (0..parts.len())
        .map(|i| Button::callback(format!("{}", i + 1), format!("poll:{poll_id}:{i}")))
        .collect();
    let kb = Keyboard::new().row(row);
    let votes_line = poll_votes_line(&ctx, &svc, &state).await;
    let em = Embed::new()
        .title(format!("\u{1F4CA} {question}"))
        .description(state.body(votes_line.as_deref()))
        .footer(svc.tr(&ctx, "poll_footer").await)
        .color(COLOR_ACCENT);
    ctx.reply_with(Reply::embed(em).keyboard(kb)).await
}

/// Handle a `poll:<id>:<idx>` button press.
/// Drop polls (and their vote markers) older than a week. The poll id is
/// hex millis of its creation, which doubles as the age stamp.
async fn sweep_old_polls(svc: &Services) {
    let cutoff = Utc::now().timestamp_millis() - 7 * 24 * 3600 * 1000;
    let polls = svc
        .storage
        .list_prefix(POLL_PREFIX)
        .await
        .unwrap_or_default();
    for (key, _) in polls {
        let Some(id) = key.strip_prefix(POLL_PREFIX) else {
            continue;
        };
        let Ok(created) = i64::from_str_radix(id, 16) else {
            let _ = svc.storage.del(&key).await; // unreadable id: drop it
            continue;
        };
        if created < cutoff {
            let _ = svc.storage.del(&key).await;
            let votes = svc
                .storage
                .list_prefix(&format!("{POLL_VOTE_PREFIX}{id}:"))
                .await
                .unwrap_or_default();
            for (vkey, _) in votes {
                let _ = svc.storage.del(&vkey).await;
            }
        }
    }
}

async fn poll_vote(ctx: &Ctx, svc: &Services, rest: &str) -> Result<()> {
    let Some((poll_id, idx_str)) = rest.split_once(':') else {
        return Ok(());
    };
    let Ok(idx) = idx_str.parse::<usize>() else {
        return Ok(());
    };

    // The tally update below is read-modify-write over two storage rows,
    // so hold the lock for the whole thing - two near-simultaneous taps
    // must not both read the same counts.
    let _guard = POLL_VOTE_LOCK.lock().await;

    let key = format!("{POLL_PREFIX}{poll_id}");
    let Some(blob) = svc.storage.get(&key).await? else {
        // Poll expired or was never stored (e.g. after a restart with an
        // in-memory store). Nothing to update - and the notice is noise
        // once read, so let it clean itself up.
        return ctx.reply_temporary(svc.tr(ctx, "poll_gone").await, 5).await;
    };
    let Some(mut state) = PollState::parse(&blob) else {
        return Ok(());
    };
    if idx >= state.options.len() {
        return Ok(());
    }

    // One vote per user per poll. The marker also remembers which option
    // they picked, so a re-tap can move the vote instead of being ignored.
    let vote_key = format!("{POLL_VOTE_PREFIX}{poll_id}:{}", ctx.user_id());
    if let Some(prev) = svc.storage.get(&vote_key).await? {
        if let Ok(prev_idx) = prev.parse::<usize>() {
            if prev_idx == idx {
                return ctx
                    .reply_temporary(svc.tr(ctx, "poll_already").await, 5)
                    .await;
            }
            if let Some(c) = state.counts.get_mut(prev_idx) {
                *c = c.saturating_sub(1);
            }
        }
    }
    state.counts[idx] += 1;
    svc.storage.set(&vote_key, &idx.to_string()).await?;
    svc.storage.set(&key, &state.serialize()).await?;

    let votes_line = poll_votes_line(ctx, svc, &state).await;
    let em = Embed::new()
        .title(format!("\u{1F4CA} {}", state.question))
        .description(state.body(votes_line.as_deref()))
        .footer(svc.tr(ctx, "poll_footer").await)
        .color(COLOR_ACCENT);
    // Rebuild the same buttons so they survive the edit.
    let row: Vec<Button> = (0..state.options.len())
        .map(|i| Button::callback(format!("{}", i + 1), format!("poll:{poll_id}:{i}")))
        .collect();
    let kb = Keyboard::new().row(row);
    ctx.edit_reply(Reply::embed(em).keyboard(kb)).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn poll_state_round_trips() {
        let state = PollState {
            question: "Best language?".to_owned(),
            options: vec!["Rust".to_owned(), "Go".to_owned()],
            counts: vec![3, 1],
        };
        let blob = state.serialize();
        let parsed = PollState::parse(&blob).expect("parses back");
        assert_eq!(parsed.question, "Best language?");
        assert_eq!(parsed.options, vec!["Rust", "Go"]);
        assert_eq!(parsed.counts, vec![3, 1]);
    }

    #[test]
    fn poll_state_parse_rejects_empty() {
        assert!(PollState::parse("").is_none());
        assert!(PollState::parse("Only a question, no options").is_none());
    }
}
