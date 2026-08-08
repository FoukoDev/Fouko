//! The `/ai` command: manage private LLM hosts, chats and settings, and
//! talk to a model. Management is DM-only and owner-only; a chat message
//! only ever carries the user's own system prompt and history.
//!
//! Sub-commands (all under `/ai`):
//!   /ai                         - show your menu (chats, hosts)
//!   /ai host add <name> <url> [key]
//!   /ai host del <name>
//!   /ai host insecure <name> on|off  - accept a self-signed certificate
//!   /ai model add <host> <model>
//!   /ai model del <host> <model>
//!   /ai model tag <host> <model> <cap>  - mark what a model generates
//!   /ai model untag <host> <model>      - back to the name heuristic
//!   /ai model check <host> <model>      - probe a model for canned answers
//!   /ai models [host]           - list models and what they can do
//!   /ai gen [cap [host model|auto]] - pin which model draws/films/speaks
//!   /ai chat new <name> <host> <model>
//!   /ai chat del <name>
//!   /ai use <chat>              - pick the active chat
//!   /ai prompt <text>           - set the active chat's system prompt
//!   /ai clear                   - wipe the active chat's history
//!   /ai say <text>              - talk (also: plain text in a DM)
//!   /ai draw <prompt>           - generate an image (alias: img)
//!   /ai video <prompt>          - generate a video (alias: vid)
//!   /ai speak [voice:<name>] <text> - text to speech (alias: tts)
//!   /ai speak voices            - list known voice names
//!   /ai tools [on|off]          - let the model call draw/video/speak itself

use crate::ai::{
    cap_from_arg, find_chat_by_name, find_host_by_name, Chat, ChatMessage, ChatOutcome, GenError,
    Host, ModelCaps, PendingShare, ProbeReport, ProbeVerdict, Share, SharedHost, ToolCall,
    ToolSpec,
};
use crate::commands::Services;
use foukoapi::{Button, Ctx, Embed, Keyboard, PlatformKind, Reply, Result};
use std::sync::{Arc, Mutex};
use std::time::Duration;

const COLOR_ACCENT: u32 = 0x7A5BE8;
const COLOR_OK: u32 = 0x43B581;
const COLOR_WARN: u32 = 0xF59F00;
/// Red, for hard failures like a canned model.
const COLOR_ERR: u32 = 0xED4245;

/// The Mini App URL, when the operator runs the web app.
fn webapp_url() -> Option<String> {
    std::env::var("WEBAPP_URL").ok().filter(|s| !s.is_empty())
}

/// Resolve the caller's primary identity (so data follows an account link).
async fn primary(ctx: &Ctx, svc: &Services) -> String {
    svc.accounts
        .primary_for(ctx.platform(), ctx.user_id())
        .await
        .unwrap_or_else(|_| format!("{}:{}", ctx.platform(), ctx.user_id()))
}

/// Entry point wired into the bot as the `/ai` handler.
///
/// A storage/decrypt failure inside any sub-command surfaces here as an
/// error; tell the user to retry instead of failing silently. Mutations
/// abort before writing, so nothing is lost.
pub async fn ai(ctx: Ctx, svc: Services) -> Result<()> {
    if let Err(e) = ai_inner(&ctx, &svc).await {
        tracing::warn!(error = %e, "ai command failed");
        let _ = warn(&ctx, &svc, "ai_store_error").await;
    }
    Ok(())
}

async fn ai_inner(ctx: &Ctx, svc: &Services) -> Result<()> {
    let Some(store) = svc.ai.clone() else {
        let em = Embed::new()
            .title("\u{1F916} AI")
            .description(svc.tr(ctx, "ai_disabled").await)
            .color(COLOR_WARN);
        return ctx.reply_with(Reply::embed(em)).await;
    };

    // Family-access invitation buttons land here as callbacks. Only the
    // grantee (the person the invite was sent to) can press them - the
    // callback carries no other user's data and we act as the presser.
    if let Some(data) = ctx.callback_data() {
        if let Some(owner) = data.strip_prefix("ai:accept:") {
            return share_accept(ctx, svc, &store, owner).await;
        }
        if let Some(owner) = data.strip_prefix("ai:decline:") {
            return share_decline(ctx, svc, &store, owner).await;
        }
        // Menu buttons: pick the active chat, or clear history. These act on
        // the presser's own data (resolved by their primary), so there's no
        // cross-user risk - a button always operates on whoever tapped it.
        if let Some(chat_id) = data.strip_prefix("ai:use:") {
            let p = primary(ctx, svc).await;
            if store.chat(&p, chat_id).await.is_some() {
                store.set_active_chat(&p, chat_id).await?;
            }
            return show_menu(ctx, svc, &store).await;
        }
        if data == "ai:clear" {
            return clear_history(ctx, svc, &store).await;
        }
        if data == "ai:menu" {
            return show_menu(ctx, svc, &store).await;
        }
        if data == "ai:models" {
            return list_model_caps(ctx, svc, &store, "").await;
        }
        // Setup wizard buttons.
        if data == "ai:wiz:addhost" {
            let p = primary(ctx, svc).await;
            store.set_wizard_state(&p, "host:name").await?;
            return wizard_prompt(ctx, svc, "ai_wiz_host_name").await;
        }
        if data == "ai:wiz:addchat" {
            return wizard_pick_host(ctx, svc, &store).await;
        }
        if let Some(host_id) = data.strip_prefix("ai:wiz:chathost:") {
            return wizard_pick_model(ctx, svc, &store, host_id).await;
        }
        if let Some(rest) = data.strip_prefix("ai:wiz:chatmodel:") {
            // rest = <host_id>:<model index>
            let Some((host_id, idx)) = rest.rsplit_once(':') else {
                return Ok(());
            };
            return wizard_model_chosen(ctx, svc, &store, host_id, idx).await;
        }
        if data == "ai:wiz:cancel" {
            let p = primary(ctx, svc).await;
            store.clear_wizard(&p).await?;
            return show_menu(ctx, svc, &store).await;
        }
        // Management views (chat/host settings, generation pins). Like the
        // menu they belong to, they act on the presser's own data and only
        // ever live in a DM - refuse a stray press anywhere else.
        if data.starts_with("ai:cm:") || data.starts_with("ai:hm:") || data.starts_with("ai:gen") {
            if !ctx.is_dm() {
                let em = Embed::new()
                    .description(svc.tr(ctx, "ai_dm_only").await)
                    .color(COLOR_WARN);
                return ctx.reply_temporary(Reply::embed(em), 5).await;
            }
            if let Some(rest) = data.strip_prefix("ai:cm:") {
                return chat_manage_cb(ctx, svc, &store, rest).await;
            }
            if let Some(rest) = data.strip_prefix("ai:hm:") {
                return host_manage_cb(ctx, svc, &store, rest).await;
            }
            return gen_cb(ctx, svc, &store, data).await;
        }
        return Ok(());
    }

    let args = ctx.args().trim().to_owned();
    let mut parts = args.splitn(2, char::is_whitespace);
    let sub = parts.next().unwrap_or("").to_ascii_lowercase();
    let rest = parts.next().unwrap_or("").trim().to_owned();

    // Talking to the model is allowed anywhere; everything that manages
    // secrets or sharing is restricted to DMs below.
    match sub.as_str() {
        "" => {
            // The menu exposes your chats and hosts - keep it to DMs. In a
            // channel, just point the user to their DM.
            if !ctx.is_dm() {
                let em = Embed::new()
                    .description(svc.tr(ctx, "ai_open_in_dm").await)
                    .color(COLOR_ACCENT);
                return ctx.reply_with(Reply::embed(em)).await;
            }
            show_menu(ctx, svc, &store).await
        }
        "say" => talk(ctx, svc, &store, &rest).await,
        "draw" | "img" => draw(ctx, svc, &store, &rest).await,
        "video" | "vid" => video(ctx, svc, &store, &rest).await,
        "speak" | "tts" => speak(ctx, svc, &store, &rest).await,
        "tools" => tools_toggle(ctx, svc, &rest).await,
        "help" => ai_help(ctx, svc).await,
        "use" => use_chat(ctx, svc, &store, &rest).await,
        "clear" => clear_history(ctx, svc, &store).await,
        "host" | "model" | "models" | "caps" | "chat" | "prompt" | "gen" | "share" | "unshare"
        | "shared" => {
            if !ctx.is_dm() {
                let em = Embed::new()
                    .title(svc.tr(ctx, "ai_dm_only_title").await)
                    .description(svc.tr(ctx, "ai_dm_only").await)
                    .color(COLOR_WARN);
                return ctx.reply_with(Reply::embed(em)).await;
            }
            match sub.as_str() {
                "host" => manage_host(ctx, svc, &store, &rest).await,
                "model" => manage_model(ctx, svc, &store, &rest).await,
                "models" | "caps" => list_model_caps(ctx, svc, &store, &rest).await,
                "chat" => manage_chat(ctx, svc, &store, &rest).await,
                "prompt" => set_prompt(ctx, svc, &store, &rest).await,
                "gen" => gen_prefs(ctx, svc, &store, &rest).await,
                "share" => share_offer(ctx, svc, &store, &rest).await,
                "unshare" => share_revoke(ctx, svc, &store, &rest).await,
                "shared" => share_list(ctx, svc, &store).await,
                _ => unreachable!(),
            }
        }
        // Anything else is treated as a message to the model.
        _ => talk(ctx, svc, &store, &args).await,
    }
}

/// Plain text sent in a DM (no slash) routes here so a user can just chat.
/// Replying to one of the bot's messages counts as addressing it too, so
/// that works even in a group.
pub async fn ai_passive(ctx: Ctx, svc: Services) -> Result<()> {
    // Never treat a button press as chat input: on some platforms the
    // callback id also shows up as the message text, and we must not feed
    // that to the model. Also skip commands, and empty text - unless the
    // message carries a photo (caption-less photos are a valid turn).
    let has_image = ctx.has_incoming_image();
    if ctx.is_callback()
        || ctx.text().trim().starts_with('/')
        || (ctx.text().trim().is_empty() && !has_image)
    {
        return Ok(());
    }
    // In a DM every message goes to the model; elsewhere only replies to
    // the bot do.
    if !ctx.is_dm() && !ctx.is_reply_to_bot() {
        return Ok(());
    }
    let Some(store) = svc.ai.clone() else {
        return Ok(());
    };
    let p = primary(&ctx, &svc).await;
    let text = ctx.text().trim().to_owned();

    let outcome = async {
        // A running setup wizard eats the input first - one answer per step.
        if ctx.is_dm() && store.wizard_state(&p).await.is_some() {
            return wizard_step(&ctx, &svc, &store, &p, &text).await;
        }

        if store.active_chat(&p).await.is_none() {
            return Ok(());
        }
        // "draw me a cat" and friends route to the image model - but only
        // as a fallback for when the model can't decide itself: with tool
        // calling available the model picks generation on its own, so the
        // triggers step aside. They still cover hosts that rejected tools
        // and users who switched tools off. A photo caption never triggers
        // generation: the image goes to the vision chat model instead.
        let tools_available = crate::ai::tools::user_tools_enabled(&svc, &p).await
            && crate::ai::tools::active_host_tools_ok(&store, &p).await;
        if !tools_available {
            if !has_image
                && is_video_request(&text)
                && crate::ai::tools::resolve_capable_host(&store, &p, ModelCaps::VIDEO)
                    .await
                    .is_some()
            {
                return video(&ctx, &svc, &store, &text).await;
            }
            if !has_image
                && is_draw_request(&text)
                && crate::ai::tools::resolve_capable_host(&store, &p, ModelCaps::IMAGE)
                    .await
                    .is_some()
            {
                return draw(&ctx, &svc, &store, &text).await;
            }
        }
        talk(&ctx, &svc, &store, &text).await
    }
    .await;

    if let Err(e) = outcome {
        tracing::warn!(error = %e, "ai passive handler failed");
        let _ = warn(&ctx, &svc, "ai_store_error").await;
    }
    Ok(())
}

