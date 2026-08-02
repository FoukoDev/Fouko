//! Tool commands: time, weather, QR, URL shortener, calculator, reminders.

use super::helpers::{format_offset, human_duration, parse_duration, parse_utc_offset};
use super::Services;
use super::{COLOR_ACCENT, COLOR_OK, COLOR_WARN};
use chrono::Utc;
use foukoapi::{util::urlencode, Button, Ctx, Embed, Keyboard, PlatformKind, Reply, Result};

/// `/time`, `/time +10`, `/time -5:30`.
pub(crate) async fn time(ctx: Ctx, svc: Services) -> Result<()> {
    let arg = ctx.args().trim();

    let title = svc.tr(&ctx, "time_title").await;
    let when_label = svc.tr(&ctx, "time_now").await;
    let tz_label = svc.tr(&ctx, "time_tz").await;

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
            let em = Embed::new()
                .title(title)
                .description(svc.tr(&ctx, "time_bad_offset").await)
                .color(COLOR_WARN);
            ctx.reply_with(Reply::embed(em)).await
        }
    }
}

/// `/qr TEXT` - renders a QR code pointing at TEXT using goqr.me.
pub(crate) async fn qr(ctx: Ctx, svc: Services) -> Result<()> {
    let text = ctx.args().trim();
    if text.is_empty() {
        let em = Embed::new()
            .title(svc.tr(&ctx, "qr_title").await)
            .description(svc.tr(&ctx, "qr_usage").await)
            .color(COLOR_WARN);
        return ctx.reply_with(Reply::embed(em)).await;
    }
    // Count characters, not bytes - the limit the message promises.
    if text.chars().count() > 800 {
        let em = Embed::new()
            .title(svc.tr(&ctx, "qr_title").await)
            .description(svc.tr(&ctx, "qr_too_long").await)
            .color(COLOR_WARN);
        return ctx.reply_with(Reply::embed(em)).await;
    }
    if svc.rate_limited(&ctx, "qr", 3).await? {
        return Ok(());
    }
    let url = format!(
        "https://api.qrserver.com/v1/create-qr-code/?size=400x400&data={}",
        urlencode(text)
    );
    let shown: String = text.chars().take(80).collect();
    let em = Embed::new()
        .title(svc.tr(&ctx, "qr_title").await)
        .description(svc.trf(&ctx, "qr_encoded", &[&shown]).await)
        .image(url)
        .color(COLOR_ACCENT);
    ctx.reply_with(Reply::embed(em)).await
}

/// `/shorten URL` - shortens a link. Tries TinyURL first, then is.gd as a
/// fallback, so one provider being down doesn't kill the command.
pub(crate) async fn shorten(ctx: Ctx, svc: Services) -> Result<()> {
    let raw = ctx.args().trim();
    if raw.is_empty() {
        let em = Embed::new()
            .title(svc.tr(&ctx, "shorten_title").await)
            .description(svc.tr(&ctx, "shorten_usage").await)
            .color(COLOR_WARN);
        return ctx.reply_with(Reply::embed(em)).await;
    }
    if !(raw.starts_with("http://") || raw.starts_with("https://")) {
        let em = Embed::new()
            .title(svc.tr(&ctx, "shorten_title").await)
            .description(svc.tr(&ctx, "shorten_bad_url").await)
            .color(COLOR_WARN);
        return ctx.reply_with(Reply::embed(em)).await;
    }
    if svc.rate_limited(&ctx, "shorten", 5).await? {
        return Ok(());
    }

    match shorten_url(raw).await {
        Some(short) => {
            let em = Embed::new()
                .title(svc.tr(&ctx, "shorten_done_title").await)
                .field(svc.tr(&ctx, "shorten_original").await, format!("`{raw}`"))
                .field(svc.tr(&ctx, "shorten_short").await, format!("**{short}**"))
                .color(COLOR_ACCENT);
            ctx.reply_with(Reply::embed(em)).await
        }
        None => {
            let em = Embed::new()
                .title(svc.tr(&ctx, "shorten_failed_title").await)
                .description(svc.tr(&ctx, "shorten_failed_body").await)
                .color(COLOR_WARN);
            ctx.reply_with(Reply::embed(em)).await
        }
    }
}

