//! The `/ai` command: manage private LLM hosts, chats and settings, and
//! talk to a model. Management is DM-only and owner-only; a chat message
//! only ever carries the user's own system prompt and history.
//!
//! Sub-commands (all under `/ai`):
//!   /ai                         - show your menu (chats, hosts)
//!   /ai host add <name> <url> [key]
//!   /ai host del <name>
//!   /ai model add <host> <model>
//!   /ai model del <host> <model>
//!   /ai chat new <name> <host> <model>
//!   /ai chat del <name>
//!   /ai use <chat>              - pick the active chat
//!   /ai prompt <text>           - set the active chat's system prompt
//!   /ai clear                   - wipe the active chat's history
//!   /ai say <text>              - talk (also: plain text in a DM)

use crate::ai::{Chat, ChatMessage, Host, PendingShare, Share, SharedHost};
use crate::commands::Services;
use foukoapi::{Button, Ctx, Embed, Keyboard, PlatformKind, Reply, Result};

const COLOR_ACCENT: u32 = 0x7A5BE8;
const COLOR_OK: u32 = 0x43B581;
const COLOR_WARN: u32 = 0xF59F00;

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
        "help" => ai_help(ctx, svc).await,
        "use" => use_chat(ctx, svc, &store, &rest).await,
        "clear" => clear_history(ctx, svc, &store).await,
        "host" | "model" | "chat" | "prompt" | "share" | "unshare" | "shared" => {
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
                "chat" => manage_chat(ctx, svc, &store, &rest).await,
                "prompt" => set_prompt(ctx, svc, &store, &rest).await,
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
    // that to the model. Also skip commands and empty text.
    if ctx.is_callback() || ctx.text().trim().starts_with('/') || ctx.text().trim().is_empty() {
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
                "`/ai model add <хост> <модель>` - добавить модель вручную\n",
                "`/ai chat new <имя> <хост> <модель>` - создать чат\n",
                "`/ai prompt <текст>` - задать системный промпт активного чата\n",
                "`/ai use <чат>` - выбрать активный чат\n",
                "`/ai clear` - очистить историю активного чата\n\n",
                "**Общение:** просто пиши в ЛС, или `/ai say <текст>` где угодно\n\n",
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
                "`/ai model add <host> <model>` - add a model by hand\n",
                "`/ai chat new <name> <host> <model>` - create a chat\n",
                "`/ai prompt <text>` - set the active chat's system prompt\n",
                "`/ai use <chat>` - pick the active chat\n",
                "`/ai clear` - clear the active chat's history\n\n",
                "**Chatting:** just type in DM, or `/ai say <text>` anywhere\n\n",
                "**Family access (in DM):**\n",
                "`/ai share <user> <host> [models]` - share a host\n",
                "`/ai unshare <user>` - revoke access\n",
                "`/ai shared` - see who you've shared with\n\n",
                "Everything is stored encrypted and follows your /link.",
            ),
        ),
    };
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

    // One button per chat to switch the active one, chunked for a grid.
    for chunk in chats.chunks(2) {
        let row: Vec<Button> = chunk
            .iter()
            .map(|c| {
                let mark = if Some(&c.id) == active.as_ref() {
                    "\u{2705} "
                } else {
                    ""
                };
                Button::callback(format!("{mark}{}", c.name), format!("ai:use:{}", c.id))
            })
            .collect();
        kb = kb.row(row);
    }
    if active.is_some() {
        let clear_label = if lang == "ru" {
            "\u{1F9F9} Очистить историю"
        } else {
            "\u{1F9F9} Clear history"
        };
        kb = kb.row([Button::callback(clear_label, "ai:clear")]);
    }
    // Setup actions: add a host, and (once there's a host) a new chat.
    let (add_host_label, add_chat_label) = if lang == "ru" {
        ("\u{2795} Хост", "\u{2795} Чат")
    } else {
        ("\u{2795} Host", "\u{2795} Chat")
    };
    if hosts.is_empty() {
        kb = kb.row([Button::callback(add_host_label, "ai:wiz:addhost")]);
    } else {
        kb = kb.row([
            Button::callback(add_host_label, "ai:wiz:addhost"),
            Button::callback(add_chat_label, "ai:wiz:addchat"),
        ]);
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
            let key = f.next().unwrap_or("").to_owned();
            let had_key = !key.is_empty();
            if store.host(&p, name).await.is_some() {
                return warn(ctx, svc, "ai_host_exists").await;
            }
            let mut host = Host {
                id: crate::ai::new_id("h"),
                name: name.to_owned(),
                base_url: url.to_owned(),
                api_key: key,
                models: Vec::new(),
            };
            // Same auto-discovery as the wizard: pull the model list from
            // the host when it exposes one.
            ctx.typing().await;
            let discovered = crate::ai::list_models(&host).await;
            let found = discovered.len();
            host.models = discovered;
            store.add_host(&p, host).await?;
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
            let discovered = crate::ai::list_models(&host).await;
            if discovered.is_empty() {
                return warn(ctx, svc, "ai_refresh_none").await;
            }
            let found = discovered.len();
            let mut hosts = store.hosts(&p).await?;
            if let Some(h) = hosts.iter_mut().find(|h| h.id == host.id) {
                h.models = discovered;
            }
            store.set_hosts(&p, &hosts).await?;
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
        _ => warn(ctx, svc, "ai_host_usage").await,
    }
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
    let mut f = tail.splitn(2, char::is_whitespace);
    let host_name = f.next().unwrap_or("");
    let model = f.next().unwrap_or("").trim();

    if host_name.is_empty() || model.is_empty() {
        return warn(ctx, svc, "ai_model_usage").await;
    }
    let mut hosts = store.hosts(&p).await?;
    let Some(host) = hosts.iter_mut().find(|h| h.name == host_name) else {
        return warn(ctx, svc, "ai_host_missing").await;
    };
    match action.as_str() {
        "add" => {
            if !host.models.iter().any(|m| m == model) {
                host.models.push(model.to_owned());
            }
            store.set_hosts(&p, &hosts).await?;
            ok(ctx, svc, "ai_model_added").await
        }
        "del" => {
            host.models.retain(|m| m != model);
            store.set_hosts(&p, &hosts).await?;
            ok(ctx, svc, "ai_model_removed").await
        }
        _ => warn(ctx, svc, "ai_model_usage").await,
    }
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
            let Some((host_id, ok_model)) =
                resolve_host_for_chat(store, &p, host_name, model).await
            else {
                return warn(ctx, svc, "ai_host_missing").await;
            };
            if !ok_model {
                return warn(ctx, svc, "ai_model_missing").await;
            }
            let chat = Chat {
                id: crate::ai::new_id("c"),
                name: name.to_owned(),
                host_id,
                model: model.to_owned(),
                system_prompt: String::new(),
            };
            let id = chat.id.clone();
            store.add_chat(&p, chat).await?;
            store.set_active_chat(&p, &id).await?;
            ok(ctx, svc, "ai_chat_created").await
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