/// Full sub-command reference for `/ai`, with examples - including how to
/// set a system prompt, which isn't obvious otherwise.
async fn ai_help(ctx: &Ctx, svc: &Services) -> Result<()> {
    let lang = svc.lang(ctx).await;
    let (title, body) = match lang.as_str() {
        "ru" => (
            "\u{1F916} Помощь по ИИ",
            concat!(
                "**Настройка (в ЛС):**\n",
                "`/ai host add <имя> <url> [ключ]` - добавить хост (Ollama, LiteLLM, LM Studio, OpenRouter...). ",
                "Модели подтянутся с хоста сами\n",
                "`/ai host refresh <имя>` - перечитать список моделей\n",
                "`/ai host insecure <имя> on|off` - принимать самоподписанный сертификат хоста ",
                "(только для своих хостов!)\n",
                "`/ai model add <хост> <модель>` - добавить модель вручную\n",
                "`/ai model tag <хост> <модель> image|video|audio` - подсказать боту, что умеет модель ",
                "(если он не понял по имени); `/ai model untag` уберёт метки\n",
                "`/ai model check <хост> <модель>` - проверить, не отвечает ли модель заглушкой\n",
                "`/ai models [хост]` - список моделей и что они умеют\n",
                "`/ai gen` - какие модели закреплены за картинками/видео/озвучкой; ",
                "`/ai gen image <хост> <модель>` закрепит, `/ai gen image auto` вернёт автовыбор\n",
                "`/ai chat new <имя> <хост> <модель>` - создать чат\n",
                "`/ai prompt <текст>` - задать системный промпт активного чата\n",
                "`/ai use <чат>` - выбрать активный чат\n",
                "`/ai clear` - очистить историю активного чата\n\n",
                "**Общение:** просто пиши в ЛС, или `/ai say <текст>` где угодно\n",
                "Пришли фото (можно с подписью) - vision-модель (gpt-4o, llava, qwen-vl...) его опишет\n",
                "`/ai draw <промпт>` - нарисовать картинку (нужна модель типа DALL-E, SDXL, Flux); ",
                "в ЛС сработает и просто «нарисуй кота»\n",
                "`/ai video <промпт>` - сгенерировать видео (нужна модель типа Sora); ",
                "занимает пару минут\n",
                "`/ai speak [voice:<имя>] <текст>` - озвучить текст (нужна TTS-модель); ",
                "`/ai speak voices` - список голосов, последний выбранный голос запоминается\n",
                "`/ai tools on|off` - разрешить модели самой вызывать генерацию (по умолчанию включено)\n\n",
                "**Семейный доступ (в ЛС):**\n",
                "`/ai share <юзер> <хост> [модели]` - поделиться хостом\n",
                "`/ai unshare <юзер>` - отозвать доступ\n",
                "`/ai shared` - кому открыт доступ\n\n",
                "Всё хранится зашифрованным и синхронизируется через /link.",
            ),
        ),
        _ => (
            "\u{1F916} AI help",
            concat!(
                "**Setup (in DM):**\n",
                "`/ai host add <name> <url> [key]` - add a host (Ollama, LiteLLM, LM Studio, OpenRouter...). ",
                "Models are picked up from the host automatically\n",
                "`/ai host refresh <name>` - re-read the model list\n",
                "`/ai host insecure <name> on|off` - accept the host's self-signed certificate ",
                "(only for hosts you own!)\n",
                "`/ai model add <host> <model>` - add a model by hand\n",
                "`/ai model tag <host> <model> image|video|audio` - tell the bot what a model can do ",
                "(when the name gives nothing away); `/ai model untag` clears the tags\n",
                "`/ai model check <host> <model>` - probe whether a model gives canned answers\n",
                "`/ai models [host]` - list models and what they can do\n",
                "`/ai gen` - which models are pinned for image/video/speech; ",
                "`/ai gen image <host> <model>` pins one, `/ai gen image auto` goes back to auto-pick\n",
                "`/ai chat new <name> <host> <model>` - create a chat\n",
                "`/ai prompt <text>` - set the active chat's system prompt\n",
                "`/ai use <chat>` - pick the active chat\n",
                "`/ai clear` - clear the active chat's history\n\n",
                "**Chatting:** just type in DM, or `/ai say <text>` anywhere\n",
                "Send a photo (caption optional) - a vision model (gpt-4o, llava, qwen-vl...) will describe it\n",
                "`/ai draw <prompt>` - generate an image (needs a model like DALL-E, SDXL, Flux); ",
                "in DM a plain \"draw a cat\" works too\n",
                "`/ai video <prompt>` - generate a video (needs a model like Sora); ",
                "takes a couple of minutes\n",
                "`/ai speak [voice:<name>] <text>` - text to speech (needs a TTS model); ",
                "`/ai speak voices` - list voices, the last picked voice is remembered\n",
                "`/ai tools on|off` - let the model trigger generation itself (on by default)\n\n",
                "**Family access (in DM):**\n",
                "`/ai share <user> <host> [models]` - share a host\n",
                "`/ai unshare <user>` - revoke access\n",
                "`/ai shared` - see who you've shared with\n\n",
                "Everything is stored encrypted and follows your /link.",
            ),
        ),
    };
    // Mention the Mini App only when the operator actually runs it.
    let mut body = body.to_owned();
    if webapp_url().is_some() {
        body.push_str(match lang.as_str() {
            "ru" => {
                "\n\nВсё то же самое доступно в приложении - кнопка «Открыть приложение» в /ai."
            }
            _ => {
                "\n\nAll of this is also available in the app - the \"Open the app\" button in /ai."
            }
        });
    }
    let em = Embed::new()
        .title(title)
        .description(body)
        .color(COLOR_ACCENT);
    ctx.reply_with(Reply::embed(em)).await
}

async fn show_menu(ctx: &Ctx, svc: &Services, store: &crate::ai::AiStore) -> Result<()> {
    let p = primary(ctx, svc).await;
    let hosts = store.hosts(&p).await?;
    let chats = store.chats(&p).await?;
    let active = store.active_chat(&p).await;

    let lang = svc.lang(ctx).await;
    let (title, hosts_label, chats_label, active_label, hint) = match lang.as_str() {
        "ru" => (
            "\u{1F916} ИИ",
            "Хосты",
            "Чаты",
            "Активный",
            "Подробнее: /help",
        ),
        _ => ("\u{1F916} AI", "Hosts", "Chats", "Active", "More: /help"),
    };

    let host_lines = if hosts.is_empty() {
        "-".to_owned()
    } else {
        let mut lines = Vec::with_capacity(hosts.len());
        for h in &hosts {
            let count = svc
                .trf(ctx, "ai_models_count", &[&h.models.len().to_string()])
                .await;
            lines.push(format!("• {} {count}", h.name));
        }
        lines.join("\n")
    };
    let chat_lines = if chats.is_empty() {
        "-".to_owned()
    } else {
        chats
            .iter()
            .map(|c| {
                let mark = if Some(&c.id) == active.as_ref() {
                    " \u{2705}"
                } else {
                    ""
                };
                format!("• {} - {}{mark}", c.name, c.model)
            })
            .collect::<Vec<_>>()
            .join("\n")
    };
    let active_name = active
        .as_ref()
        .and_then(|id| chats.iter().find(|c| &c.id == id))
        .map(|c| c.name.clone())
        .unwrap_or_else(|| "-".to_owned());

    let em = Embed::new()
        .title(title)
        .field(hosts_label, host_lines)
        .field(chats_label, chat_lines)
        .field_inline(active_label, active_name)
        .footer(hint)
        .color(COLOR_ACCENT);

    // Pending family-access invites, surfaced with accept/decline buttons
    // right in the menu (so it works even if the DM notification didn't
    // arrive).
    let pending = store.pending(&p).await?;
    let mut kb = Keyboard::new();

    // Mini App entry point, when the operator runs the web app. First row
    // so it's the most visible thing in the menu. Telegram only: the app
    // authenticates through Telegram init data, so on Discord the button
    // would open a stub that can't sign anyone in.
    if ctx.platform() == foukoapi::PlatformKind::Telegram {
        if let Some(url) = webapp_url() {
            kb = kb.row([Button::web_app(svc.tr(ctx, "ai_open_app").await, url)]);
        }
    }

    // One row per chat: tap the name to switch, the gear to manage it
    // (change model, delete).
    for c in &chats {
        let mark = if Some(&c.id) == active.as_ref() {
            "\u{2705} "
        } else {
            ""
        };
        kb = kb.row([
            Button::callback(format!("{mark}{}", c.name), format!("ai:use:{}", c.id)),
            Button::callback("\u{2699}\u{FE0F}", format!("ai:cm:o:{}", c.id)),
        ]);
    }
    if active.is_some() {
        let clear_label = if lang == "ru" {
            "\u{1F9F9} Очистить историю"
        } else {
            "\u{1F9F9} Clear history"
        };
        kb = kb.row([Button::callback(clear_label, "ai:clear")]);
    }
    // One button per own host, opening its manage view (refresh models,
    // insecure toggle, delete). Shared hosts have no manage view - a
    // grantee can't touch the owner's settings.
    for chunk in hosts.chunks(2) {
        let row: Vec<Button> = chunk
            .iter()
            .map(|h| {
                Button::callback(
                    format!("\u{2699}\u{FE0F} {}", h.name),
                    format!("ai:hm:o:{}", h.id),
                )
            })
            .collect();
        kb = kb.row(row);
    }
    // Setup actions: add a host, and (once there's a host) a new chat plus
    // a model list.
    let (add_host_label, add_chat_label, models_label) = if lang == "ru" {
        ("\u{2795} Хост", "\u{2795} Чат", "\u{1F4CB} Модели")
    } else {
        ("\u{2795} Host", "\u{2795} Chat", "\u{1F4CB} Models")
    };
    if hosts.is_empty() {
        kb = kb.row([Button::callback(add_host_label, "ai:wiz:addhost")]);
    } else {
        kb = kb.row([
            Button::callback(add_host_label, "ai:wiz:addhost"),
            Button::callback(add_chat_label, "ai:wiz:addchat"),
            Button::callback(models_label, "ai:models"),
        ]);
        kb = kb.row([Button::callback(svc.tr(ctx, "ai_gen_btn").await, "ai:gen")]);
    }
    for pend in &pending {
        let label = if lang == "ru" {
            format!(
                "\u{1F46A} Принять доступ от {}",
                pretty_identity(&pend.owner)
            )
        } else {
            format!(
                "\u{1F46A} Accept access from {}",
                pretty_identity(&pend.owner)
            )
        };
        kb = kb.row([
            Button::callback(label, format!("ai:accept:{}", pend.owner)),
            Button::callback("\u{274C}", format!("ai:decline:{}", pend.owner)),
        ]);
    }

    if kb.is_empty() {
        ctx.reply_with(Reply::embed(em)).await
    } else if ctx.is_callback() {
        // Reuse the same message when this came from a button press.
        ctx.edit_reply(Reply::embed(em).keyboard(kb)).await
    } else {
        ctx.reply_with(Reply::embed(em).keyboard(kb)).await
    }
}

async fn manage_host(
    ctx: &Ctx,
    svc: &Services,
    store: &crate::ai::AiStore,
    rest: &str,
) -> Result<()> {
    let p = primary(ctx, svc).await;
    let mut it = rest.splitn(2, char::is_whitespace);
    let action = it.next().unwrap_or("").to_ascii_lowercase();
    let tail = it.next().unwrap_or("").trim();

    match action.as_str() {
        "add" => {
            // /ai host add <name> <url> [key]
            let mut f = tail.split_whitespace();
            let (Some(name), Some(url)) = (f.next(), f.next()) else {
                return warn(ctx, svc, "ai_host_add_usage").await;
            };
            // Same URL sanity check as the wizard.
            if !(url.starts_with("http://") || url.starts_with("https://")) {
                return warn(ctx, svc, "ai_wiz_bad_url").await;
            }
            let key = f.next().unwrap_or("");
            let had_key = !key.is_empty();
            if store.host(&p, name).await.is_some() {
                return warn(ctx, svc, "ai_host_exists").await;
            }
            // Same auto-discovery as the wizard: pull the model list from
            // the host when it exposes one. A failed discovery doesn't
            // block the add - the user can register models by hand.
            ctx.typing().await;
            let (found, discover_err) =
                crate::ai::add_host_discovering(store, &p, name, url, key).await?;
            if let Some(e) = discover_err {
                return host_added_no_discovery(ctx, svc, name, &e, had_key).await;
            }
            if found > 0 {
                let em = Embed::new()
                    .description(
                        svc.trf(ctx, "ai_host_added_models", &[&found.to_string()])
                            .await,
                    )
                    .color(COLOR_OK);
                return ctx.reply_with(Reply::embed(em)).await;
            }
            // The key is now stored encrypted, but the message the user
            // typed still sits in their chat history. We can't delete their
            // message, so nudge them to remove it themselves.
            if had_key {
                return ok(ctx, svc, "ai_host_added_key").await;
            }
            ok(ctx, svc, "ai_host_added").await
        }
        "refresh" => {
            // /ai host refresh <name> - re-pull the model list.
            if tail.is_empty() {
                return warn(ctx, svc, "ai_host_usage").await;
            }
            let Some(host) = find_host_by_name(store, &p, tail).await else {
                return warn(ctx, svc, "ai_host_missing").await;
            };
            ctx.typing().await;
            let found = match crate::ai::refresh_host_models(store, &p, &host).await? {
                Ok(0) => return warn(ctx, svc, "ai_refresh_none").await,
                Ok(n) => n,
                // Keep the stored list untouched and show why.
                Err(e) => return discover_failed(ctx, svc, &host.name, &e).await,
            };
            let em = Embed::new()
                .description(
                    svc.trf(ctx, "ai_host_added_models", &[&found.to_string()])
                        .await,
                )
                .color(COLOR_OK);
            ctx.reply_with(Reply::embed(em)).await
        }
        "del" => {
            if tail.is_empty() {
                return warn(ctx, svc, "ai_host_del_usage").await;
            }
            let Some(host) = find_host_by_name(store, &p, tail).await else {
                return warn(ctx, svc, "ai_host_missing").await;
            };
            store.remove_host(&p, &host.id).await?;
            ok(ctx, svc, "ai_host_removed").await
        }
        "insecure" => {
            // /ai host insecure <name> on|off - accept the host's
            // self-signed TLS certificate. Own hosts only.
            let mut f = tail.split_whitespace();
            let (Some(name), Some(mode)) = (f.next(), f.next()) else {
                return warn(ctx, svc, "ai_host_usage").await;
            };
            let on = match mode.to_ascii_lowercase().as_str() {
                "on" => true,
                "off" => false,
                _ => return warn(ctx, svc, "ai_host_usage").await,
            };
            if !crate::ai::set_host_insecure(store, &p, name, on).await? {
                return warn(ctx, svc, "ai_host_missing").await;
            }
            if on {
                // Not a plain "done": the user should understand the trade.
                let em = Embed::new()
                    .description(svc.tr(ctx, "ai_insecure_on").await)
                    .color(COLOR_WARN);
                return ctx.reply_with(Reply::embed(em)).await;
            }
            ok(ctx, svc, "ai_insecure_off").await
        }
        _ => warn(ctx, svc, "ai_host_usage").await,
    }
}

/// Host added but model discovery failed: say both - the host is in,
/// the model list isn't - plus the key-cleanup nudge when a key was
/// typed in the open.
async fn host_added_no_discovery(
    ctx: &Ctx,
    svc: &Services,
    host_name: &str,
    err: &str,
    had_key: bool,
) -> Result<()> {
    let mut body = svc.tr(ctx, "ai_host_added").await;
    body.push('\n');
    body.push_str(&discover_failed_body(ctx, svc, host_name, err).await);
    if had_key {
        body.push('\n');
        body.push_str(&svc.tr(ctx, "ai_host_added_key").await);
    }
    let em = Embed::new().description(body).color(COLOR_WARN);
    ctx.reply_with(Reply::embed(em)).await
}

/// Show a failed model discovery: the reason, and a cert hint when the
/// error smells like a self-signed certificate.
async fn discover_failed(ctx: &Ctx, svc: &Services, host_name: &str, err: &str) -> Result<()> {
    let body = discover_failed_body(ctx, svc, host_name, err).await;
    let em = Embed::new().description(body).color(COLOR_WARN);
    ctx.reply_with(Reply::embed(em)).await
}

/// The shared "discovery failed" text: reason plus the cert hint.
async fn discover_failed_body(ctx: &Ctx, svc: &Services, host_name: &str, err: &str) -> String {
    let mut body = svc.trf(ctx, "ai_discover_failed", &[err]).await;
    if is_cert_error(err) {
        body.push('\n');
        body.push_str(&svc.trf(ctx, "ai_insecure_hint", &[host_name]).await);
    }
    body
}

/// Does this error text look like a TLS certificate problem?
fn is_cert_error(err: &str) -> bool {
    let low = err.to_lowercase();
    low.contains("certificate") || low.contains("unknownissuer") || low.contains("self-signed")
}