/// Ask each provider in turn for a short link; return the first that works.
async fn shorten_url(raw: &str) -> Option<String> {
    let encoded = urlencode(raw);
    let endpoints = [
        format!("https://tinyurl.com/api-create.php?url={encoded}"),
        format!("https://is.gd/create.php?format=simple&url={encoded}"),
        format!("https://v.gd/create.php?format=simple&url={encoded}"),
    ];
    // Some shorteners reject requests without a User-Agent, so set one.
    let client = reqwest::Client::builder()
        .user_agent("FoukoBot/1.0")
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .ok()?;
    for url in endpoints {
        let Ok(resp) = client.get(&url).send().await else {
            continue;
        };
        if !resp.status().is_success() {
            continue;
        }
        let Ok(body) = resp.text().await else {
            continue;
        };
        let short = body.trim();
        if short.starts_with("http") {
            return Some(short.to_owned());
        }
    }
    None
}

// ---------- Weather ---------------------------------------------------------

pub(crate) async fn weather(ctx: Ctx, svc: Services) -> Result<()> {
    let lang = svc.lang(&ctx).await;
    let city = ctx.args().trim();
    if city.is_empty() {
        let em = Embed::new()
            .title(svc.tr(&ctx, "weather_title").await)
            .description(svc.tr(&ctx, "weather_usage").await)
            .color(COLOR_WARN);
        return ctx.reply_with(Reply::embed(em)).await;
    }
    if svc.rate_limited(&ctx, "weather", 5).await? {
        return Ok(());
    }
    match fetch_weather_embed(&ctx, &svc, city, &lang).await {
        Ok(em) => ctx.reply_with(Reply::embed(em)).await,
        Err(e) => {
            tracing::warn!(error = %e, "weather fetch failed");
            // Display doesn't know the user's language, so map the error
            // variant to its catalogue key right here.
            let reason = match e {
                WeatherError::CityNotFound => svc.tr(&ctx, "weather_err_city").await,
                WeatherError::Http(code) => {
                    svc.trf(&ctx, "weather_err_http", &[&code.to_string()])
                        .await
                }
                WeatherError::Network => svc.tr(&ctx, "weather_err_network").await,
                WeatherError::BadJson => svc.tr(&ctx, "weather_err_json").await,
            };
            let em = Embed::new()
                .title(svc.tr(&ctx, "weather_failed_title").await)
                .description(svc.trf(&ctx, "weather_failed_body", &[&reason]).await)
                .color(COLOR_WARN);
            ctx.reply_with(Reply::embed(em)).await
        }
    }
}

/// Weather lookup errors.
#[derive(Debug)]
enum WeatherError {
    CityNotFound,
    Http(u16),
    Network,
    BadJson,
}

impl std::fmt::Display for WeatherError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::CityNotFound => f.write_str("couldn't find that city"),
            Self::Http(code) => write!(f, "the weather provider answered HTTP {code}"),
            Self::Network => f.write_str("the weather provider is unreachable"),
            Self::BadJson => f.write_str("the weather provider sent an odd response"),
        }
    }
}

impl std::error::Error for WeatherError {}

fn weather_client() -> std::result::Result<reqwest::Client, WeatherError> {
    reqwest::Client::builder()
        .user_agent("FoukoBot/1.0")
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|_| WeatherError::Network)
}