/// Send `text` to the active chat's model and reply with the answer.
async fn talk(ctx: &Ctx, svc: &Services, store: &crate::ai::AiStore, text: &str) -> Result<()> {
    if text.trim().is_empty() {
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

    // Build the message list: the user's own system prompt (if any) plus
    // their stored history plus the new turn. Nothing else is added.
    let mut messages: Vec<ChatMessage> = Vec::new();
    if !chat.system_prompt.is_empty() {
        messages.push(ChatMessage {
            role: "system".to_owned(),
            content: chat.system_prompt.clone(),
        });
    }
    messages.extend(store.history(&p, &active).await?);
    messages.push(ChatMessage {
        role: "user".to_owned(),
        content: text.to_owned(),
    });

    // Let the user know we're working - model replies can take a while.
    ctx.typing().await;

    match crate::ai::chat_completion(&host, &chat.model, &messages).await {
        Ok(answer) => {
            // Persist both turns so the conversation continues next time.
            store
                .push_history(
                    &p,
                    &active,
                    ChatMessage {
                        role: "user".to_owned(),
                        content: text.to_owned(),
                    },
                )
                .await?;
            store
                .push_history(
                    &p,
                    &active,
                    ChatMessage {
                        role: "assistant".to_owned(),
                        content: answer.clone(),
                    },
                )
                .await?;
            send_answer(ctx, &chat.name, &answer).await
        }
        Err(e) => {
            let em = Embed::new()
                .description(svc.trf(ctx, "ai_error", &[&e]).await)
                .color(COLOR_WARN);
            ctx.reply_with(Reply::embed(em)).await
        }
    }
}

/// Send a model answer, splitting it across messages when it's long so
/// nothing gets clipped at the platform's length limit. No decoration -
/// just the model's text.
async fn send_answer(ctx: &Ctx, _chat_name: &str, answer: &str) -> Result<()> {
    let answer = answer.trim();
    if answer.is_empty() {
        return ctx.reply("…").await;
    }
    // Telegram tops out near 4096 chars; keep a little headroom.
    let chunks = foukoapi::util::split_chunks(answer, 4000);
    for part in chunks {
        ctx.reply(&part).await?;
    }
    Ok(())
}

// -- helpers -----------------------------------------------------------------

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
        let key = if input == "-" {
            String::new()
        } else {
            input.to_owned()
        };
        let had_key = !key.is_empty();
        let host = Host {
            id: crate::ai::new_id("h"),
            name: name.to_owned(),
            base_url: url.to_owned(),
            api_key: key,
            models: Vec::new(),
        };

        // Ask the host what it serves so the user doesn't have to type
        // model names by hand. Works for Ollama, LiteLLM, LM Studio and
        // anything else with an OpenAI-style /v1/models; if the endpoint
        // isn't there, the manual `/ai model add` path still exists.
        ctx.typing().await;
        let mut host = host;
        let discovered = crate::ai::list_models(&host).await;
        let found = discovered.len();
        host.models = discovered;

        store.add_host(p, host).await?;
        store.clear_wizard(p).await?;

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

/// Chat wizard, step 2: pick a model on the chosen host.
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
    if models.is_empty() {
        return warn(ctx, svc, "ai_wiz_no_models").await;
    }
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

/// Resolve a host by name across the user's own hosts and any shared with
/// them, returning its id and whether `model` is available. Own hosts win
/// on a name clash.
async fn resolve_host_for_chat(
    store: &crate::ai::AiStore,
    user: &str,
    host_name: &str,
    model: &str,
) -> Option<(String, bool)> {
    if let Some(h) = find_host_by_name(store, user, host_name).await {
        let ok = h.models.iter().any(|m| m == model);
        return Some((h.id, ok));
    }
    for (_owner, host, models) in store.shared_hosts_for(user).await {
        if host.name == host_name {
            let ok = models.iter().any(|m| m == model);
            return Some((host.id, ok));
        }
    }
    None
}

async fn find_host_by_name(store: &crate::ai::AiStore, primary: &str, name: &str) -> Option<Host> {
    store
        .hosts(primary)
        .await
        .ok()?
        .into_iter()
        .find(|h| h.name == name)
}

async fn find_chat_by_name(store: &crate::ai::AiStore, primary: &str, name: &str) -> Option<Chat> {
    store
        .chats(primary)
        .await
        .ok()?
        .into_iter()
        .find(|c| c.name == name)
}

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