async fn manage_model(
    ctx: &Ctx,
    svc: &Services,
    store: &crate::ai::AiStore,
    rest: &str,
) -> Result<()> {
    let p = primary(ctx, svc).await;
    let mut it = rest.splitn(2, char::is_whitespace);
    let action = it.next().unwrap_or("").to_ascii_lowercase();
    let tail = it.next().unwrap_or("").trim();

    // `tag` carries the capability as the last token; everything else is
    // just <host> <model>. Only own hosts here - a grantee can't retag
    // someone else's models.
    if action == "tag" {
        let mut f = tail.splitn(2, char::is_whitespace);
        let host_name = f.next().unwrap_or("");
        let rest2 = f.next().unwrap_or("").trim();
        let Some((model, cap_arg)) = rest2.rsplit_once(char::is_whitespace) else {
            return warn(ctx, svc, "ai_model_usage").await;
        };
        let model = model.trim();
        if host_name.is_empty() || model.is_empty() {
            return warn(ctx, svc, "ai_model_usage").await;
        }
        let Some(cap) = cap_from_arg(cap_arg) else {
            return warn(ctx, svc, "ai_cap_unknown").await;
        };
        return match crate::ai::tag_model(store, &p, host_name, model, cap).await? {
            crate::ai::ModelEdit::Done => ok(ctx, svc, "ai_tag_done").await,
            crate::ai::ModelEdit::HostMissing => warn(ctx, svc, "ai_host_missing").await,
            crate::ai::ModelEdit::ModelMissing => warn(ctx, svc, "ai_model_missing").await,
        };
    }

    // `check` probes the model for canned answers - it has its own flow
    // (network calls, cooldown), so route it out before the edit actions.
    if action == "check" {
        return check_model(ctx, svc, store, tail).await;
    }

    let mut f = tail.splitn(2, char::is_whitespace);
    let host_name = f.next().unwrap_or("");
    let model = f.next().unwrap_or("").trim();

    if host_name.is_empty() || model.is_empty() {
        return warn(ctx, svc, "ai_model_usage").await;
    }
    let outcome = match action.as_str() {
        "add" => crate::ai::add_model(store, &p, host_name, model).await?,
        "del" => crate::ai::del_model(store, &p, host_name, model).await?,
        "untag" => crate::ai::untag_model(store, &p, host_name, model).await?,
        _ => return warn(ctx, svc, "ai_model_usage").await,
    };
    match outcome {
        crate::ai::ModelEdit::Done => {
            let key = match action.as_str() {
                "add" => "ai_model_added",
                "del" => "ai_model_removed",
                _ => "ai_untag_done",
            };
            ok(ctx, svc, key).await
        }
        crate::ai::ModelEdit::HostMissing => warn(ctx, svc, "ai_host_missing").await,
        crate::ai::ModelEdit::ModelMissing => warn(ctx, svc, "ai_model_missing").await,
    }
}

// -- model liveness check ----------------------------------------------------
//
// Some proxies keep "answering" after the upstream model died, replaying
// a captured response. The framework probes catch that: tiny inputs that
// a live backend must react to. One probe per capability the model has.

/// How severe a verdict is, for merging multi-cap results. Higher wins.
fn verdict_rank(v: ProbeVerdict) -> u8 {
    match v {
        ProbeVerdict::Canned => 3,
        ProbeVerdict::Unstable => 2,
        ProbeVerdict::NotSupported => 1,
        ProbeVerdict::Live => 0,
    }
}

/// Run every applicable probe for a model and merge the reports: the
/// worst verdict wins, evidence is concatenated. `None` when the model
/// has no probeable capability (video-only).
async fn run_probes(host: &Host, model: &str) -> std::result::Result<Option<ProbeReport>, String> {
    let caps = host.caps_of(model);
    let client = crate::ai::client_for(host);
    let mut merged: Option<ProbeReport> = None;

    let mut absorb = |report: ProbeReport| match &mut merged {
        None => merged = Some(report),
        Some(m) => {
            if verdict_rank(report.verdict) > verdict_rank(m.verdict) {
                m.verdict = report.verdict;
            }
            m.evidence.extend(report.evidence);
        }
    };

    // Text is the implicit capability when no generation bit is set.
    if caps.is_text() {
        absorb(client.probe_chat(model).await.map_err(|e| e.to_string())?);
    }
    if caps.audio() {
        absorb(
            client
                .probe_speech(model)
                .await
                .map_err(|e| e.to_string())?,
        );
    }
    if caps.image() {
        absorb(
            client
                .probe_image_cheap(model)
                .await
                .map_err(|e| e.to_string())?,
        );
    }
    // Video has no cheap probe yet - a caps-only video model yields None.
    Ok(merged)
}

/// Render a merged probe report as an embed: localized verdict headline,
/// raw evidence lines below.
async fn probe_report_embed(ctx: &Ctx, svc: &Services, report: &ProbeReport) -> Embed {
    let (key, color) = match report.verdict {
        ProbeVerdict::Live => ("ai_check_live", COLOR_OK),
        ProbeVerdict::Canned => ("ai_check_canned", COLOR_ERR),
        ProbeVerdict::Unstable => ("ai_check_unstable", COLOR_WARN),
        ProbeVerdict::NotSupported => ("ai_check_notsupported", COLOR_WARN),
    };
    // Evidence lines stay English on purpose: they're tech diagnostics
    // from the framework ("same answer to different prompts"), not UI
    // copy, and translating them would only blur what the host did.
    let body = report
        .evidence
        .iter()
        .map(|e| format!("• {e}"))
        .collect::<Vec<_>>()
        .join("\n");
    Embed::new()
        .title(svc.tr(ctx, key).await)
        .description(body)
        .color(color)
}

/// `/ai model check <host> <model>` - probe a model for canned answers.
/// Own hosts only: probes spend the owner's tokens, so a grantee can't
/// run them against someone else's key.
async fn check_model(
    ctx: &Ctx,
    svc: &Services,
    store: &crate::ai::AiStore,
    tail: &str,
) -> Result<()> {
    let mut f = tail.splitn(2, char::is_whitespace);
    let host_name = f.next().unwrap_or("");
    let model = f.next().unwrap_or("").trim();
    if host_name.is_empty() || model.is_empty() {
        return warn(ctx, svc, "ai_check_usage").await;
    }
    let p = primary(ctx, svc).await;
    let Some(host) = find_host_by_name(store, &p, host_name).await else {
        return warn(ctx, svc, "ai_host_missing").await;
    };
    if !host.models.iter().any(|m| m == model) {
        return warn(ctx, svc, "ai_model_missing").await;
    }
    run_check(ctx, svc, &host, model).await
}

/// The shared probe flow behind the command and the host-manage button:
/// cooldown, typing, probes, report. Always a fresh message - the report
/// is long and shouldn't eat the view it was launched from.
async fn run_check(ctx: &Ctx, svc: &Services, host: &Host, model: &str) -> Result<()> {
    // Probes cost real tokens - a wider window than plain chat.
    let wait = svc
        .econ
        .cooldown_remaining(ctx.platform(), ctx.user_id(), "ai_check", 30)
        .await;
    if wait > 0 {
        return too_fast(ctx, svc).await;
    }
    svc.econ
        .touch_cooldown(ctx.platform(), ctx.user_id(), "ai_check")
        .await?;

    ctx.typing().await;

    let em = match run_probes(host, model).await {
        Ok(Some(report)) => probe_report_embed(ctx, svc, &report).await,
        // Video-only model: no cheap probe exists yet.
        Ok(None) => Embed::new()
            .description(svc.tr(ctx, "ai_check_no_video").await)
            .color(COLOR_WARN),
        Err(e) => Embed::new()
            .description(svc.trf(ctx, "ai_error", &[&e]).await)
            .color(COLOR_WARN),
    };
    let em = em.footer(format!("{} / {model}", host.name));
    ctx.reply_with(Reply::embed(em)).await
}

/// `/ai models [host]` - list every model the user can reach, tagged with
/// what it can produce (image/video/audio). Text models get a bare line.
async fn list_model_caps(
    ctx: &Ctx,
    svc: &Services,
    store: &crate::ai::AiStore,
    rest: &str,
) -> Result<()> {
    const MAX_LINES: usize = 40;

    let p = primary(ctx, svc).await;
    let filter = rest.split_whitespace().next().unwrap_or("");

    // Own hosts first, then family-shared ones (clipped to the models the
    // owner actually granted). The shared Host clone carries the owner's
    // caps map, so tags stay accurate there too.
    let mut groups: Vec<(String, Vec<String>)> = Vec::new();
    let mut has_video = false;
    let cap_image = svc.tr(ctx, "ai_cap_image").await;
    let cap_video = svc.tr(ctx, "ai_cap_video").await;
    let cap_audio = svc.tr(ctx, "ai_cap_audio").await;

    let mut line_of = |host: &Host, model: &str| -> String {
        let caps = host.caps_of(model);
        if caps.is_text() {
            return format!("`{model}`");
        }
        let mut tags = Vec::new();
        if caps.image() {
            tags.push(cap_image.as_str());
        }
        if caps.video() {
            tags.push(cap_video.as_str());
            has_video = true;
        }
        if caps.audio() {
            tags.push(cap_audio.as_str());
        }
        format!("`{model}` - {}", tags.join(", "))
    };

    for host in store.hosts(&p).await? {
        if !filter.is_empty() && host.name != filter {
            continue;
        }
        let lines: Vec<String> = host.models.iter().map(|m| line_of(&host, m)).collect();
        groups.push((host.name.clone(), lines));
    }
    for (_owner, host, allowed) in store.shared_hosts_for(&p).await {
        if !filter.is_empty() && host.name != filter {
            continue;
        }
        let lines: Vec<String> = allowed.iter().map(|m| line_of(&host, m)).collect();
        groups.push((format!("\u{1F46A} {}", host.name), lines));
    }

    if groups.iter().all(|(_, lines)| lines.is_empty()) {
        return warn(ctx, svc, "ai_models_none").await;
    }

    let mut body = String::new();
    let mut printed = 0usize;
    let mut skipped = 0usize;
    for (name, lines) in &groups {
        if lines.is_empty() {
            continue;
        }
        if printed >= MAX_LINES {
            skipped += lines.len();
            continue;
        }
        body.push_str(&format!("**{name}**\n"));
        for line in lines {
            if printed >= MAX_LINES {
                skipped += 1;
                continue;
            }
            body.push_str(line);
            body.push('\n');
            printed += 1;
        }
        body.push('\n');
    }
    if skipped > 0 {
        body.push_str(
            &svc.trf(ctx, "ai_models_more", &[&skipped.to_string()])
                .await,
        );
        body.push('\n');
    }
    if has_video {
        body.push('\n');
        body.push_str(&svc.tr(ctx, "ai_models_note_video").await);
    }

    let em = Embed::new()
        .title(svc.tr(ctx, "ai_models_title").await)
        .description(body.trim_end())
        .color(COLOR_ACCENT);
    if ctx.is_callback() {
        return ctx.edit_reply(Reply::embed(em)).await;
    }
    ctx.reply_with(Reply::embed(em)).await
}

async fn manage_chat(
    ctx: &Ctx,
    svc: &Services,
    store: &crate::ai::AiStore,
    rest: &str,
) -> Result<()> {
    let p = primary(ctx, svc).await;
    let mut it = rest.splitn(2, char::is_whitespace);
    let action = it.next().unwrap_or("").to_ascii_lowercase();
    let tail = it.next().unwrap_or("").trim();

    match action.as_str() {
        "new" => {
            // /ai chat new <name> <host> <model>
            let mut f = tail.split_whitespace();
            let (Some(name), Some(host_name), Some(model)) = (f.next(), f.next(), f.next()) else {
                return warn(ctx, svc, "ai_chat_new_usage").await;
            };
            // Look among the user's own hosts and any shared with them, so a
            // family-access grantee can build a chat on a shared host too.
            match crate::ai::create_chat(store, &p, name, host_name, model).await? {
                Ok(_) => ok(ctx, svc, "ai_chat_created").await,
                Err(crate::ai::ChatCreateError::HostMissing) => {
                    warn(ctx, svc, "ai_host_missing").await
                }
                Err(crate::ai::ChatCreateError::ModelMissing) => {
                    warn(ctx, svc, "ai_model_missing").await
                }
            }
        }
        "del" => {
            if tail.is_empty() {
                return warn(ctx, svc, "ai_chat_del_usage").await;
            }
            let Some(chat) = find_chat_by_name(store, &p, tail).await else {
                return warn(ctx, svc, "ai_chat_missing").await;
            };
            store.remove_chat(&p, &chat.id).await?;
            ok(ctx, svc, "ai_chat_removed").await
        }
        _ => warn(ctx, svc, "ai_chat_usage").await,
    }
}

async fn use_chat(ctx: &Ctx, svc: &Services, store: &crate::ai::AiStore, rest: &str) -> Result<()> {
    let p = primary(ctx, svc).await;
    if rest.is_empty() {
        return warn(ctx, svc, "ai_use_usage").await;
    }
    let Some(chat) = find_chat_by_name(store, &p, rest).await else {
        return warn(ctx, svc, "ai_chat_missing").await;
    };
    store.set_active_chat(&p, &chat.id).await?;
    ok(ctx, svc, "ai_chat_active").await
}

async fn set_prompt(
    ctx: &Ctx,
    svc: &Services,
    store: &crate::ai::AiStore,
    rest: &str,
) -> Result<()> {
    let p = primary(ctx, svc).await;
    let Some(active) = store.active_chat(&p).await else {
        return warn(ctx, svc, "ai_no_active").await;
    };
    let mut chats = store.chats(&p).await?;
    let Some(chat) = chats.iter_mut().find(|c| c.id == active) else {
        return warn(ctx, svc, "ai_no_active").await;
    };
    // The prompt rides along with every model call, so keep it bounded.
    chat.system_prompt = rest.chars().take(4_000).collect();
    store.set_chats(&p, &chats).await?;
    ok(ctx, svc, "ai_prompt_set").await
}

async fn clear_history(ctx: &Ctx, svc: &Services, store: &crate::ai::AiStore) -> Result<()> {
    let p = primary(ctx, svc).await;
    let Some(active) = store.active_chat(&p).await else {
        return warn(ctx, svc, "ai_no_active").await;
    };
    store.clear_history(&p, &active).await?;
    ok(ctx, svc, "ai_history_cleared").await
}