/// Resolve `city` and return a ready-to-send [`Embed`]. Runs on
/// open-meteo.com, which is free and needs no API key: one call to its
/// geocoder, one for the current conditions.
async fn fetch_weather_embed(
    ctx: &Ctx,
    svc: &Services,
    city: &str,
    lang: &str,
) -> std::result::Result<Embed, WeatherError> {
    let client = weather_client()?;
    let (lat, lon, nice_name) = geocode_city(&client, city, lang)
        .await?
        .ok_or(WeatherError::CityNotFound)?;

    #[derive(serde::Deserialize)]
    struct Resp {
        current: Current,
        /// Offset of the city's timezone from UTC, for the local clock.
        #[serde(default)]
        utc_offset_seconds: i64,
    }
    #[derive(serde::Deserialize)]
    struct Current {
        temperature_2m: f64,
        apparent_temperature: f64,
        relative_humidity_2m: f64,
        wind_speed_10m: f64,
        weather_code: u8,
    }

    let url = format!(
        "https://api.open-meteo.com/v1/forecast?latitude={lat}&longitude={lon}\
         &current=temperature_2m,apparent_temperature,relative_humidity_2m,wind_speed_10m,weather_code\
         &wind_speed_unit=ms&timezone=auto"
    );
    let resp = client
        .get(&url)
        .send()
        .await
        .map_err(|_| WeatherError::Network)?;
    if !resp.status().is_success() {
        return Err(WeatherError::Http(resp.status().as_u16()));
    }
    let parsed: Resp = resp.json().await.map_err(|_| WeatherError::BadJson)?;
    let cur = parsed.current;

    // Local wall-clock time in the city, from UTC plus the zone offset.
    let local = chrono::Utc::now() + chrono::Duration::seconds(parsed.utc_offset_seconds);
    let local_time = local.format("%H:%M").to_string();

    let (cond_icon, cond_key) = describe_weather_code(cur.weather_code);
    let cond = svc.tr(ctx, cond_key).await;

    Ok(Embed::new()
        .title(format!("{cond_icon} {nice_name}"))
        .description(cond)
        .field_inline(
            svc.tr(ctx, "weather_label_temp").await,
            format!("{:.1}\u{00B0}C", cur.temperature_2m),
        )
        .field_inline(
            svc.tr(ctx, "weather_label_feels").await,
            format!("{:.1}\u{00B0}C", cur.apparent_temperature),
        )
        .field_inline(
            svc.tr(ctx, "weather_label_humidity").await,
            format!("{}%", cur.relative_humidity_2m),
        )
        .field_inline(
            svc.tr(ctx, "weather_label_wind").await,
            format!(
                "{:.1} {}",
                cur.wind_speed_10m,
                svc.tr(ctx, "weather_wind_unit").await
            ),
        )
        .field_inline(
            svc.tr(ctx, "weather_label_local_time").await,
            format!("\u{1F550} {local_time}"),
        )
        .color(COLOR_ACCENT))
}

/// open-meteo's geocoder: `(lat, lon, "City, CC")` of the best match, or
/// `None`. Passing the user's language gets localised city names back.
async fn geocode_city(
    client: &reqwest::Client,
    city: &str,
    lang: &str,
) -> std::result::Result<Option<(f64, f64, String)>, WeatherError> {
    #[derive(serde::Deserialize)]
    struct GeoResp {
        #[serde(default)]
        results: Vec<GeoHit>,
    }
    #[derive(serde::Deserialize)]
    struct GeoHit {
        latitude: f64,
        longitude: f64,
        name: String,
        country_code: Option<String>,
    }
    let url = format!(
        "https://geocoding-api.open-meteo.com/v1/search?name={}&count=1&language={}",
        urlencode(city),
        urlencode(lang)
    );
    let resp = client
        .get(&url)
        .send()
        .await
        .map_err(|_| WeatherError::Network)?;
    if !resp.status().is_success() {
        return Err(WeatherError::Http(resp.status().as_u16()));
    }
    let parsed: GeoResp = resp.json().await.map_err(|_| WeatherError::BadJson)?;
    let Some(hit) = parsed.results.into_iter().next() else {
        return Ok(None);
    };
    let pretty = match hit.country_code {
        Some(cc) if !cc.is_empty() => format!("{}, {cc}", hit.name),
        _ => hit.name,
    };
    Ok(Some((hit.latitude, hit.longitude, pretty)))
}

/// Map a WMO weather code to an icon and a description key; the caller
/// translates the key into the user's language.
fn describe_weather_code(code: u8) -> (&'static str, &'static str) {
    match code {
        0 => ("\u{2600}\u{FE0F}", "weather_cond_clear"),
        1..=2 => ("\u{1F324}\u{FE0F}", "weather_cond_partly"),
        3 => ("\u{2601}\u{FE0F}", "weather_cond_overcast"),
        45 | 48 => ("\u{1F32B}\u{FE0F}", "weather_cond_fog"),
        51..=57 => ("\u{1F326}\u{FE0F}", "weather_cond_drizzle"),
        61..=67 | 80..=82 => ("\u{1F327}\u{FE0F}", "weather_cond_rain"),
        71..=77 | 85..=86 => ("\u{1F328}\u{FE0F}", "weather_cond_snow"),
        95..=99 => ("\u{26C8}\u{FE0F}", "weather_cond_thunder"),
        _ => ("\u{1F324}\u{FE0F}", "weather_cond_generic"),
    }
}