/// Send `text` to the active chat's model and reply with the answer. When
/// the message carries a photo and the chat's model can see images, the
/// photo rides along in this one request - it is never stored.
async fn talk(ctx: &Ctx, svc: &Services, store: &crate::ai::AiStore, text: &str) -> Result<()> {
    let has_image = ctx.has_incoming_image();
    let mut text = text.trim().to_owned();
    // A bare photo (no caption) is a valid turn - a localized default
    // prompt fills in below, after the vision checks.
    if text.is_empty() && !has_image {
        return warn(ctx, svc, "ai_say_usage").await;
    }
    // A single prompt can't be arbitrarily large: it goes into encrypted
    // history rows and straight to the model, so cap it.
    if text.chars().count() > 8_000 {
        return warn(ctx, svc, "ai_too_long").await;
    }
    let p = primary(ctx, svc).await;
    let Some(active) = store.active_chat(&p).await else {
        return warn(ctx, svc, "ai_no_active").await;
    };
    let Some(chat) = store.chat(&p, &active).await else {
        return warn(ctx, svc, "ai_no_active").await;
    };
    // A photo needs a vision-capable chat model. Refuse the whole turn
    // otherwise - sending just the caption would silently drop the image.
    if has_image && !crate::ai::model_sees_images(&chat.model) {
        return warn(ctx, svc, "ai_vision_no_model").await;
    }
    // Resolve the host among the user's own and any still-valid shared
    // ones. If a shared permission was changed/revoked, this returns None
    // and the chat simply stops working - no stale access.
    let Some(host) = store.usable_host(&p, &chat.host_id, &chat.model).await else {
        return warn(ctx, svc, "ai_host_revoked").await;
    };

    // Rate-limit model calls per user - they're slow and can cost money, so
    // one every few seconds is plenty and stops a flood of parallel calls.
    let wait = svc
        .econ
        .cooldown_remaining(ctx.platform(), ctx.user_id(), "ai_say", 5)
        .await;
    if wait > 0 {
        let em = Embed::new()
            .description(svc.tr(ctx, "ai_too_fast").await)
            .color(COLOR_WARN);
        return ctx.reply_temporary(Reply::embed(em), 5).await;
    }
    svc.econ
        .touch_cooldown(ctx.platform(), ctx.user_id(), "ai_say")
        .await?;

    // Download the photo only now, after every cheap check passed.
    let image = if has_image {
        match ctx.incoming_image().await {
            Ok(img) => img,
            Err(e) => {
                tracing::warn!(error = %e, "incoming image download failed");
                return warn(ctx, svc, "ai_vision_fetch_failed").await;
            }
        }
    } else {
        None
    };
    if text.is_empty() {
        text = svc.tr(ctx, "ai_vision_default_prompt").await;
    }
    // History keeps only the text, tagged so later turns still make sense.
    // The image itself is one-shot and never written anywhere.
    let hist_text = if image.is_some() {
        format!("{text} [image]")
    } else {
        text.clone()
    };

    // Build the message list: the user's own system prompt (if any) plus
    // their stored history plus the new turn. Nothing else is added.
    let mut messages: Vec<ChatMessage> = Vec::new();
    if !chat.system_prompt.is_empty() {
        messages.push(ChatMessage {
            role: "system".to_owned(),
            content: chat.system_prompt.clone(),
            ts: None,
        });
    }
    messages.extend(store.history(&p, &active).await?);
    messages.push(ChatMessage {
        role: "user".to_owned(),
        content: text.clone(),
        ts: None,
    });

    // Let the user know we're working - model replies can take a while.
    ctx.typing().await;

    // Offer generation tools when the user left them on and this host
    // hasn't already rejected them. With tools the model triggers image/
    // video/speech generation itself instead of relying on text triggers.
    let mut tools: Vec<ToolSpec> = if crate::ai::tools::user_tools_enabled(svc, &p).await
        && !crate::ai::tools::host_unsupported(&host.base_url)
    {
        crate::ai::tools::specs_for(store, &p).await
    } else {
        Vec::new()
    };

    // The tool loop: stream a round, execute whatever the model called,
    // feed the results back and stream again so it can comment. Capped at
    // MAX_TOOL_ROUNDS; the round after the cap goes out without tools to
    // force a plain-text finish. The image rides on the first request only.
    const MAX_TOOL_ROUNDS: usize = 3;
    // Wire-only tail: assistant call echoes and tool results. Never
    // persisted - the encrypted history keeps just the user turn and the
    // final answer.
    let mut extra: Vec<foukoapi::genai::ChatMessage> = Vec::new();
    let mut markers: Vec<&'static str> = Vec::new();
    let mut image = image;
    let mut round = 0usize;
    let answer = loop {
        let offered: &[ToolSpec] = if round < MAX_TOOL_ROUNDS { &tools } else { &[] };
        let img = image.take();
        let (result, partial) = stream_round(
            ctx,
            svc,
            &host,
            &chat.model,
            &messages,
            &extra,
            img.clone(),
            offered,
            true,
        )
        .await?;
        let outcome = match result {
            Ok(o) => o,
            Err(GenError::NotSupported) if !offered.is_empty() => {
                // The host choked on the tools field. Remember that and
                // redo this round plain, editing the same placeholder.
                crate::ai::tools::mark_host_unsupported(&host.base_url);
                tools.clear();
                let (retry, partial) = stream_round(
                    ctx,
                    svc,
                    &host,
                    &chat.model,
                    &messages,
                    &extra,
                    img,
                    &[],
                    false,
                )
                .await?;
                match retry {
                    Ok(o) => o,
                    Err(e) => {
                        return stream_error(ctx, svc, &host.name, &e.to_string(), &partial).await
                    }
                }
            }
            Err(e) => return stream_error(ctx, svc, &host.name, &e.to_string(), &partial).await,
        };

        // No calls (or the model ignored the forced-plain round): done.
        if outcome.tool_calls.is_empty() || round >= MAX_TOOL_ROUNDS {
            break outcome.text;
        }

        // Any streamed text is already on screen - finalize it. When the
        // model went straight for the tools, leave the placeholder as is:
        // execute_image/execute_video immediately edit it into their own
        // "working on it" notice, so touching it here would just flash.
        if !outcome.text.is_empty() {
            send_answer(ctx, &chat.name, &outcome.text).await?;
        }

        // Echo the calls (with the text, if any) so the follow-up request
        // is well-formed, then execute and report each one.
        let mut echo = foukoapi::genai::ChatMessage::assistant_tool_calls(&outcome.tool_calls);
        echo.content = outcome.text.clone();
        extra.push(echo);
        for call in &outcome.tool_calls {
            let (result_text, marker) = execute_tool_call(ctx, svc, store, &p, call).await;
            if let Some(m) = marker {
                if !markers.contains(&m) {
                    markers.push(m);
                }
            }
            extra.push(foukoapi::genai::ChatMessage::tool_result(
                call.id.clone(),
                result_text,
            ));
        }
        round += 1;
    };

    // Persist both turns so the conversation continues next time. Tool
    // traffic stays out; a generation leaves only a text marker, the same
    // way a one-shot photo leaves " [image]" on the user side.
    store
        .push_history(
            &p,
            &active,
            ChatMessage {
                role: "user".to_owned(),
                content: hist_text,
                ts: None,
            },
        )
        .await?;
    store
        .push_history(
            &p,
            &active,
            ChatMessage {
                role: "assistant".to_owned(),
                content: format!("{answer}{}", markers.concat()),
                ts: None,
            },
        )
        .await?;
    send_answer(ctx, &chat.name, &answer).await
}

/// One streamed request: post (or reuse) a "Thinking..." placeholder,
/// stream the model's text into it with a throttled updater, return the
/// outcome plus whatever partial text made it through. `fresh` picks
/// between a new placeholder message and editing the current one (used
/// when retrying the same round without tools).
#[allow(clippy::too_many_arguments)]
async fn stream_round(
    ctx: &Ctx,
    svc: &Services,
    host: &Host,
    model: &str,
    messages: &[ChatMessage],
    extra: &[foukoapi::genai::ChatMessage],
    image: Option<Vec<u8>>,
    tools: &[ToolSpec],
    fresh: bool,
) -> Result<(std::result::Result<ChatOutcome, GenError>, String)> {
    let thinking = Embed::new()
        .description(svc.tr(ctx, "ai_thinking").await)
        .color(COLOR_ACCENT);
    if fresh {
        ctx.reply_with(Reply::embed(thinking)).await?;
    } else {
        ctx.edit_reply(Reply::embed(thinking)).await?;
    }

    // Accumulated text shared between the SSE callback and the updater.
    let acc: Arc<Mutex<String>> = Arc::new(Mutex::new(String::new()));

    // Background updater: every 2s take a snapshot and, if it changed and
    // is non-empty, edit the placeholder with it. 2s throttle respects
    // platform edit rate limits; a model that finishes before the first
    // tick gets only the final edit, no flicker. Stopped via the watch
    // channel and awaited before the final edit, so a stale in-flight
    // edit can never overwrite the finished answer.
    let (stop_tx, mut stop_rx) = tokio::sync::watch::channel(false);
    let updater = {
        let ctx2 = ctx.clone();
        let acc2 = Arc::clone(&acc);
        tokio::spawn(async move {
            let mut last_sent = String::new();
            loop {
                tokio::select! {
                    _ = tokio::time::sleep(Duration::from_secs(2)) => {}
                    _ = stop_rx.changed() => break,
                }
                let snapshot = acc2.lock().map(|s| s.clone()).unwrap_or_default();
                if snapshot.is_empty() || snapshot == last_sent {
                    continue;
                }
                // Edits don't split into chunks - clip long partials. A
                // failed edit (rate limit, half-open markdown the platform
                // rejects) is retried on the next tick, not fatal.
                if ctx2
                    .edit_reply(Reply::text(clip_stream_text(&snapshot)))
                    .await
                    .is_ok()
                {
                    last_sent = snapshot;
                }
            }
        })
    };

    let result = {
        let acc2 = Arc::clone(&acc);
        crate::ai::chat_completion_stream_tools(
            host,
            model,
            messages,
            extra,
            image,
            tools,
            move |sofar| {
                // on_delta hands us the accumulated text, not an increment.
                if let Ok(mut s) = acc2.lock() {
                    s.clear();
                    s.push_str(sofar);
                }
            },
        )
        .await
    };

    // Stop the updater and wait it out before touching the message again.
    let _ = stop_tx.send(true);
    let _ = updater.await;

    let partial = acc.lock().map(|s| s.clone()).unwrap_or_default();
    Ok((result, partial))
}

/// A stream broke: show the error, keeping any partial text the user
/// already saw. Nothing is persisted - a truncated answer is not a turn.
/// A cert-looking error gets the same insecure hint as discovery.
async fn stream_error(
    ctx: &Ctx,
    svc: &Services,
    host_name: &str,
    err: &str,
    partial: &str,
) -> Result<()> {
    let hint = if is_cert_error(err) {
        let h = svc.trf(ctx, "ai_insecure_hint", &[host_name]).await;
        format!("\n{h}")
    } else {
        String::new()
    };
    if partial.is_empty() {
        // Nothing streamed: replace the placeholder with the error.
        let body = svc.trf(ctx, "ai_error", &[err]).await;
        let em = Embed::new()
            .description(format!("{body}{hint}"))
            .color(COLOR_WARN);
        return ctx.edit_reply(Reply::embed(em)).await;
    }
    let tail = svc.trf(ctx, "ai_stream_broken", &[err]).await;
    let text = format!("{}\n\n{tail}{hint}", clip_stream_text(partial));
    ctx.edit_reply(Reply::text(text)).await
}

/// Longest text we put into a single message edit. Edits can't be split
/// into chunks like fresh replies, so partials get clipped to this.
const STREAM_EDIT_LIMIT: usize = 3500;

/// Clip a streaming partial to the edit-safe length, with a "..." tail.
fn clip_stream_text(text: &str) -> String {
    if text.chars().count() <= STREAM_EDIT_LIMIT {
        return text.to_owned();
    }
    let clipped: String = text.chars().take(STREAM_EDIT_LIMIT).collect();
    format!("{clipped}...")
}

/// Deliver the final answer: edit the streaming placeholder with the
/// first chunk, then send the rest as ordinary replies so nothing gets
/// clipped at the platform's length limit. No decoration - just the
/// model's text.
async fn send_answer(ctx: &Ctx, _chat_name: &str, answer: &str) -> Result<()> {
    let answer = answer.trim();
    if answer.is_empty() {
        return ctx.edit_reply(Reply::text("…")).await;
    }
    // The edited part stays under the edit-safe cap; overflow goes out
    // as regular chunked replies (Telegram tops out near 4096 chars).
    let mut chunks = foukoapi::util::split_chunks(answer, STREAM_EDIT_LIMIT).into_iter();
    if let Some(first) = chunks.next() {
        ctx.edit_reply(Reply::text(first)).await?;
    }
    for part in chunks {
        ctx.reply(&part).await?;
    }
    Ok(())
}

// -- image generation ----------------------------------------------------

/// Phrases that turn a plain DM message into an image request. Checked
/// case-insensitively against the start of the text.
const DRAW_TRIGGERS: &[&str] = &[
    // en
    "draw ",
    "draw me ",
    "generate an image",
    "make an image",
    "paint ",
    // ru
    "нарисуй ",
    "сгенерируй картинку",
    "сгенерируй изображение",
    // uk
    "намалюй ",
    "згенеруй картинку",
    // de
    "zeichne ",
    "male ",
    "erstelle ein bild",
    // es
    "dibuja ",
    "dibújame ",
    "genera una imagen",
];

/// Does this message look like a request to draw something?
fn is_draw_request(text: &str) -> bool {
    let low = text.trim_start().to_lowercase();
    DRAW_TRIGGERS.iter().any(|t| low.starts_with(t))
}

/// Phrases that turn a plain DM message into a video request. Same
/// mechanism as [`DRAW_TRIGGERS`]. Speech has no triggers on purpose:
/// "say ..." would misfire on ordinary chat constantly.
const VIDEO_TRIGGERS: &[&str] = &[
    // en
    "make a video",
    "generate a video",
    "create a video",
    // ru
    "сгенерируй видео",
    "сними видео",
    "сделай видео",
    // uk
    "згенеруй відео",
    "зроби відео",
    // de
    "erstelle ein video",
    "mach ein video",
    // es
    "genera un video",
    "haz un video",
];

/// Does this message look like a request to make a video?
fn is_video_request(text: &str) -> bool {
    let low = text.trim_start().to_lowercase();
    VIDEO_TRIGGERS.iter().any(|t| low.starts_with(t))
}

/// Why a generation didn't happen. Shared by the slash commands and the
/// tool path, which report the same conditions their own way.
enum ExecError {
    /// The per-user cooldown hasn't elapsed yet.
    RateLimited,
    /// No reachable model with the needed capability.
    NoModel,
    /// The host lacks the endpoint.
    NotSupported,
    /// Generation or delivery failed.
    Failed(String),
}

impl ExecError {
    /// The short line fed back to the model as a tool result.
    fn tool_text(&self) -> String {
        match self {
            Self::RateLimited => "rate limited, ask the user to wait".to_owned(),
            Self::NoModel => "failed: no capable model available".to_owned(),
            Self::NotSupported => "failed: the host does not support this".to_owned(),
            Self::Failed(e) => {
                let short: String = e.chars().take(200).collect();
                format!("failed: {short}")
            }
        }
    }
}

/// Check-and-touch one generation cooldown. The same keys guard the
/// commands and the tool path, so the model can't route around them.
async fn gen_cooldown(
    ctx: &Ctx,
    svc: &Services,
    key: &str,
    secs: i64,
) -> std::result::Result<(), ExecError> {
    let wait = svc
        .econ
        .cooldown_remaining(ctx.platform(), ctx.user_id(), key, secs)
        .await;
    if wait > 0 {
        return Err(ExecError::RateLimited);
    }
    svc.econ
        .touch_cooldown(ctx.platform(), ctx.user_id(), key)
        .await
        .map_err(|e| ExecError::Failed(e.to_string()))
}

/// The image pipeline both `/ai draw` and the `generate_image` tool run:
/// resolve a capable host, rate-limit, generate, send the photo as its
/// own message.
async fn execute_image(
    ctx: &Ctx,
    svc: &Services,
    store: &crate::ai::AiStore,
    p: &str,
    prompt: &str,
) -> std::result::Result<(), ExecError> {
    let Some((host, model)) =
        crate::ai::tools::resolve_capable_host(store, p, ModelCaps::IMAGE).await
    else {
        return Err(ExecError::NoModel);
    };
    // A wider window than chat: image generation is slow and pricey.
    gen_cooldown(ctx, svc, "ai_draw", 30).await?;

    // Say what's happening - generation runs 10-60s and a silent bot
    // reads as a hang. Edit-in-place: in the tool path this recycles the
    // streaming placeholder instead of stacking a second notice message;
    // for /ai draw there is nothing to edit yet, so it just sends.
    let em = Embed::new()
        .description(svc.trf(ctx, "ai_draw_working", &[&model]).await)
        .color(COLOR_ACCENT);
    let _ = ctx.edit_reply(Reply::embed(em)).await;
    ctx.typing().await;

    match crate::ai::image_generation(&host, &model, prompt).await {
        Ok(bytes) => {
            // Caption is the prompt, clipped so it never trips a platform
            // caption limit.
            let caption: String = prompt.chars().take(200).collect();
            let reply = Reply::text(caption).image_bytes(bytes, "image.png");
            // Oversized images error out at send time - surface that
            // instead of a generic store error.
            ctx.reply_with(reply)
                .await
                .map_err(|e| ExecError::Failed(e.to_string()))
        }
        Err(GenError::NotSupported) => Err(ExecError::NotSupported),
        Err(GenError::Other(e)) => Err(ExecError::Failed(e)),
    }
}

/// The video pipeline shared by `/ai video` and the `generate_video`
/// tool. Posts the "working on it" notice up front - rendering takes
/// minutes and must not go silent.
async fn execute_video(
    ctx: &Ctx,
    svc: &Services,
    store: &crate::ai::AiStore,
    p: &str,
    prompt: &str,
) -> std::result::Result<(), ExecError> {
    let Some((host, model)) =
        crate::ai::tools::resolve_capable_host(store, p, ModelCaps::VIDEO).await
    else {
        return Err(ExecError::NoModel);
    };
    // Video is by far the priciest and slowest thing here - one every two
    // minutes is plenty.
    gen_cooldown(ctx, svc, "ai_video", 120).await?;

    let em = Embed::new()
        .description(svc.tr(ctx, "ai_video_working").await)
        .color(COLOR_ACCENT);
    // Edit-in-place recycles the tool path's placeholder, see execute_image.
    ctx.edit_reply(Reply::embed(em))
        .await
        .map_err(|e| ExecError::Failed(e.to_string()))?;
    ctx.typing().await;

    match crate::ai::video_generation(&host, &model, prompt).await {
        Ok(bytes) => {
            let caption: String = prompt.chars().take(200).collect();
            let reply = Reply::text(caption).video_bytes(bytes, "video.mp4");
            // Discord rejects files over 8 MiB at send time - surface that
            // instead of a generic store error.
            ctx.reply_with(reply)
                .await
                .map_err(|e| ExecError::Failed(e.to_string()))
        }
        Err(GenError::NotSupported) => Err(ExecError::NotSupported),
        Err(GenError::Other(e)) => Err(ExecError::Failed(e)),
    }
}

/// The speech pipeline shared by `/ai speak` and the `speak` tool. With
/// no explicit voice the user's remembered pick applies, then the server
/// default.
async fn execute_speech(
    ctx: &Ctx,
    svc: &Services,
    store: &crate::ai::AiStore,
    p: &str,
    text: &str,
    voice: Option<&str>,
) -> std::result::Result<(), ExecError> {
    let Some((host, model)) =
        crate::ai::tools::resolve_capable_host(store, p, ModelCaps::AUDIO).await
    else {
        return Err(ExecError::NoModel);
    };
    gen_cooldown(ctx, svc, "ai_speak", 30).await?;

    // Same edit-in-place trick as execute_image: recycle the tool path's
    // placeholder into a short status so nothing dangles.
    let em = Embed::new()
        .description(svc.trf(ctx, "ai_speak_working", &[&model]).await)
        .color(COLOR_ACCENT);
    let _ = ctx.edit_reply(Reply::embed(em)).await;
    ctx.typing().await;

    match crate::ai::speech_generation(&host, &model, text, voice).await {
        Ok(bytes) => {
            let reply = Reply::text("").audio_bytes(bytes, "speech.mp3");
            ctx.reply_with(reply)
                .await
                .map_err(|e| ExecError::Failed(e.to_string()))
        }
        Err(GenError::NotSupported) => Err(ExecError::NotSupported),
        Err(GenError::Other(e)) => Err(ExecError::Failed(e)),
    }
}

/// The voice a user last picked explicitly, if any.
async fn stored_voice(svc: &Services, p: &str) -> Option<String> {
    crate::ai::stored_voice(&svc.storage, p).await
}

/// Run one model-issued tool call. Returns the line fed back to the model
/// and, on success, the marker appended to the assistant turn in history.
async fn execute_tool_call(
    ctx: &Ctx,
    svc: &Services,
    store: &crate::ai::AiStore,
    p: &str,
    call: &ToolCall,
) -> (String, Option<&'static str>) {
    let Ok(args) = serde_json::from_str::<serde_json::Value>(&call.arguments) else {
        return ("invalid arguments".to_owned(), None);
    };
    // A required string field, trimmed and clipped like the command input.
    let field = |name: &str| -> Option<String> {
        args.get(name)
            .and_then(|v| v.as_str())
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(|s| s.chars().take(2_000).collect())
    };
    match call.name.as_str() {
        "generate_image" => {
            let Some(prompt) = field("prompt") else {
                return ("invalid arguments".to_owned(), None);
            };
            match execute_image(ctx, svc, store, p, &prompt).await {
                Ok(()) => (
                    "image generated and sent to the user".to_owned(),
                    Some(" [generated image]"),
                ),
                Err(e) => (e.tool_text(), None),
            }
        }
        "generate_video" => {
            let Some(prompt) = field("prompt") else {
                return ("invalid arguments".to_owned(), None);
            };
            match execute_video(ctx, svc, store, p, &prompt).await {
                Ok(()) => (
                    "video generated and sent to the user".to_owned(),
                    Some(" [generated video]"),
                ),
                Err(e) => (e.tool_text(), None),
            }
        }
        "speak" => {
            let Some(text) = field("text") else {
                return ("invalid arguments".to_owned(), None);
            };
            let voice = match field("voice") {
                Some(v) => Some(v.to_ascii_lowercase()),
                None => stored_voice(svc, p).await,
            };
            match execute_speech(ctx, svc, store, p, &text, voice.as_deref()).await {
                Ok(()) => (
                    "audio generated and sent to the user".to_owned(),
                    Some(" [generated audio]"),
                ),
                Err(e) => (e.tool_text(), None),
            }
        }
        _ => ("unknown tool".to_owned(), None),
    }
}

/// A generation command hit the cooldown: tell the user, briefly.
async fn too_fast(ctx: &Ctx, svc: &Services) -> Result<()> {
    let em = Embed::new()
        .description(svc.tr(ctx, "ai_too_fast").await)
        .color(COLOR_WARN);
    ctx.reply_temporary(Reply::embed(em), 5).await
}

/// A generation command failed: show the localized reason.
async fn exec_failed(ctx: &Ctx, svc: &Services, key: &str, err: &str) -> Result<()> {
    let em = Embed::new()
        .description(svc.trf(ctx, key, &[err]).await)
        .color(COLOR_WARN);
    ctx.reply_with(Reply::embed(em)).await
}

/// `/ai draw <prompt>` - generate an image and send it as a photo. The
/// prompt is not written to chat history: it's not a dialogue turn.
async fn draw(ctx: &Ctx, svc: &Services, store: &crate::ai::AiStore, prompt: &str) -> Result<()> {
    let prompt = prompt.trim();
    if prompt.is_empty() {
        return warn(ctx, svc, "ai_draw_usage").await;
    }
    if prompt.chars().count() > 2_000 {
        return warn(ctx, svc, "ai_too_long").await;
    }
    let p = primary(ctx, svc).await;
    match execute_image(ctx, svc, store, &p, prompt).await {
        Ok(()) => Ok(()),
        Err(ExecError::NoModel) => warn(ctx, svc, "ai_draw_no_model").await,
        Err(ExecError::RateLimited) => too_fast(ctx, svc).await,
        Err(ExecError::NotSupported) => warn(ctx, svc, "ai_draw_not_supported").await,
        Err(ExecError::Failed(e)) => exec_failed(ctx, svc, "ai_draw_failed", &e).await,
    }
}

/// `/ai video <prompt>` - generate a short video and send it natively.
/// Like `draw`, the prompt never touches chat history.
async fn video(ctx: &Ctx, svc: &Services, store: &crate::ai::AiStore, prompt: &str) -> Result<()> {
    let prompt = prompt.trim();
    if prompt.is_empty() {
        return warn(ctx, svc, "ai_video_usage").await;
    }
    if prompt.chars().count() > 2_000 {
        return warn(ctx, svc, "ai_too_long").await;
    }
    let p = primary(ctx, svc).await;
    match execute_video(ctx, svc, store, &p, prompt).await {
        Ok(()) => Ok(()),
        Err(ExecError::NoModel) => warn(ctx, svc, "ai_video_no_model").await,
        Err(ExecError::RateLimited) => too_fast(ctx, svc).await,
        Err(ExecError::NotSupported) => warn(ctx, svc, "ai_video_not_supported").await,
        Err(ExecError::Failed(e)) => exec_failed(ctx, svc, "ai_video_failed", &e).await,
    }
}

/// `/ai speak [voice:<name>] <text>` - synthesize speech and send it as an
/// audio file. The text never touches chat history. `voice:` as the first
/// token picks a voice; `/ai speak voices` lists the known ones. The last
/// used voice is remembered per user and becomes the default.
async fn speak(ctx: &Ctx, svc: &Services, store: &crate::ai::AiStore, text: &str) -> Result<()> {
    let text = text.trim();
    if text.eq_ignore_ascii_case("voices") {
        return speak_voices(ctx, svc).await;
    }
    let p = primary(ctx, svc).await;
    // First token "voice:<name>" selects a voice; the rest is the text.
    // Names are lowercased (OpenAI voices are all lowercase) but unknown
    // ones pass through - proxies may serve their own voices.
    let (explicit_voice, text) = match text.split_once(char::is_whitespace) {
        Some((head, rest)) if head.to_ascii_lowercase().starts_with("voice:") => {
            (Some(head[6..].to_ascii_lowercase()), rest.trim())
        }
        None if text.to_ascii_lowercase().starts_with("voice:") => {
            // A bare "voice:x" with no text to speak.
            (Some(text[6..].to_ascii_lowercase()), "")
        }
        _ => (None, text),
    };
    let explicit_voice = explicit_voice.filter(|v| !v.is_empty());
    if text.is_empty() {
        return warn(ctx, svc, "ai_speak_usage").await;
    }
    // Fall back to the last voice the user picked, then the server default.
    let voice = match &explicit_voice {
        Some(v) => Some(v.clone()),
        None => stored_voice(svc, &p).await,
    };
    match execute_speech(ctx, svc, store, &p, text, voice.as_deref()).await {
        Ok(()) => {
            // Remember an explicitly chosen voice only after it worked.
            if let Some(v) = &explicit_voice {
                let _ = crate::ai::set_stored_voice(&svc.storage, &p, v).await;
            }
            Ok(())
        }
        Err(ExecError::NoModel) => warn(ctx, svc, "ai_speak_no_model").await,
        Err(ExecError::RateLimited) => too_fast(ctx, svc).await,
        Err(ExecError::NotSupported) => warn(ctx, svc, "ai_speak_not_supported").await,
        Err(ExecError::Failed(e)) => {
            let mut body = svc.trf(ctx, "ai_speak_failed", &[&e]).await;
            // A server error with an unknown voice: hint at the voice list.
            if voice
                .as_deref()
                .is_some_and(|v| !crate::ai::KNOWN_VOICES.contains(&v))
            {
                body.push('\n');
                body.push_str(&svc.tr(ctx, "ai_speak_voice_hint").await);
            }
            let em = Embed::new().description(body).color(COLOR_WARN);
            ctx.reply_with(Reply::embed(em)).await
        }
    }
}

/// `/ai speak voices` - list the voice names OpenAI-compatible hosts
/// usually understand. Proxies may accept others too.
async fn speak_voices(ctx: &Ctx, svc: &Services) -> Result<()> {
    let body = crate::ai::KNOWN_VOICES
        .iter()
        .map(|v| format!("`{v}`"))
        .collect::<Vec<_>>()
        .join(", ");
    let em = Embed::new()
        .title(svc.tr(ctx, "ai_speak_voices_title").await)
        .description(body)
        .color(COLOR_ACCENT);
    ctx.reply_with(Reply::embed(em)).await
}