// ---------- Reminder --------------------------------------------------------

const REMIND_PREFIX: &str = "foukobot:remind:";

/// `/remind 10m walk the dog` - pings back after the delay. Bare `/remind`
/// opens a small manager listing pending reminders with delete buttons.
///
/// Delivery prefers the user's DM; when that fails (Discord DMs closed,
/// bot blocked) it falls back to the chat where the reminder was created,
/// mentioning the user. Reminders survive restarts: they're stored and
/// re-armed on boot (see [`restore_reminders`]).
pub(crate) async fn remind(ctx: Ctx, svc: Services) -> Result<()> {
    // Delete button from the manager: `remind:<invoker>:del:<id>`.
    if let Some(data) = ctx.callback_data() {
        if let Some(rest) = data.strip_prefix("remind:") {
            let mut parts = rest.splitn(3, ':');
            let invoker = parts.next().unwrap_or("");
            let action = parts.next().unwrap_or("");
            let millis = parts.next().unwrap_or("");
            if invoker != ctx.user_id() {
                return ctx
                    .reply_temporary(svc.tr(&ctx, "not_your_button").await, 5)
                    .await;
            }
            if action == "del" && !millis.is_empty() {
                // Rebuild the row key from the presser's own id, so the
                // button can only ever delete that person's reminder.
                let id = format!("{millis}-{}", ctx.user_id());
                let _ = svc.storage.del(&format!("{REMIND_PREFIX}{id}")).await;
            }
            return remind_menu(&ctx, &svc, true).await;
        }
        return Ok(());
    }

    let lang = svc.lang(&ctx).await;
    let args = ctx.args().trim();

    // Bare `/remind`: show the manager.
    if args.is_empty() {
        return remind_menu(&ctx, &svc, false).await;
    }

    let (delay_str, note) = args.split_once(char::is_whitespace).unwrap_or((args, ""));
    let secs = parse_duration(delay_str);

    let (Some(secs), false) = (secs, note.trim().is_empty()) else {
        let em = Embed::new()
            .title(svc.tr(&ctx, "remind_title").await)
            .description(svc.tr(&ctx, "remind_usage").await)
            .color(COLOR_WARN);
        return ctx.reply_with(Reply::embed(em)).await;
    };

    if secs == 0 || secs > 24 * 3600 {
        let em = Embed::new()
            .title(svc.tr(&ctx, "remind_title").await)
            .description(svc.tr(&ctx, "remind_range").await)
            .color(COLOR_WARN);
        return ctx.reply_with(Reply::embed(em)).await;
    }

    // Every reminder is a stored row plus a live timer task, so cap the
    // rate and the note size - otherwise one user could stack thousands.
    if svc.rate_limited(&ctx, "remind_new", 15).await? {
        return Ok(());
    }

    // Persist first so a crash between "ack" and "fire" doesn't lose it.
    // Tabs/newlines in the note are flattened to spaces so the record's
    // simple tab-separated layout stays intact.
    let fire_ts = Utc::now().timestamp() + secs as i64;
    let note_clean: String = note
        .trim()
        .replace(['\t', '\n'], " ")
        .chars()
        .take(500)
        .collect();
    // Millis (not seconds) so two quick reminders from the same user with
    // the same target time can't collide and silently overwrite.
    let id = format!("{}-{}", Utc::now().timestamp_millis(), ctx.user_id());
    let record = format!(
        "{fire_ts}\t{}\t{}\t{}\t{note_clean}",
        ctx.platform(),
        ctx.chat_id(),
        ctx.user_id(),
    );
    svc.storage
        .set(&format!("{REMIND_PREFIX}{id}"), &record)
        .await?;

    let em = Embed::new()
        .title(svc.tr(&ctx, "remind_set_title").await)
        .description(
            svc.trf(
                &ctx,
                "remind_set_body",
                &[&human_duration(secs as i64, &lang)],
            )
            .await,
        )
        .color(COLOR_OK);
    ctx.reply_with(Reply::embed(em)).await?;

    arm_reminder(
        svc.clone(),
        Pending {
            id,
            secs,
            platform: ctx.platform(),
            chat_id: ctx.chat_id().to_owned(),
            user_id: ctx.user_id().to_owned(),
            note: note_clean,
        },
    );
    Ok(())
}