/// `/ai tools [on|off]` - the per-user switch for model-driven generation.
/// No argument shows the current state.
async fn tools_toggle(ctx: &Ctx, svc: &Services, rest: &str) -> Result<()> {
    let p = primary(ctx, svc).await;
    match rest.trim().to_ascii_lowercase().as_str() {
        "on" => {
            crate::ai::tools::set_user_tools(svc, &p, true).await?;
            ok(ctx, svc, "ai_tools_on").await
        }
        "off" => {
            crate::ai::tools::set_user_tools(svc, &p, false).await?;
            ok(ctx, svc, "ai_tools_off").await
        }
        // No argument (or anything unrecognized): show the current state.
        _ => {
            let key = if crate::ai::tools::user_tools_enabled(svc, &p).await {
                "ai_tools_status_on"
            } else {
                "ai_tools_status_off"
            };
            let em = Embed::new()
                .description(svc.tr(ctx, key).await)
                .color(COLOR_ACCENT);
            ctx.reply_with(Reply::embed(em)).await
        }
    }
}

// -- helpers -----------------------------------------------------------------

// -- pinned generation models ----------------------------------------------

/// The localized display name of a capability.
async fn cap_label(ctx: &Ctx, svc: &Services, cap: u8) -> String {
    let key = match cap {
        ModelCaps::IMAGE => "ai_cap_image",
        ModelCaps::VIDEO => "ai_cap_video",
        _ => "ai_cap_audio",
    };
    svc.tr(ctx, key).await
}

/// `/ai gen` - pin which model handles each generation capability.
/// No arguments shows the overview; `<cap> <host> <model>` pins,
/// `<cap> auto` unpins.
async fn gen_prefs(
    ctx: &Ctx,
    svc: &Services,
    store: &crate::ai::AiStore,
    rest: &str,
) -> Result<()> {
    let p = primary(ctx, svc).await;
    if rest.is_empty() {
        return gen_overview(ctx, svc, store, &p).await;
    }
    let mut f = rest.splitn(2, char::is_whitespace);
    let cap_arg = f.next().unwrap_or("");
    let tail = f.next().unwrap_or("").trim();
    let Some(cap) = cap_from_arg(cap_arg) else {
        return warn(ctx, svc, "ai_cap_unknown").await;
    };
    if tail.eq_ignore_ascii_case("auto") {
        store.clear_gen_pref(&p, cap).await?;
        return ok(ctx, svc, "ai_gen_reset").await;
    }
    let Some((host_name, model)) = tail.split_once(char::is_whitespace) else {
        return warn(ctx, svc, "ai_gen_usage").await;
    };
    let model = model.trim();
    // The pick must be real right now: a host the user can reach, with
    // that model on it, able to produce this kind of output.
    if crate::ai::tools::pinned_candidate(store, &p, cap, host_name, model)
        .await
        .is_none()
    {
        return warn(ctx, svc, "ai_gen_bad").await;
    }
    store.set_gen_pref(&p, cap, host_name, model).await?;
    let label = cap_label(ctx, svc, cap).await;
    let em = Embed::new()
        .title("\u{2705}")
        .description(
            svc.trf(
                ctx,
                "ai_gen_set",
                &[&format!("{host_name}/{model}"), &label],
            )
            .await,
        )
        .color(COLOR_OK);
    ctx.reply_with(Reply::embed(em)).await
}

/// The `/ai gen` overview: one line per capability - the pinned model,
/// or what auto-pick would use right now, or "no model".
async fn gen_overview(
    ctx: &Ctx,
    svc: &Services,
    store: &crate::ai::AiStore,
    p: &str,
) -> Result<()> {
    let body = gen_overview_body(ctx, svc, store, p).await;
    let em = Embed::new()
        .title(svc.tr(ctx, "ai_gen_title").await)
        .description(body.trim_end())
        .color(COLOR_ACCENT);
    ctx.reply_with(Reply::embed(em)).await
}

/// The overview text shared by `/ai gen` and the button view.
async fn gen_overview_body(
    ctx: &Ctx,
    svc: &Services,
    store: &crate::ai::AiStore,
    p: &str,
) -> String {
    let auto_none = svc.tr(ctx, "ai_gen_none").await;
    let mut body = String::new();
    for cap in [ModelCaps::IMAGE, ModelCaps::VIDEO, ModelCaps::AUDIO] {
        let label = cap_label(ctx, svc, cap).await;
        // A pin only counts while it still resolves; a dead one shows as
        // whatever auto-pick falls back to, matching what generation does.
        let pinned = match store.gen_pref(p, cap).await {
            Some((host, model)) => crate::ai::tools::pinned_candidate(store, p, cap, &host, &model)
                .await
                .map(|_| format!("{host}/{model}")),
            None => None,
        };
        let line = match pinned {
            Some(pick) => pick,
            None => match crate::ai::tools::resolve_capable_host(store, p, cap).await {
                Some((host, model)) => {
                    svc.trf(ctx, "ai_gen_auto", &[&format!("{}/{model}", host.name)])
                        .await
                }
                None => auto_none.clone(),
            },
        };
        body.push_str(&format!("**{label}**: {line}\n"));
    }
    body.push('\n');
    body.push_str(&svc.tr(ctx, "ai_gen_hint").await);
    body
}

// -- setup wizard --------------------------------------------------------
//
// Button-driven, one answer per message. State lives in the store (encrypt-
// ed, since it can hold a URL or key) as:
//   host:name                      - waiting for the new host's name
//   host:url:<name>                - waiting for its URL
//   host:key:<name>\t<url>         - waiting for an API key ("-" for none)
//   chat:name:<host_id>\t<model>   - waiting for the new chat's name

/// Feed one typed answer into the running wizard.
async fn wizard_step(
    ctx: &Ctx,
    svc: &Services,
    store: &crate::ai::AiStore,
    p: &str,
    input: &str,
) -> Result<()> {
    let Some(state) = store.wizard_state(p).await else {
        return Ok(());
    };
    let input = input.trim();

    if state == "host:name" {
        if input.contains(char::is_whitespace) {
            return wizard_prompt(ctx, svc, "ai_wiz_no_spaces").await;
        }
        if store.host(p, input).await.is_some() {
            return wizard_prompt(ctx, svc, "ai_host_exists").await;
        }
        store
            .set_wizard_state(p, &format!("host:url:{input}"))
            .await?;
        return wizard_prompt(ctx, svc, "ai_wiz_host_url").await;
    }
    if let Some(name) = state.strip_prefix("host:url:") {
        if !(input.starts_with("http://") || input.starts_with("https://")) {
            return wizard_prompt(ctx, svc, "ai_wiz_bad_url").await;
        }
        store
            .set_wizard_state(p, &format!("host:key:{name}\t{input}"))
            .await?;
        return wizard_prompt(ctx, svc, "ai_wiz_host_key").await;
    }
    if let Some(rest) = state.strip_prefix("host:key:") {
        let Some((name, url)) = rest.split_once('\t') else {
            store.clear_wizard(p).await?;
            return Ok(());
        };
        let key = if input == "-" { "" } else { input };
        let had_key = !key.is_empty();

        // Ask the host what it serves so the user doesn't have to type
        // model names by hand. Works for Ollama, LiteLLM, LM Studio and
        // anything else with an OpenAI-style /v1/models. A failed
        // discovery doesn't block the add - the reason is shown and the
        // manual `/ai model add` path still exists.
        ctx.typing().await;
        let (found, discover_err) =
            crate::ai::add_host_discovering(store, p, name, url, key).await?;
        store.clear_wizard(p).await?;

        if let Some(e) = discover_err {
            return host_added_no_discovery(ctx, svc, name, &e, had_key).await;
        }
        if found > 0 {
            let em = Embed::new()
                .description(
                    svc.trf(ctx, "ai_host_added_models", &[&found.to_string()])
                        .await,
                )
                .color(COLOR_OK);
            return ctx.reply_with(Reply::embed(em)).await;
        }
        if had_key {
            return ok(ctx, svc, "ai_host_added_key").await;
        }
        return ok(ctx, svc, "ai_host_added").await;
    }
    if let Some(host_id) = state.strip_prefix("chat:model:") {
        // A typed model name at the pick-model step. Used verbatim - it
        // is NOT added to the host's model list, so a typo doesn't stick.
        if input.is_empty() || input.contains(char::is_whitespace) {
            return wizard_prompt(ctx, svc, "ai_wiz_no_spaces").await;
        }
        store
            .set_wizard_state(p, &format!("chat:name:{host_id}\t{input}"))
            .await?;
        return wizard_prompt(ctx, svc, "ai_wiz_chat_name").await;
    }
    if let Some(rest) = state.strip_prefix("chat:name:") {
        let Some((host_id, model)) = rest.split_once('\t') else {
            store.clear_wizard(p).await?;
            return Ok(());
        };
        let chat = Chat {
            id: crate::ai::new_id("c"),
            name: input.to_owned(),
            host_id: host_id.to_owned(),
            model: model.to_owned(),
            system_prompt: String::new(),
        };
        let id = chat.id.clone();
        store.add_chat(p, chat).await?;
        store.set_active_chat(p, &id).await?;
        store.clear_wizard(p).await?;
        return ok(ctx, svc, "ai_chat_created").await;
    }

    // Unknown state: drop it so the user isn't stuck.
    store.clear_wizard(p).await?;
    Ok(())
}

/// Chat wizard, step 1: pick which host the chat will use.
/// Ask the next wizard question with a cancel button. When the update came
/// from a button press we edit that message in place; a typed answer gets a
/// fresh message (still with cancel), keeping the flow in one bubble where
/// the platform allows it.
async fn wizard_prompt(ctx: &Ctx, svc: &Services, key: &str) -> Result<()> {
    let kb = Keyboard::new().row([Button::callback("\u{274C}", "ai:wiz:cancel")]);
    let em = Embed::new()
        .description(svc.tr(ctx, key).await)
        .color(COLOR_WARN);
    if ctx.is_callback() {
        ctx.edit_reply(Reply::embed(em).keyboard(kb)).await
    } else {
        ctx.reply_with(Reply::embed(em).keyboard(kb)).await
    }
}

async fn wizard_pick_host(ctx: &Ctx, svc: &Services, store: &crate::ai::AiStore) -> Result<()> {
    let p = primary(ctx, svc).await;
    let own = store.hosts(&p).await?;
    let shared = store.shared_hosts_for(&p).await;
    if own.is_empty() && shared.is_empty() {
        return warn(ctx, svc, "ai_wiz_no_hosts").await;
    }
    let mut kb = Keyboard::new();
    for h in &own {
        kb = kb.row([Button::callback(
            h.name.clone(),
            format!("ai:wiz:chathost:{}", h.id),
        )]);
    }
    for (_owner, h, _models) in &shared {
        kb = kb.row([Button::callback(
            format!("\u{1F46A} {}", h.name),
            format!("ai:wiz:chathost:{}", h.id),
        )]);
    }
    kb = kb.row([Button::callback("\u{274C}", "ai:wiz:cancel")]);
    let em = Embed::new()
        .description(svc.tr(ctx, "ai_wiz_pick_host").await)
        .color(COLOR_ACCENT);
    ctx.edit_reply(Reply::embed(em).keyboard(kb)).await
}

/// Chat wizard, step 2: pick a model on the chosen host. Buttons cover
/// the discovered list; a typed message works too (any model name, used
/// as-is), which is how proxies with unlisted models stay reachable.
async fn wizard_pick_model(
    ctx: &Ctx,
    svc: &Services,
    store: &crate::ai::AiStore,
    host_id: &str,
) -> Result<()> {
    let p = primary(ctx, svc).await;
    // Own host or shared one; models come from whichever applies.
    let models: Vec<String> = if let Some(h) = store.host(&p, host_id).await {
        h.models
    } else {
        store
            .shared_hosts_for(&p)
            .await
            .into_iter()
            .find(|(_, h, _)| h.id == host_id)
            .map(|(_, _, models)| models)
            .unwrap_or_default()
    };
    // Arm the typed-input path even when the list is empty - that's
    // exactly when typing a name is the only way forward.
    store
        .set_wizard_state(&p, &format!("chat:model:{host_id}"))
        .await?;
    let mut kb = Keyboard::new();
    // Enumerate globally before chunking - a chunk-local index would point
    // every row at the first two models.
    let indexed: Vec<(usize, &String)> = models.iter().enumerate().collect();
    for chunk in indexed.chunks(2) {
        let row: Vec<Button> = chunk
            .iter()
            .map(|(i, m)| {
                // Index into the model list keeps the callback well under
                // Telegram's 64-byte cap even for long model names.
                Button::callback((*m).clone(), format!("ai:wiz:chatmodel:{host_id}:{i}"))
            })
            .collect();
        kb = kb.row(row);
    }
    kb = kb.row([Button::callback("\u{274C}", "ai:wiz:cancel")]);
    let em = Embed::new()
        .description(svc.tr(ctx, "ai_wiz_pick_model").await)
        .color(COLOR_ACCENT);
    ctx.edit_reply(Reply::embed(em).keyboard(kb)).await
}

/// Chat wizard, step 3: model chosen; ask for a chat name.
async fn wizard_model_chosen(
    ctx: &Ctx,
    svc: &Services,
    store: &crate::ai::AiStore,
    host_id: &str,
    idx: &str,
) -> Result<()> {
    let p = primary(ctx, svc).await;
    let models: Vec<String> = if let Some(h) = store.host(&p, host_id).await {
        h.models
    } else {
        store
            .shared_hosts_for(&p)
            .await
            .into_iter()
            .find(|(_, h, _)| h.id == host_id)
            .map(|(_, _, models)| models)
            .unwrap_or_default()
    };
    let Some(model) = idx.parse::<usize>().ok().and_then(|i| models.get(i)) else {
        return warn(ctx, svc, "ai_wiz_no_models").await;
    };
    store
        .set_wizard_state(&p, &format!("chat:name:{host_id}\t{model}"))
        .await?;
    wizard_prompt(ctx, svc, "ai_wiz_chat_name").await
}

// -- chat manage view ------------------------------------------------------
//
// Callback layout (all under `ai:cm:`, well below the 64-byte cap since
// chat ids are short hex):
//   o:<chat_id>        - open the view
//   m:<chat_id>        - pick a new model (buttons)
//   m:<chat_id>:<idx>  - set the model by index into the fresh list
//   d:<chat_id>        - delete, first tap (asks for confirmation)
//   dy:<chat_id>       - delete, confirmed

/// Route one `ai:cm:*` press. `rest` is the part after the prefix.
async fn chat_manage_cb(
    ctx: &Ctx,
    svc: &Services,
    store: &crate::ai::AiStore,
    rest: &str,
) -> Result<()> {
    let Some((action, tail)) = rest.split_once(':') else {
        return Ok(());
    };
    match action {
        "o" => chat_manage_view(ctx, svc, store, tail, false).await,
        "d" => chat_manage_view(ctx, svc, store, tail, true).await,
        "dy" => {
            let p = primary(ctx, svc).await;
            // Same path as `/ai chat del`: the chat and its history go.
            if store.chat(&p, tail).await.is_some() {
                store.remove_chat(&p, tail).await?;
            }
            show_menu(ctx, svc, store).await
        }
        "m" => {
            // Bare chat id opens the picker; a trailing index sets the model.
            match tail.rsplit_once(':') {
                Some((chat_id, idx)) if idx.chars().all(|c| c.is_ascii_digit()) => {
                    chat_set_model(ctx, svc, store, chat_id, idx).await
                }
                _ => chat_pick_model(ctx, svc, store, tail).await,
            }
        }
        _ => Ok(()),
    }
}

/// The models a chat may switch to: its host's full list for an own host,
/// only the granted ones for a family-shared host.
async fn chat_model_choices(store: &crate::ai::AiStore, p: &str, chat: &Chat) -> Vec<String> {
    if let Some(h) = store.host(p, &chat.host_id).await {
        return h.models;
    }
    store
        .shared_hosts_for(p)
        .await
        .into_iter()
        .find(|(_, h, _)| h.id == chat.host_id)
        .map(|(_, _, models)| models)
        .unwrap_or_default()
}

/// The settings card for one chat: use / change model / delete / back.
/// `confirm_delete` swaps the delete row for a "sure?" yes/no pair.
async fn chat_manage_view(
    ctx: &Ctx,
    svc: &Services,
    store: &crate::ai::AiStore,
    chat_id: &str,
    confirm_delete: bool,
) -> Result<()> {
    let p = primary(ctx, svc).await;
    // Stale button (chat already deleted): fall back to the menu.
    let Some(chat) = store.chat(&p, chat_id).await else {
        return show_menu(ctx, svc, store).await;
    };
    let host_name = match store.host(&p, &chat.host_id).await {
        Some(h) => h.name,
        None => store
            .shared_hosts_for(&p)
            .await
            .into_iter()
            .find(|(_, h, _)| h.id == chat.host_id)
            .map(|(_, h, _)| format!("\u{1F46A} {}", h.name))
            .unwrap_or_else(|| "?".to_owned()),
    };

    let mut body = svc
        .trf(ctx, "ai_chat_manage_body", &[&chat.model, &host_name])
        .await;
    if confirm_delete {
        body.push_str("\n\n");
        body.push_str(&svc.tr(ctx, "ai_chat_del_confirm").await);
    }

    let mut kb = Keyboard::new().row([Button::callback(
        svc.tr(ctx, "ai_btn_use").await,
        format!("ai:use:{chat_id}"),
    )]);
    kb = kb.row([Button::callback(
        svc.tr(ctx, "ai_btn_change_model").await,
        format!("ai:cm:m:{chat_id}"),
    )]);
    if confirm_delete {
        kb = kb.row([
            Button::callback(
                svc.tr(ctx, "ai_btn_del_yes").await,
                format!("ai:cm:dy:{chat_id}"),
            ),
            Button::callback(
                svc.tr(ctx, "ai_btn_del_no").await,
                format!("ai:cm:o:{chat_id}"),
            ),
        ]);
    } else {
        kb = kb.row([Button::callback(
            svc.tr(ctx, "ai_btn_delete").await,
            format!("ai:cm:d:{chat_id}"),
        )]);
    }
    kb = kb.row([Button::callback(
        svc.tr(ctx, "ai_btn_back").await,
        "ai:menu",
    )]);

    let em = Embed::new()
        .title(format!("\u{2699}\u{FE0F} {}", chat.name))
        .description(body)
        .color(COLOR_ACCENT);
    ctx.edit_reply(Reply::embed(em).keyboard(kb)).await
}

/// How many model buttons a picker shows at most.
const MODEL_BUTTON_CAP: usize = 12;

/// Model picker for a chat: the host's models as buttons, current one
/// marked. History survives a switch - only the model field changes.
async fn chat_pick_model(
    ctx: &Ctx,
    svc: &Services,
    store: &crate::ai::AiStore,
    chat_id: &str,
) -> Result<()> {
    let p = primary(ctx, svc).await;
    let Some(chat) = store.chat(&p, chat_id).await else {
        return show_menu(ctx, svc, store).await;
    };
    let models = chat_model_choices(store, &p, &chat).await;
    let mut kb = Keyboard::new();
    // Global enumeration before chunking, same trick as the wizard: the
    // callback carries an index into this fresh list, not the name.
    let indexed: Vec<(usize, &String)> = models.iter().take(MODEL_BUTTON_CAP).enumerate().collect();
    for chunk in indexed.chunks(2) {
        let row: Vec<Button> = chunk
            .iter()
            .map(|(i, m)| {
                let mark = if **m == chat.model { "\u{2705} " } else { "" };
                Button::callback(format!("{mark}{m}"), format!("ai:cm:m:{chat_id}:{i}"))
            })
            .collect();
        kb = kb.row(row);
    }
    kb = kb.row([Button::callback(
        svc.tr(ctx, "ai_btn_back").await,
        format!("ai:cm:o:{chat_id}"),
    )]);
    let key = if models.is_empty() {
        "ai_wiz_no_models"
    } else {
        "ai_chat_pick_model"
    };
    let em = Embed::new()
        .title(format!("\u{2699}\u{FE0F} {}", chat.name))
        .description(svc.tr(ctx, key).await)
        .color(COLOR_ACCENT);
    ctx.edit_reply(Reply::embed(em).keyboard(kb)).await
}

/// Apply a model pick. The list is re-read on press, so a stale index
/// (models changed since the buttons were drawn) just redraws the picker.
async fn chat_set_model(
    ctx: &Ctx,
    svc: &Services,
    store: &crate::ai::AiStore,
    chat_id: &str,
    idx: &str,
) -> Result<()> {
    let p = primary(ctx, svc).await;
    let Some(chat) = store.chat(&p, chat_id).await else {
        return show_menu(ctx, svc, store).await;
    };
    let models = chat_model_choices(store, &p, &chat).await;
    let Some(model) = idx.parse::<usize>().ok().and_then(|i| models.get(i)) else {
        return chat_pick_model(ctx, svc, store, chat_id).await;
    };
    let mut chats = store.chats(&p).await?;
    if let Some(c) = chats.iter_mut().find(|c| c.id == chat_id) {
        c.model = model.clone();
    }
    store.set_chats(&p, &chats).await?;
    chat_manage_view(ctx, svc, store, chat_id, false).await
}

// -- host manage view -------------------------------------------------------
//
// Callback layout (under `ai:hm:`), own hosts only - a shared host has no
// manage view, since a grantee must not touch the owner's settings:
//   o:<host_id>        - open the view
//   r:<host_id>        - re-pull the model list
//   i:<host_id>        - toggle the self-signed certificate switch
//   c:<host_id>        - pick a model to check for canned answers
//   c:<host_id>:<idx>  - run the check on the model at that index
//   d:<host_id>        - delete, first tap (asks for confirmation)
//   dy:<host_id>       - delete, confirmed

/// Route one `ai:hm:*` press.
async fn host_manage_cb(
    ctx: &Ctx,
    svc: &Services,
    store: &crate::ai::AiStore,
    rest: &str,
) -> Result<()> {
    let Some((action, tail)) = rest.split_once(':') else {
        return Ok(());
    };
    // The check actions carry an optional model index after the host id.
    let (host_id, idx) = match action {
        "c" => match tail.rsplit_once(':') {
            Some((id, idx)) if idx.chars().all(|c| c.is_ascii_digit()) => (id, Some(idx)),
            _ => (tail, None),
        },
        _ => (tail, None),
    };
    let p = primary(ctx, svc).await;
    // Every action needs the host to still exist and be the presser's own.
    let Some(host) = store.host(&p, host_id).await else {
        return show_menu(ctx, svc, store).await;
    };
    match action {
        "o" => host_manage_view(ctx, svc, store, &host, false).await,
        "d" => host_manage_view(ctx, svc, store, &host, true).await,
        "dy" => {
            store.remove_host(&p, host_id).await?;
            show_menu(ctx, svc, store).await
        }
        "c" => match idx {
            // A picked model: run the probes. The report goes out as a
            // fresh message so the picker stays usable.
            Some(idx) => {
                let Some(model) = idx.parse::<usize>().ok().and_then(|i| host.models.get(i)) else {
                    return host_check_pick(ctx, svc, &host).await;
                };
                run_check(ctx, svc, &host, &model.clone()).await
            }
            None => host_check_pick(ctx, svc, &host).await,
        },
        "i" => {
            crate::ai::set_host_insecure(store, &p, &host.name, !host.insecure).await?;
            // Re-read so the view shows the flipped state.
            match store.host(&p, host_id).await {
                Some(h) => host_manage_view(ctx, svc, store, &h, false).await,
                None => show_menu(ctx, svc, store).await,
            }
        }
        "r" => {
            ctx.typing().await;
            // Same path as `/ai host refresh`, result shown in place with
            // a way back to the host card.
            let body = match crate::ai::refresh_host_models(store, &p, &host).await? {
                Ok(0) => svc.tr(ctx, "ai_refresh_none").await,
                Ok(n) => {
                    svc.trf(ctx, "ai_host_added_models", &[&n.to_string()])
                        .await
                }
                Err(e) => discover_failed_body(ctx, svc, &host.name, &e).await,
            };
            let kb = Keyboard::new().row([Button::callback(
                svc.tr(ctx, "ai_btn_back").await,
                format!("ai:hm:o:{host_id}"),
            )]);
            let em = Embed::new().description(body).color(COLOR_ACCENT);
            ctx.edit_reply(Reply::embed(em).keyboard(kb)).await
        }
        _ => Ok(()),
    }
}

/// The settings card for one own host: refresh / insecure toggle / delete.
async fn host_manage_view(
    ctx: &Ctx,
    svc: &Services,
    _store: &crate::ai::AiStore,
    host: &Host,
    confirm_delete: bool,
) -> Result<()> {
    let count = svc
        .trf(ctx, "ai_models_count", &[&host.models.len().to_string()])
        .await;
    let insecure_line = if host.insecure {
        svc.tr(ctx, "ai_host_manage_insecure_on").await
    } else {
        svc.tr(ctx, "ai_host_manage_insecure_off").await
    };
    let mut body = format!("`{}` {count}\n{insecure_line}", host.base_url);
    if confirm_delete {
        body.push_str("\n\n");
        body.push_str(&svc.tr(ctx, "ai_host_del_confirm").await);
    }

    let mut kb = Keyboard::new().row([Button::callback(
        svc.tr(ctx, "ai_btn_refresh").await,
        format!("ai:hm:r:{}", host.id),
    )]);
    // Only offer the check when there's something to check.
    if !host.models.is_empty() {
        kb = kb.row([Button::callback(
            svc.tr(ctx, "ai_btn_check").await,
            format!("ai:hm:c:{}", host.id),
        )]);
    }
    let insecure_label = if host.insecure {
        svc.tr(ctx, "ai_btn_insecure_off").await
    } else {
        svc.tr(ctx, "ai_btn_insecure_on").await
    };
    kb = kb.row([Button::callback(
        insecure_label,
        format!("ai:hm:i:{}", host.id),
    )]);
    if confirm_delete {
        kb = kb.row([
            Button::callback(
                svc.tr(ctx, "ai_btn_del_yes").await,
                format!("ai:hm:dy:{}", host.id),
            ),
            Button::callback(
                svc.tr(ctx, "ai_btn_del_no").await,
                format!("ai:hm:o:{}", host.id),
            ),
        ]);
    } else {
        kb = kb.row([Button::callback(
            svc.tr(ctx, "ai_btn_delete").await,
            format!("ai:hm:d:{}", host.id),
        )]);
    }
    kb = kb.row([Button::callback(
        svc.tr(ctx, "ai_btn_back").await,
        "ai:menu",
    )]);

    let em = Embed::new()
        .title(format!("\u{2699}\u{FE0F} {}", host.name))
        .description(body)
        .color(COLOR_ACCENT);
    ctx.edit_reply(Reply::embed(em).keyboard(kb)).await
}

/// Model picker for the liveness check: the host's models as buttons,
/// same index-into-fresh-list trick as the chat model picker.
async fn host_check_pick(ctx: &Ctx, svc: &Services, host: &Host) -> Result<()> {
    let mut kb = Keyboard::new();
    let indexed: Vec<(usize, &String)> = host
        .models
        .iter()
        .take(MODEL_BUTTON_CAP)
        .enumerate()
        .collect();
    for chunk in indexed.chunks(2) {
        let row: Vec<Button> = chunk
            .iter()
            .map(|(i, m)| Button::callback((*m).clone(), format!("ai:hm:c:{}:{i}", host.id)))
            .collect();
        kb = kb.row(row);
    }
    kb = kb.row([Button::callback(
        svc.tr(ctx, "ai_btn_back").await,
        format!("ai:hm:o:{}", host.id),
    )]);
    let key = if host.models.is_empty() {
        "ai_wiz_no_models"
    } else {
        "ai_check_pick"
    };
    let em = Embed::new()
        .title(format!("\u{2699}\u{FE0F} {}", host.name))
        .description(svc.tr(ctx, key).await)
        .color(COLOR_ACCENT);
    ctx.edit_reply(Reply::embed(em).keyboard(kb)).await
}

// -- generation pins by buttons ----------------------------------------------
//
// Callback layout:
//   ai:gen           - overview with one button per capability
//   ai:gen:<c>       - candidate models for that capability (c = i|v|a)
//   ai:gen:<c>:auto  - drop the pin, back to auto-pick
//   ai:gen:<c>:<idx> - pin the candidate at that index (fresh list)

/// One-letter capability codes keep the callback tiny.
fn cap_from_code(code: &str) -> Option<u8> {
    match code {
        "i" => Some(ModelCaps::IMAGE),
        "v" => Some(ModelCaps::VIDEO),
        "a" => Some(ModelCaps::AUDIO),
        _ => None,
    }
}

fn cap_code(cap: u8) -> &'static str {
    match cap {
        ModelCaps::IMAGE => "i",
        ModelCaps::VIDEO => "v",
        _ => "a",
    }
}

/// How many pin candidates the picker offers at most.
const GEN_CANDIDATE_CAP: usize = 10;