/// The reminder manager: pending list with a delete button per row.
async fn remind_menu(ctx: &Ctx, svc: &Services, edit: bool) -> Result<()> {
    let mine_suffix = format!("-{}", ctx.user_id());
    let mut entries = svc
        .storage
        .list_prefix(REMIND_PREFIX)
        .await
        .unwrap_or_default();
    // The store returns rows in whatever order it likes; the closest
    // reminder should sit on top. The key starts with creation millis,
    // but sorting by fire time is what the user actually expects.
    entries.sort_by_key(|(_, record)| {
        record
            .split('\t')
            .next()
            .and_then(|t| t.parse::<i64>().ok())
            .unwrap_or(i64::MAX)
    });
    let now = Utc::now().timestamp();
    let lang = svc.lang(ctx).await;

    let mut body = String::new();
    let mut kb = Keyboard::new();
    let mut count = 0usize;
    for (key, record) in &entries {
        let Some(id) = key.strip_prefix(REMIND_PREFIX) else {
            continue;
        };
        if !id.ends_with(&mine_suffix) {
            continue; // someone else's
        }
        let mut parts = record.splitn(5, '\t');
        let fire_ts: i64 = parts.next().unwrap_or("0").parse().unwrap_or(0);
        let (_platform, _chat, _uid) = (parts.next(), parts.next(), parts.next());
        let note = parts.next().unwrap_or("");
        let left = human_duration(fire_ts - now, &lang);
        let short: String = note.chars().take(40).collect();
        count += 1;
        body.push_str(&format!("{count}. {short} (\u{23F3} {left})\n"));
        // Only the millis half goes into the callback; the presser's own
        // id completes it on the other end. Keeps the payload well under
        // Telegram's 64-byte cap and makes forging someone else's delete
        // impossible.
        let millis = id.strip_suffix(&mine_suffix).unwrap_or(id);
        kb = kb.row([Button::callback(
            format!("\u{1F5D1} {count}"),
            format!("remind:{}:del:{millis}", ctx.user_id()),
        )]);
        if count >= 10 {
            break; // keep the card manageable
        }
    }

    let em = if count == 0 {
        Embed::new()
            .title(svc.tr(ctx, "remind_title").await)
            .description(svc.tr(ctx, "remind_none").await)
            .color(COLOR_ACCENT)
    } else {
        Embed::new()
            .title(svc.tr(ctx, "remind_title").await)
            .description(body.trim_end().to_owned())
            .footer(svc.tr(ctx, "remind_menu_hint").await)
            .color(COLOR_ACCENT)
    };

    if edit {
        ctx.edit_reply(Reply::embed(em).keyboard(kb)).await
    } else {
        ctx.reply_with(Reply::embed(em).keyboard(kb)).await
    }
}

/// Everything needed to deliver one reminder.
struct Pending {
    id: String,
    secs: u64,
    platform: PlatformKind,
    chat_id: String,
    user_id: String,
    note: String,
}