/// Route one `ai:gen*` press. `data` is the full callback payload.
async fn gen_cb(ctx: &Ctx, svc: &Services, store: &crate::ai::AiStore, data: &str) -> Result<()> {
    let Some(rest) = data.strip_prefix("ai:gen") else {
        return Ok(());
    };
    if rest.is_empty() {
        return gen_view(ctx, svc, store).await;
    }
    let rest = rest.trim_start_matches(':');
    let (code, tail) = match rest.split_once(':') {
        Some((c, t)) => (c, Some(t)),
        None => (rest, None),
    };
    let Some(cap) = cap_from_code(code) else {
        return Ok(());
    };
    let p = primary(ctx, svc).await;
    match tail {
        None => gen_pick_view(ctx, svc, store, &p, cap).await,
        Some("auto") => {
            // Same as `/ai gen <cap> auto`.
            store.clear_gen_pref(&p, cap).await?;
            gen_view(ctx, svc, store).await
        }
        Some(idx) => {
            // Index into a freshly enumerated candidate list; a stale one
            // (models changed since the buttons were drawn) redraws the
            // picker instead of pinning the wrong thing.
            let candidates = gen_candidates(store, &p, cap).await;
            let Some((host_name, model)) =
                idx.parse::<usize>().ok().and_then(|i| candidates.get(i))
            else {
                return gen_pick_view(ctx, svc, store, &p, cap).await;
            };
            store.set_gen_pref(&p, cap, host_name, model).await?;
            gen_view(ctx, svc, store).await
        }
    }
}

/// Every host/model pair the user can pin for a capability: own hosts
/// first, then family-shared ones (granted models only). Capped so the
/// keyboard stays sane.
async fn gen_candidates(store: &crate::ai::AiStore, p: &str, cap: u8) -> Vec<(String, String)> {
    let mut out = Vec::new();
    for host in store.hosts(p).await.unwrap_or_default() {
        for m in &host.models {
            if host.caps_of(m).0 & cap != 0 {
                out.push((host.name.clone(), m.clone()));
            }
        }
    }
    for (_owner, host, models) in store.shared_hosts_for(p).await {
        for m in &models {
            if host.caps_of(m).0 & cap != 0 {
                out.push((host.name.clone(), m.clone()));
            }
        }
    }
    out.truncate(GEN_CANDIDATE_CAP);
    out
}

/// The button-driven generation overview: same lines as `/ai gen`, plus
/// one button per capability to change its pin.
async fn gen_view(ctx: &Ctx, svc: &Services, store: &crate::ai::AiStore) -> Result<()> {
    let p = primary(ctx, svc).await;
    let body = gen_overview_body(ctx, svc, store, &p).await;
    let mut kb = Keyboard::new();
    let mut row = Vec::new();
    for cap in [ModelCaps::IMAGE, ModelCaps::VIDEO, ModelCaps::AUDIO] {
        row.push(Button::callback(
            cap_label(ctx, svc, cap).await,
            format!("ai:gen:{}", cap_code(cap)),
        ));
    }
    kb = kb.row(row);
    kb = kb.row([Button::callback(
        svc.tr(ctx, "ai_btn_back").await,
        "ai:menu",
    )]);
    let em = Embed::new()
        .title(svc.tr(ctx, "ai_gen_title").await)
        .description(body.trim_end())
        .color(COLOR_ACCENT);
    ctx.edit_reply(Reply::embed(em).keyboard(kb)).await
}

/// Candidate picker for one capability: "host/model" buttons plus Auto.
async fn gen_pick_view(
    ctx: &Ctx,
    svc: &Services,
    store: &crate::ai::AiStore,
    p: &str,
    cap: u8,
) -> Result<()> {
    let label = cap_label(ctx, svc, cap).await;
    let candidates = gen_candidates(store, p, cap).await;
    let mut body = svc.trf(ctx, "ai_gen_pick", &[&label]).await;
    if candidates.is_empty() {
        body.push_str("\n\n");
        body.push_str(&svc.tr(ctx, "ai_gen_no_candidates").await);
    }
    let code = cap_code(cap);
    let mut kb = Keyboard::new();
    let indexed: Vec<(usize, &(String, String))> = candidates.iter().enumerate().collect();
    for chunk in indexed.chunks(2) {
        let row: Vec<Button> = chunk
            .iter()
            .map(|(i, (host, model))| {
                Button::callback(format!("{host}/{model}"), format!("ai:gen:{code}:{i}"))
            })
            .collect();
        kb = kb.row(row);
    }
    kb = kb.row([Button::callback(
        svc.tr(ctx, "ai_btn_auto").await,
        format!("ai:gen:{code}:auto"),
    )]);
    kb = kb.row([Button::callback(svc.tr(ctx, "ai_btn_back").await, "ai:gen")]);
    let em = Embed::new()
        .title(svc.tr(ctx, "ai_gen_title").await)
        .description(body)
        .color(COLOR_ACCENT);
    ctx.edit_reply(Reply::embed(em).keyboard(kb)).await
}

// -- family access -----------------------------------------------------------

/// `/ai share <user> <host> [model1,model2,...]` - offer a grantee access
/// to one of your hosts. `<user>` is `platform:id` (or a bare id for the
/// same platform). With no model list, all of that host's models are
/// shared.
async fn share_offer(
    ctx: &Ctx,
    svc: &Services,
    store: &crate::ai::AiStore,
    rest: &str,
) -> Result<()> {
    let owner = primary(ctx, svc).await;
    let mut f = rest.split_whitespace();
    let (Some(target), Some(host_name)) = (f.next(), f.next()) else {
        return warn(ctx, svc, "ai_share_usage").await;
    };
    // The model list may contain spaces after commas ("m1, m2"), so glue
    // the remaining tokens back together instead of taking just one.
    let models_arg = f.collect::<Vec<_>>().join(" ");
    let models_arg = models_arg.as_str();

    // Resolve the target to their primary identity: the pending invite is
    // stored under it and the accept button looks it up by the presser's
    // primary, so both sides must agree even when the grantee has linked
    // accounts. This also makes the self-share check watertight - typing
    // your own alt's id still maps back to your primary.
    let raw = normalize_identity(ctx, target);
    let grantee = match split_identity(&raw) {
        Some((platform, id)) => svc
            .accounts
            .primary_for(platform, &id)
            .await
            .unwrap_or(raw.clone()),
        None => raw.clone(),
    };
    if grantee == owner {
        return warn(ctx, svc, "ai_share_self").await;
    }

    let Some(host) = find_host_by_name(store, &owner, host_name).await else {
        return warn(ctx, svc, "ai_host_missing").await;
    };
    let models: Vec<String> = if models_arg.is_empty() {
        host.models.clone()
    } else {
        models_arg
            .split(',')
            .map(|m| m.trim().to_owned())
            .filter(|m| !m.is_empty() && host.models.iter().any(|hm| hm == m))
            .collect()
    };
    if models.is_empty() {
        return warn(ctx, svc, "ai_share_no_models").await;
    }

    let created_at = now_ts();
    let pend = PendingShare {
        owner: owner.clone(),
        hosts: vec![SharedHost {
            host_id: host.id.clone(),
            models,
        }],
        created_at,
    };
    store.add_pending(&grantee, pend).await?;

    notify_invite(svc, &owner, &grantee).await;
    spawn_invite_timeout(svc.clone(), owner.clone(), grantee.clone(), created_at);

    ok(ctx, svc, "ai_share_sent").await
}

/// `/ai unshare <user>` - revoke a grantee's access entirely.
async fn share_revoke(
    ctx: &Ctx,
    svc: &Services,
    store: &crate::ai::AiStore,
    rest: &str,
) -> Result<()> {
    let owner = primary(ctx, svc).await;
    let target = rest.split_whitespace().next().unwrap_or("");
    if target.is_empty() {
        return warn(ctx, svc, "ai_unshare_usage").await;
    }
    // Same resolution as share_offer: shares are keyed by the grantee's
    // primary, so revoking by any of their linked ids must land on it.
    let raw = normalize_identity(ctx, target);
    let grantee = match split_identity(&raw) {
        Some((platform, id)) => svc
            .accounts
            .primary_for(platform, &id)
            .await
            .unwrap_or(raw.clone()),
        None => raw.clone(),
    };
    store.revoke_share(&owner, &grantee).await?;
    ok(ctx, svc, "ai_unshare_done").await
}

/// `/ai shared` - list who you've granted access to, and what.
async fn share_list(ctx: &Ctx, svc: &Services, store: &crate::ai::AiStore) -> Result<()> {
    let owner = primary(ctx, svc).await;
    let shares = store.shares(&owner).await?;
    let hosts = store.hosts(&owner).await?;

    if shares.is_empty() {
        return ok(ctx, svc, "ai_shared_none").await;
    }
    let mut body = String::new();
    for share in &shares {
        body.push_str(&format!("• {}\n", pretty_identity(&share.grantee)));
        for sh in &share.hosts {
            let host_name = hosts
                .iter()
                .find(|h| h.id == sh.host_id)
                .map(|h| h.name.as_str())
                .unwrap_or("?");
            body.push_str(&format!("   {} - {}\n", host_name, sh.models.join(", ")));
        }
    }
    let em = Embed::new()
        .title(svc.tr(ctx, "ai_family_title").await)
        .description(body.trim_end())
        .color(COLOR_ACCENT);
    ctx.reply_with(Reply::embed(em)).await
}

/// Grantee accepts an invite: turn the pending offer into a live share.
async fn share_accept(
    ctx: &Ctx,
    svc: &Services,
    store: &crate::ai::AiStore,
    owner: &str,
) -> Result<()> {
    let grantee = primary(ctx, svc).await;
    let pending = store.pending(&grantee).await?;
    let Some(pend) = pending.into_iter().find(|p| p.owner == owner) else {
        return warn(ctx, svc, "ai_invite_gone").await;
    };
    if now_ts() - pend.created_at > 24 * 3600 {
        let _ = store.remove_pending(&grantee, owner).await;
        return warn(ctx, svc, "ai_invite_expired").await;
    }
    store
        .upsert_share(
            owner,
            Share {
                grantee: grantee.clone(),
                hosts: pend.hosts,
            },
        )
        .await?;
    store.remove_pending(&grantee, owner).await?;
    ok(ctx, svc, "ai_invite_accepted").await
}

/// Grantee declines an invite.
async fn share_decline(
    ctx: &Ctx,
    svc: &Services,
    store: &crate::ai::AiStore,
    owner: &str,
) -> Result<()> {
    let grantee = primary(ctx, svc).await;
    store.remove_pending(&grantee, owner).await?;
    ok(ctx, svc, "ai_invite_declined").await
}

/// Send the invite DM with accept/decline buttons, best effort.
/// The DM goes to the grantee, so their language wins - not the owner's.
async fn notify_invite(svc: &Services, owner: &str, grantee: &str) {
    let (platform, chat_id) = match split_identity(grantee) {
        Some(v) => v,
        None => return,
    };
    let lang = svc.lang_of(platform, &chat_id).await;
    let kb = foukoapi::Keyboard::new().row([
        foukoapi::Button::callback(
            svc.i18n.t(&lang, "ai_invite_accept_btn"),
            format!("ai:accept:{owner}"),
        ),
        foukoapi::Button::callback(
            svc.i18n.t(&lang, "ai_invite_decline_btn"),
            format!("ai:decline:{owner}"),
        ),
    ]);
    let em = Embed::new()
        .title(svc.i18n.t(&lang, "ai_family_invite_title"))
        .description(
            svc.i18n
                .tf(&lang, "ai_family_invite_dm", &[&pretty_identity(owner)]),
        )
        .color(COLOR_ACCENT);
    let _ = svc
        .notifier
        .send(platform, chat_id, Reply::embed(em).keyboard(kb))
        .await;
}

/// After 24h, if the invite is still pending, tell the owner it went
/// unanswered (best effort) and drop it.
///
/// `created_at` pins the timer to the invite it was armed for: if the
/// owner re-sends a fresh invite meanwhile (which replaces the pending
/// row), the stale timer must not cancel the new one early.
fn spawn_invite_timeout(svc: Services, owner: String, grantee: String, created_at: i64) {
    let Some(store) = svc.ai.clone() else {
        return;
    };
    tokio::spawn(async move {
        tokio::time::sleep(std::time::Duration::from_secs(24 * 3600)).await;
        let still = store
            .pending(&grantee)
            .await
            .unwrap_or_default()
            .into_iter()
            .any(|p| p.owner == owner && p.created_at == created_at);
        if !still {
            return;
        }
        let _ = store.remove_pending(&grantee, &owner).await;
        if let Some((platform, chat_id)) = split_identity(&owner) {
            // The notice goes to the invite's owner: use their language.
            let lang = svc.lang_of(platform, &chat_id).await;
            let em = Embed::new()
                .title(svc.i18n.t(&lang, "ai_family_timeout_title"))
                .description(svc.i18n.tf(
                    &lang,
                    "ai_family_timeout_body",
                    &[&pretty_identity(&grantee)],
                ))
                .color(COLOR_WARN);
            let _ = svc.notifier.send(platform, chat_id, Reply::embed(em)).await;
        }
    });
}

/// Turn a user-typed target into a `platform:id`. A bare token is assumed
/// to be on the same platform as the caller.
fn normalize_identity(ctx: &Ctx, target: &str) -> String {
    let target = target
        .trim_start_matches("<@")
        .trim_start_matches('!')
        .trim_end_matches('>')
        .trim_start_matches('@');
    if target.contains(':') {
        target.to_owned()
    } else {
        format!("{}:{}", ctx.platform(), target)
    }
}

/// Split `"platform:id"` into a `PlatformKind` and the id, for the notifier.
fn split_identity(ident: &str) -> Option<(PlatformKind, String)> {
    let (platform, id) = ident.split_once(':')?;
    let platform = match platform {
        "telegram" => PlatformKind::Telegram,
        "discord" => PlatformKind::Discord,
        _ => return None,
    };
    Some((platform, id.to_owned()))
}

fn pretty_identity(ident: &str) -> String {
    let (platform, id) = ident.split_once(':').unwrap_or(("", ident));
    let short: String = id.chars().take(8).collect();
    if platform.is_empty() {
        short
    } else {
        let mut c = platform.chars();
        let cap = c
            .next()
            .map(|f| f.to_uppercase().collect::<String>() + c.as_str());
        format!("{} {short}", cap.unwrap_or_default())
    }
}

fn now_ts() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

// -- helpers -----------------------------------------------------------------

async fn ok(ctx: &Ctx, svc: &Services, key: &str) -> Result<()> {
    let em = Embed::new()
        .title("\u{2705}")
        .description(svc.tr(ctx, key).await)
        .color(COLOR_OK);
    ctx.reply_with(Reply::embed(em)).await
}

async fn warn(ctx: &Ctx, svc: &Services, key: &str) -> Result<()> {
    let em = Embed::new()
        .title("\u{2139}\u{FE0F}")
        .description(svc.tr(ctx, key).await)
        .color(COLOR_WARN);
    ctx.reply_with(Reply::embed(em)).await
}