/// Spawn the timer that delivers one reminder and clears its record.
///
/// Delivery order: DM first; if the platform rejects it (user never opened
/// a DM, blocked the bot), fall back to the chat where the reminder was
/// set, with a mention so it isn't missed.
fn arm_reminder(svc: Services, p: Pending) {
    tokio::spawn(async move {
        tokio::time::sleep(std::time::Duration::from_secs(p.secs)).await;

        // The row may be gone if the user deleted the reminder meanwhile.
        let key = format!("{REMIND_PREFIX}{}", p.id);
        if svc.storage.get(&key).await.ok().flatten().is_none() {
            return;
        }

        // The ping goes to the reminder's owner, so translate the title
        // into their language, not whatever the current context holds.
        let lang = svc.lang_of(p.platform, &p.user_id).await;
        let title = svc.i18n.t(&lang, "remind_ping_title");

        let em = Embed::new()
            .title(title.clone())
            .description(p.note.clone())
            .color(COLOR_ACCENT);

        // In Telegram a DM chat id equals the user id; Discord needs the
        // user's DM channel which we can't open from here, so for Discord
        // the DM attempt only works when the reminder was set in a DM.
        let dm_target = p.user_id.clone();
        let dm_ok = svc
            .notifier
            .send(p.platform, dm_target, Reply::embed(em.clone()))
            .await
            .is_ok();

        if !dm_ok {
            // Group fallback needs a mention so the ping isn't missed.
            // Telegram uses a tg://user deep link, which md_to_tg turns
            // into a working profile link.
            let mention = match p.platform {
                PlatformKind::Discord => format!("<@{}>", p.user_id),
                PlatformKind::Telegram => {
                    let primary = svc
                        .accounts
                        .primary_for(p.platform, &p.user_id)
                        .await
                        .unwrap_or_else(|_| format!("{}:{}", p.platform, p.user_id));
                    let name = svc
                        .econ
                        .display_name_of(&primary)
                        .await
                        .unwrap_or_else(|| title.clone());
                    format!("[{name}](tg://user?id={})", p.user_id)
                }
                _ => String::new(),
            };
            let body = Embed::new()
                .title(title)
                .description(p.note)
                .color(COLOR_ACCENT);
            let reply = if mention.is_empty() {
                Reply::embed(body)
            } else {
                Reply::text(mention).with_embed(body)
            };
            if let Err(e) = svc.notifier.send(p.platform, p.chat_id, reply).await {
                tracing::warn!(error = %e, "reminder delivery failed");
            }
        }
        let _ = svc.storage.del(&key).await;
    });
}

/// Reload pending reminders on startup and re-arm their timers. Anything
/// already overdue fires almost immediately; the rest wait out the
/// remaining time. Call this once after the notifier's adapters are up.
pub async fn restore_reminders(svc: Services) {
    let entries = match svc.storage.list_prefix(REMIND_PREFIX).await {
        Ok(e) => e,
        Err(e) => {
            tracing::warn!(error = %e, "could not load pending reminders");
            return;
        }
    };
    let now = Utc::now().timestamp();
    let mut restored = 0u32;
    for (key, record) in entries {
        let Some(id) = key.strip_prefix(REMIND_PREFIX) else {
            continue;
        };
        let mut parts = record.splitn(5, '\t');
        let (Some(fire), Some(platform_str), Some(chat_id), Some(user_id), note) = (
            parts.next(),
            parts.next(),
            parts.next(),
            parts.next(),
            parts.next().unwrap_or(""),
        ) else {
            // Old-format or corrupt rows: drop them rather than carry
            // them forever.
            let _ = svc.storage.del(&key).await;
            continue;
        };
        // A record whose fire time doesn't parse is corrupt - drop it
        // rather than fire it at a random moment.
        let Ok(fire_ts) = fire.parse::<i64>() else {
            let _ = svc.storage.del(&key).await;
            continue;
        };
        let platform = match platform_str {
            "telegram" => PlatformKind::Telegram,
            "discord" => PlatformKind::Discord,
            _ => {
                let _ = svc.storage.del(&key).await;
                continue;
            }
        };
        // Fire overdue ones after a small grace delay rather than instantly,
        // to avoid a burst at boot; cap the wait at the original window.
        let remaining = (fire_ts - now).clamp(2, 24 * 3600) as u64;
        arm_reminder(
            svc.clone(),
            Pending {
                id: id.to_owned(),
                secs: remaining,
                platform,
                chat_id: chat_id.to_owned(),
                user_id: user_id.to_owned(),
                note: note.to_owned(),
            },
        );
        restored += 1;
    }
    if restored > 0 {
        tracing::info!(count = restored, "restored pending reminders");
    }
}

// ---------- Calculator ------------------------------------------------------

/// `/calc 2*(3+4)-1` - a small recursive-descent evaluator over +, -, *,
/// /, % and parentheses.
pub(crate) async fn calc(ctx: Ctx, svc: Services) -> Result<()> {
    let expr = ctx.args().trim();
    if expr.is_empty() {
        let em = Embed::new()
            .title(svc.tr(&ctx, "calc_title").await)
            .description(svc.tr(&ctx, "calc_usage").await)
            .color(COLOR_WARN);
        return ctx.reply_with(Reply::embed(em)).await;
    }
    match eval_expr(expr) {
        Some(value) => {
            // Print integers cleanly, keep a few decimals otherwise.
            let shown = if (value.fract()).abs() < 1e-9 {
                format!("{}", value.round() as i64)
            } else {
                format!("{value:.6}")
                    .trim_end_matches('0')
                    .trim_end_matches('.')
                    .to_owned()
            };
            let em = Embed::new()
                .title(svc.tr(&ctx, "calc_title").await)
                .field(svc.tr(&ctx, "calc_expression").await, format!("`{expr}`"))
                .field(svc.tr(&ctx, "calc_result").await, format!("**{shown}**"))
                .color(COLOR_ACCENT);
            ctx.reply_with(Reply::embed(em)).await
        }
        None => {
            let em = Embed::new()
                .title(svc.tr(&ctx, "calc_title").await)
                .description(svc.tr(&ctx, "calc_bad_expr").await)
                .color(COLOR_WARN);
            ctx.reply_with(Reply::embed(em)).await
        }
    }
}

/// Evaluate an arithmetic expression, or `None` if it's malformed.
///
/// Length-capped: the parser recurses on `-`/`(`, so a kilometre of minus
/// signs would otherwise blow the stack.
fn eval_expr(input: &str) -> Option<f64> {
    if input.len() > 256 {
        return None;
    }
    let tokens: Vec<char> = input.chars().filter(|c| !c.is_whitespace()).collect();
    let mut pos = 0;
    let value = parse_add_sub(&tokens, &mut pos)?;
    if pos == tokens.len() {
        Some(value)
    } else {
        None
    }
}

fn parse_add_sub(t: &[char], pos: &mut usize) -> Option<f64> {
    let mut acc = parse_mul_div(t, pos)?;
    while let Some(&op) = t.get(*pos) {
        if op == '+' || op == '-' {
            *pos += 1;
            let rhs = parse_mul_div(t, pos)?;
            acc = if op == '+' { acc + rhs } else { acc - rhs };
        } else {
            break;
        }
    }
    Some(acc)
}

fn parse_mul_div(t: &[char], pos: &mut usize) -> Option<f64> {
    let mut acc = parse_unary(t, pos)?;
    while let Some(&op) = t.get(*pos) {
        if op == '*' || op == '/' || op == '%' {
            *pos += 1;
            let rhs = parse_unary(t, pos)?;
            match op {
                '*' => acc *= rhs,
                '/' => {
                    if rhs == 0.0 {
                        return None;
                    }
                    acc /= rhs;
                }
                _ => {
                    if rhs == 0.0 {
                        return None;
                    }
                    acc %= rhs;
                }
            }
        } else {
            break;
        }
    }
    Some(acc)
}

fn parse_unary(t: &[char], pos: &mut usize) -> Option<f64> {
    match t.get(*pos) {
        Some('-') => {
            *pos += 1;
            Some(-parse_unary(t, pos)?)
        }
        Some('+') => {
            *pos += 1;
            parse_unary(t, pos)
        }
        _ => parse_atom(t, pos),
    }
}

fn parse_atom(t: &[char], pos: &mut usize) -> Option<f64> {
    if t.get(*pos) == Some(&'(') {
        *pos += 1;
        let value = parse_add_sub(t, pos)?;
        if t.get(*pos) != Some(&')') {
            return None;
        }
        *pos += 1;
        return Some(value);
    }
    let start = *pos;
    while let Some(&c) = t.get(*pos) {
        if c.is_ascii_digit() || c == '.' {
            *pos += 1;
        } else {
            break;
        }
    }
    if *pos == start {
        return None;
    }
    t[start..*pos].iter().collect::<String>().parse().ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn calculator_basics() {
        assert_eq!(eval_expr("2+3"), Some(5.0));
        assert_eq!(eval_expr("2*(3+4)"), Some(14.0));
        assert_eq!(eval_expr("-5 + 2"), Some(-3.0));
        assert_eq!(eval_expr("10 % 3"), Some(1.0));
        assert_eq!(eval_expr("2 * 3 - 4 / 2"), Some(4.0));
    }

    #[test]
    fn calculator_rejects_bad_input() {
        assert_eq!(eval_expr(""), None);
        assert_eq!(eval_expr("1/0"), None); // guarded division
        assert_eq!(eval_expr("2+"), None);
        assert_eq!(eval_expr("(1+2"), None); // unbalanced parens
        assert_eq!(eval_expr("abc"), None);
    }
}
