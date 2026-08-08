//! JSON + SSE handlers behind X-Init-Data auth. Business logic lives in
//! crate::ai; this file only translates HTTP to it and back.

use super::AppState;
use crate::ai::{self, ChatMessage, GenError, ModelCaps, ToolSpec};
use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::response::sse::{Event, KeepAlive, Sse};
use axum::response::{IntoResponse, Response};
use axum::Json;
use base64::Engine as _;
use foukoapi::PlatformKind;
use serde::Deserialize;
use serde_json::json;
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Accept init data signed within the last 12 hours.
const INIT_DATA_MAX_AGE: Duration = Duration::from_secs(12 * 3600);

/// A verified caller.
struct Auth {
    user_id: i64,
    primary: String,
    lang: String,
}

/// JSON error body with a status code.
fn err(status: StatusCode, msg: &str) -> Response {
    (status, Json(json!({ "error": msg }))).into_response()
}

fn unauthorized() -> Response {
    err(StatusCode::UNAUTHORIZED, "unauthorized")
}

fn bad_request(msg: &str) -> Response {
    err(StatusCode::BAD_REQUEST, msg)
}

fn store_error() -> Response {
    err(
        StatusCode::INTERNAL_SERVER_ERROR,
        "storage error, try again",
    )
}

/// Validate the X-Init-Data header and apply the per-user request gap.
/// For mutating endpoints; reads use [`auth_read`] so a refresh right
/// after an action (or during a long stream) never trips the gate.
async fn auth(state: &AppState, headers: &HeaderMap) -> Result<Auth, Response> {
    let a = auth_read(state, headers).await?;
    if !state.rate_ok(a.user_id) {
        return Err(err(StatusCode::TOO_MANY_REQUESTS, "too many requests"));
    }
    Ok(a)
}

/// Validate the X-Init-Data header and resolve the caller to the same
/// primary identity the bot uses, so both see one dataset. No rate gate.
async fn auth_read(state: &AppState, headers: &HeaderMap) -> Result<Auth, Response> {
    let raw = headers
        .get("x-init-data")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    let data = foukoapi::webapp::validate_init_data(raw, &state.bot_token, INIT_DATA_MAX_AGE)
        .map_err(|_| unauthorized())?;
    let user = data.user.ok_or_else(unauthorized)?;
    let id = user.id.to_string();
    // Identical to the bot's `primary()`: platform prefix + user id,
    // resolved through account links.
    let primary = state
        .svc
        .accounts
        .primary_for(PlatformKind::Telegram, &id)
        .await
        .unwrap_or_else(|_| format!("{}:{id}", PlatformKind::Telegram));
    // Saved /lang wins when the user changed it; otherwise the client's
    // language decides.
    let saved = state
        .svc
        .accounts
        .lang_for(PlatformKind::Telegram, &id)
        .await
        .unwrap_or_else(|_| "en".to_owned());
    let client = user
        .language_code
        .as_deref()
        .map(|l| l.chars().take(2).collect::<String>())
        .unwrap_or_default();
    let lang = if saved != "en" {
        saved
    } else if client == "ru" {
        "ru".to_owned()
    } else {
        "en".to_owned()
    };
    Ok(Auth {
        user_id: user.id,
        primary,
        lang,
    })
}

/// Capability names of a model on a host, for the UI.
fn caps_list(host: &ai::Host, model: &str) -> Vec<&'static str> {
    let caps = host.caps_of(model);
    let mut out = Vec::new();
    if caps.image() {
        out.push("image");
    }
    if caps.video() {
        out.push("video");
    }
    if caps.audio() {
        out.push("audio");
    }
    out
}

// -- /api/state ---------------------------------------------------------------

pub async fn state(State(st): State<Arc<AppState>>, headers: HeaderMap) -> Response {
    let a = match auth_read(&st, &headers).await {
        Ok(a) => a,
        Err(r) => return r,
    };
    let store = &st.ai;
    let Ok(hosts) = store.hosts(&a.primary).await else {
        return store_error();
    };
    let Ok(chats) = store.chats(&a.primary).await else {
        return store_error();
    };
    let active = store.active_chat(&a.primary).await;
    let shared = store.shared_hosts_for(&a.primary).await;

    let host_name_of = |host_id: &str| -> String {
        hosts
            .iter()
            .find(|h| h.id == host_id)
            .map(|h| h.name.clone())
            .or_else(|| {
                shared
                    .iter()
                    .find(|(_, h, _)| h.id == host_id)
                    .map(|(_, h, _)| h.name.clone())
            })
            .unwrap_or_default()
    };

    let chats_json: Vec<_> = chats
        .iter()
        .map(|c| {
            json!({
                "name": c.name,
                "model": c.model,
                "host": host_name_of(&c.host_id),
                "active": Some(&c.id) == active.as_ref(),
                "has_prompt": !c.system_prompt.is_empty(),
            })
        })
        .collect();

    // Own hosts carry their URL; shared ones only what the owner granted.
    // API keys never leave the server.
    let mut hosts_json: Vec<_> = hosts
        .iter()
        .map(|h| {
            json!({
                "name": h.name,
                "url": h.base_url,
                "insecure": h.insecure,
                "shared": false,
                "models": h.models.iter().map(|m| json!({
                    "name": m,
                    "caps": caps_list(h, m),
                })).collect::<Vec<_>>(),
            })
        })
        .collect();
    for (_owner, h, models) in &shared {
        hosts_json.push(json!({
            "name": h.name,
            "url": "",
            "insecure": h.insecure,
            "shared": true,
            "models": models.iter().map(|m| json!({
                "name": m,
                "caps": caps_list(h, m),
            })).collect::<Vec<_>>(),
        }));
    }

    let mut gen = serde_json::Map::new();
    for cap in [ModelCaps::IMAGE, ModelCaps::VIDEO, ModelCaps::AUDIO] {
        let name = ai::cap_name(cap).unwrap_or("?");
        let pinned = match store.gen_pref(&a.primary, cap).await {
            Some((host, model)) => {
                crate::ai::tools::pinned_candidate(store, &a.primary, cap, &host, &model)
                    .await
                    .map(|_| json!({ "host": host, "model": model, "pinned": true }))
            }
            None => None,
        };
        let value = match pinned {
            Some(v) => v,
            None => match crate::ai::tools::resolve_capable_host(store, &a.primary, cap).await {
                Some((host, model)) => {
                    json!({ "host": host.name, "model": model, "pinned": false })
                }
                None => serde_json::Value::Null,
            },
        };
        gen.insert(name.to_owned(), value);
    }

    let tools = crate::ai::tools::user_tools_enabled(&st.svc, &a.primary).await;
    let voice = ai::stored_voice(&st.svc.storage, &a.primary).await;

    Json(json!({
        "chats": chats_json,
        "hosts": hosts_json,
        "gen": gen,
        "tools": tools,
        "voice": voice,
        "lang": a.lang,
        "version": env!("CARGO_PKG_VERSION"),
    }))
    .into_response()
}

// -- /api/chat/send (SSE) ------------------------------------------------------

#[derive(Deserialize)]
pub struct SendReq {
    chat: String,
    text: String,
}

/// One event pushed to the client during a streamed answer.
enum Ev {
    /// Accumulated text so far.
    Delta(String),
    /// A generated image as a data URL.
    Image(String),
    /// A machine-readable notice code the client localizes
    /// (video_queued, speech_queued).
    Notice(&'static str),
    /// Final answer text; the stream ends after this.
    Done(String),
    Error(String),
}

pub async fn chat_send(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<SendReq>,
) -> Response {
    let a = match auth(&st, &headers).await {
        Ok(a) => a,
        Err(r) => return r,
    };
    let text = req.text.trim().to_owned();
    if text.is_empty() {
        return bad_request("empty message");
    }
    if text.chars().count() > 8_000 {
        return bad_request("message too long");
    }
    let store = st.ai.clone();
    let Some(chat) = ai::find_chat_by_name(&store, &a.primary, req.chat.trim()).await else {
        return bad_request("no such chat");
    };
    let Some(host) = store
        .usable_host(&a.primary, &chat.host_id, &chat.model)
        .await
    else {
        return bad_request("host is gone or access was revoked");
    };

    // One streamed turn per chat at a time: two concurrent sends would
    // both read-modify-write the same history and lose one of the turns.
    let Some(claim) = st.claim_chat(&a.primary, &chat.id) else {
        return err(StatusCode::CONFLICT, "an answer is already streaming");
    };

    // The same per-user cooldown the bot's talk() uses.
    let uid = a.user_id.to_string();
    let wait = st
        .svc
        .econ
        .cooldown_remaining(PlatformKind::Telegram, &uid, "ai_say", 5)
        .await;
    if wait > 0 {
        return err(StatusCode::TOO_MANY_REQUESTS, "slow down");
    }
    if st
        .svc
        .econ
        .touch_cooldown(PlatformKind::Telegram, &uid, "ai_say")
        .await
        .is_err()
    {
        return store_error();
    }

    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<Ev>();
    let st2 = Arc::clone(&st);
    tokio::spawn(async move {
        run_chat_turn(st2, a, chat, host, text, tx).await;
        // Free the chat only after the turn fully settled.
        drop(claim);
    });

    let stream = futures::stream::poll_fn(move |cx| {
        rx.poll_recv(cx).map(|opt| {
            opt.map(|ev| {
                let event = match ev {
                    Ev::Delta(t) => Event::default().event("delta").data(escape_sse(&t)),
                    Ev::Image(url) => Event::default().event("image").data(url),
                    Ev::Notice(code) => Event::default().event("notice").data(code),
                    Ev::Done(t) => Event::default().event("done").data(escape_sse(&t)),
                    Ev::Error(e) => Event::default().event("error").data(escape_sse(&e)),
                };
                Ok::<_, std::convert::Infallible>(event)
            })
        })
    });
    Sse::new(stream)
        .keep_alive(KeepAlive::default())
        .into_response()
}

/// SSE data can't hold raw newlines in one data: line; ship JSON strings.
fn escape_sse(text: &str) -> String {
    serde_json::to_string(text).unwrap_or_else(|_| "\"\"".to_owned())
}

/// The web version of talk(): stream the model's answer, execute image
/// tool calls inline, queue video/speech to the bot's DM, persist both
/// turns to the same history the bot reads.
async fn run_chat_turn(
    st: Arc<AppState>,
    a: Auth,
    chat: ai::Chat,
    host: ai::Host,
    text: String,
    tx: tokio::sync::mpsc::UnboundedSender<Ev>,
) {
    let store = &st.ai;
    let p = &a.primary;

    let mut messages: Vec<ChatMessage> = Vec::new();
    if !chat.system_prompt.is_empty() {
        messages.push(ChatMessage {
            role: "system".to_owned(),
            content: chat.system_prompt.clone(),
            ts: None,
        });
    }
    messages.extend(store.history(p, &chat.id).await.unwrap_or_default());
    messages.push(ChatMessage {
        role: "user".to_owned(),
        content: text.clone(),
        ts: None,
    });

    let mut tools: Vec<ToolSpec> = if crate::ai::tools::user_tools_enabled(&st.svc, p).await
        && !crate::ai::tools::host_unsupported(&host.base_url)
    {
        crate::ai::tools::specs_for(store, p).await
    } else {
        Vec::new()
    };

    // Same loop shape as the bot: stream, run calls, feed results back.
    const MAX_TOOL_ROUNDS: usize = 3;
    let mut extra: Vec<foukoapi::genai::ChatMessage> = Vec::new();
    let mut markers: Vec<&'static str> = Vec::new();
    let mut round = 0usize;
    let answer = loop {
        let offered: &[ToolSpec] = if round < MAX_TOOL_ROUNDS { &tools } else { &[] };
        let result = stream_once(&host, &chat.model, &messages, &extra, offered, &tx).await;
        let outcome = match result {
            Ok(o) => o,
            Err(GenError::NotSupported) if !offered.is_empty() => {
                crate::ai::tools::mark_host_unsupported(&host.base_url);
                tools.clear();
                match stream_once(&host, &chat.model, &messages, &extra, &[], &tx).await {
                    Ok(o) => o,
                    Err(e) => {
                        let _ = tx.send(Ev::Error(e.to_string()));
                        return;
                    }
                }
            }
            Err(e) => {
                let _ = tx.send(Ev::Error(e.to_string()));
                return;
            }
        };

        if outcome.tool_calls.is_empty() || round >= MAX_TOOL_ROUNDS {
            break outcome.text;
        }

        let mut echo = foukoapi::genai::ChatMessage::assistant_tool_calls(&outcome.tool_calls);
        echo.content = outcome.text.clone();
        extra.push(echo);
        for call in &outcome.tool_calls {
            let (result_text, marker) = run_tool_call(&st, &a, call, &tx).await;
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

    // One history for the bot and the app.
    let _ = store
        .push_history(
            p,
            &chat.id,
            ChatMessage {
                role: "user".to_owned(),
                content: text,
                ts: None,
            },
        )
        .await;
    let _ = store
        .push_history(
            p,
            &chat.id,
            ChatMessage {
                role: "assistant".to_owned(),
                content: format!("{answer}{}", markers.concat()),
                ts: None,
            },
        )
        .await;
    let _ = tx.send(Ev::Done(answer));
}

/// One streamed request with throttled delta events.
async fn stream_once(
    host: &ai::Host,
    model: &str,
    messages: &[ChatMessage],
    extra: &[foukoapi::genai::ChatMessage],
    tools: &[ToolSpec],
    tx: &tokio::sync::mpsc::UnboundedSender<Ev>,
) -> Result<ai::ChatOutcome, GenError> {
    let tx2 = tx.clone();
    let mut last_flush = Instant::now();
    ai::chat_completion_stream_tools(host, model, messages, extra, None, tools, move |sofar| {
        // Full accumulated text every 150ms; the final text arrives via
        // the done event anyway.
        if last_flush.elapsed() >= Duration::from_millis(150) {
            last_flush = Instant::now();
            let _ = tx2.send(Ev::Delta(sofar.to_owned()));
        }
    })
    .await
}

/// Run one model-issued tool call from the web. Images come back inline
/// as data URLs; video and speech run in the background and land in the
/// user's Telegram DM from the bot (they take minutes and don't fit an
/// open SSE stream well).
async fn run_tool_call(
    st: &Arc<AppState>,
    a: &Auth,
    call: &ai::ToolCall,
    tx: &tokio::sync::mpsc::UnboundedSender<Ev>,
) -> (String, Option<&'static str>) {
    let Ok(args) = serde_json::from_str::<serde_json::Value>(&call.arguments) else {
        return ("invalid arguments".to_owned(), None);
    };
    let field = |name: &str| -> Option<String> {
        args.get(name)
            .and_then(|v| v.as_str())
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(|s| s.chars().take(2_000).collect())
    };
    let store = &st.ai;
    let p = &a.primary;
    match call.name.as_str() {
        "generate_image" => {
            let Some(prompt) = field("prompt") else {
                return ("invalid arguments".to_owned(), None);
            };
            let Some((host, model)) =
                crate::ai::tools::resolve_capable_host(store, p, ModelCaps::IMAGE).await
            else {
                return ("failed: no capable model available".to_owned(), None);
            };
            if !gen_cooldown(st, a, "ai_draw", 30).await {
                return ("rate limited, ask the user to wait".to_owned(), None);
            }
            match ai::image_generation(&host, &model, &prompt).await {
                Ok(bytes) => {
                    let _ = tx.send(Ev::Image(image_data_url(&bytes)));
                    (
                        "image generated and shown to the user".to_owned(),
                        Some(" [generated image]"),
                    )
                }
                Err(e) => (format!("failed: {e}"), None),
            }
        }
        "generate_video" => {
            let Some(prompt) = field("prompt") else {
                return ("invalid arguments".to_owned(), None);
            };
            if !gen_cooldown(st, a, "ai_video", 120).await {
                return ("rate limited, ask the user to wait".to_owned(), None);
            }
            let _ = tx.send(Ev::Notice("video_queued"));
            spawn_media_job(st, a, MediaKind::Video, prompt, None);
            (
                "video generation was queued in the background - it may still fail. \
                 Do not claim it succeeded; tell the user it is being prepared and \
                 the bot will DM either the video or an error"
                    .to_owned(),
                None,
            )
        }
        "speak" => {
            let Some(text) = field("text") else {
                return ("invalid arguments".to_owned(), None);
            };
            if !gen_cooldown(st, a, "ai_speak", 30).await {
                return ("rate limited, ask the user to wait".to_owned(), None);
            }
            let voice = match field("voice") {
                Some(v) => Some(v.to_ascii_lowercase()),
                None => ai::stored_voice(&st.svc.storage, p).await,
            };
            let _ = tx.send(Ev::Notice("speech_queued"));
            spawn_media_job(st, a, MediaKind::Speech, text, voice);
            (
                "speech synthesis was queued in the background - it may still fail. \
                 Do not claim it succeeded; tell the user it is being prepared and \
                 the bot will DM either the audio or an error"
                    .to_owned(),
                None,
            )
        }
        _ => ("unknown tool".to_owned(), None),
    }
}

/// Same cooldown keys the bot commands use, so the model can't route
/// around them by switching surfaces. `true` when the action may run.
async fn gen_cooldown(st: &Arc<AppState>, a: &Auth, key: &str, secs: i64) -> bool {
    let uid = a.user_id.to_string();
    let wait = st
        .svc
        .econ
        .cooldown_remaining(PlatformKind::Telegram, &uid, key, secs)
        .await;
    if wait > 0 {
        return false;
    }
    st.svc
        .econ
        .touch_cooldown(PlatformKind::Telegram, &uid, key)
        .await
        .is_ok()
}

enum MediaKind {
    Video,
    Speech,
}

/// Generate video/speech in the background and DM the result through the
/// bot, the way reminders are delivered. Failures are DM'd too, so a
/// silent job never leaves the user guessing.
fn spawn_media_job(
    st: &Arc<AppState>,
    a: &Auth,
    kind: MediaKind,
    prompt: String,
    voice: Option<String>,
) {
    let st = Arc::clone(st);
    let user_id = a.user_id.to_string();
    let primary = a.primary.clone();
    tokio::spawn(async move {
        let cap = match kind {
            MediaKind::Video => ModelCaps::VIDEO,
            MediaKind::Speech => ModelCaps::AUDIO,
        };
        let Some((host, model)) =
            crate::ai::tools::resolve_capable_host(&st.ai, &primary, cap).await
        else {
            let _ = st
                .svc
                .notifier
                .send_dm(
                    PlatformKind::Telegram,
                    user_id,
                    foukoapi::Reply::text("Generation failed: no capable model available."),
                )
                .await;
            return;
        };
        let result = match kind {
            MediaKind::Video => ai::video_generation(&host, &model, &prompt)
                .await
                .map(|bytes| {
                    let caption: String = prompt.chars().take(200).collect();
                    foukoapi::Reply::text(caption).video_bytes(bytes, "video.mp4")
                }),
            MediaKind::Speech => ai::speech_generation(&host, &model, &prompt, voice.as_deref())
                .await
                .map(|bytes| foukoapi::Reply::text("").audio_bytes(bytes, "speech.mp3")),
        };
        let reply = match result {
            Ok(r) => r,
            Err(e) => foukoapi::Reply::text(format!("Generation failed: {e}")),
        };
        if let Err(e) = st
            .svc
            .notifier
            .send_dm(PlatformKind::Telegram, user_id, reply)
            .await
        {
            tracing::warn!(error = %e, "webapp: media DM delivery failed");
        }
    });
}

/// Wrap image bytes into a data URL for inline display.
fn image_data_url(bytes: &[u8]) -> String {
    let mime = ai::image_mime(bytes);
    let b64 = base64::engine::general_purpose::STANDARD.encode(bytes);
    format!("data:{mime};base64,{b64}")
}

// -- chat management ------------------------------------------------------------

#[derive(Deserialize)]
pub struct ChatNewReq {
    name: String,
    host: String,
    model: String,
}

pub async fn chat_new(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<ChatNewReq>,
) -> Response {
    let a = match auth(&st, &headers).await {
        Ok(a) => a,
        Err(r) => return r,
    };
    let name = req.name.trim();
    if name.is_empty() || name.chars().count() > 64 {
        return bad_request("bad chat name");
    }
    if ai::find_chat_by_name(&st.ai, &a.primary, name)
        .await
        .is_some()
    {
        return bad_request("a chat with that name already exists");
    }
    match ai::create_chat(&st.ai, &a.primary, name, req.host.trim(), req.model.trim()).await {
        Ok(Ok(_)) => Json(json!({ "ok": true })).into_response(),
        Ok(Err(ai::ChatCreateError::HostMissing)) => bad_request("no such host"),
        Ok(Err(ai::ChatCreateError::ModelMissing)) => bad_request("no such model on that host"),
        Err(_) => store_error(),
    }
}

#[derive(Deserialize)]
pub struct NameReq {
    name: String,
}

pub async fn chat_use(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<NameReq>,
) -> Response {
    let a = match auth(&st, &headers).await {
        Ok(a) => a,
        Err(r) => return r,
    };
    let Some(chat) = ai::find_chat_by_name(&st.ai, &a.primary, req.name.trim()).await else {
        return bad_request("no such chat");
    };
    match st.ai.set_active_chat(&a.primary, &chat.id).await {
        Ok(()) => Json(json!({ "ok": true })).into_response(),
        Err(_) => store_error(),
    }
}

pub async fn chat_del(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<NameReq>,
) -> Response {
    let a = match auth(&st, &headers).await {
        Ok(a) => a,
        Err(r) => return r,
    };
    let Some(chat) = ai::find_chat_by_name(&st.ai, &a.primary, req.name.trim()).await else {
        return bad_request("no such chat");
    };
    match st.ai.remove_chat(&a.primary, &chat.id).await {
        Ok(()) => Json(json!({ "ok": true })).into_response(),
        Err(_) => store_error(),
    }
}

pub async fn chat_clear(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<NameReq>,
) -> Response {
    let a = match auth(&st, &headers).await {
        Ok(a) => a,
        Err(r) => return r,
    };
    let Some(chat) = ai::find_chat_by_name(&st.ai, &a.primary, req.name.trim()).await else {
        return bad_request("no such chat");
    };
    match st.ai.clear_history(&a.primary, &chat.id).await {
        Ok(()) => Json(json!({ "ok": true })).into_response(),
        Err(_) => store_error(),
    }
}

#[derive(Deserialize)]
pub struct PromptReq {
    name: String,
    prompt: String,
}

pub async fn chat_prompt(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<PromptReq>,
) -> Response {
    let a = match auth(&st, &headers).await {
        Ok(a) => a,
        Err(r) => return r,
    };
    let Ok(mut chats) = st.ai.chats(&a.primary).await else {
        return store_error();
    };
    let name = req.name.trim();
    let Some(chat) = chats.iter_mut().find(|c| c.name == name) else {
        return bad_request("no such chat");
    };
    // Same cap as /ai prompt.
    chat.system_prompt = req.prompt.trim().chars().take(4_000).collect();
    match st.ai.set_chats(&a.primary, &chats).await {
        Ok(()) => Json(json!({ "ok": true })).into_response(),
        Err(_) => store_error(),
    }
}

#[derive(Deserialize)]
pub struct ChatModelReq {
    name: String,
    model: String,
}

/// Switch a chat to another model. On an own host any name goes - the
/// user could register it by hand anyway; on a family-shared host only
/// the granted list is allowed. History survives the switch.
pub async fn chat_model(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<ChatModelReq>,
) -> Response {
    let a = match auth(&st, &headers).await {
        Ok(a) => a,
        Err(r) => return r,
    };
    let model = req.model.trim();
    if model.is_empty() || model.chars().count() > 128 {
        return bad_request("bad model name");
    }
    let Some(chat) = ai::find_chat_by_name(&st.ai, &a.primary, req.name.trim()).await else {
        return bad_request("no such chat");
    };
    if st.ai.host(&a.primary, &chat.host_id).await.is_none() {
        let Some(allowed) = st
            .ai
            .shared_hosts_for(&a.primary)
            .await
            .into_iter()
            .find(|(_, h, _)| h.id == chat.host_id)
            .map(|(_, _, models)| models)
        else {
            return bad_request("host is gone or access was revoked");
        };
        if !allowed.iter().any(|m| m == model) {
            return bad_request("that model is not shared with you");
        }
    }
    let Ok(mut chats) = st.ai.chats(&a.primary).await else {
        return store_error();
    };
    let Some(c) = chats.iter_mut().find(|c| c.id == chat.id) else {
        return bad_request("no such chat");
    };
    c.model = model.to_owned();
    match st.ai.set_chats(&a.primary, &chats).await {
        Ok(()) => Json(json!({ "ok": true })).into_response(),
        Err(_) => store_error(),
    }
}

pub async fn chat_history(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<NameReq>,
) -> Response {
    let a = match auth_read(&st, &headers).await {
        Ok(a) => a,
        Err(r) => return r,
    };
    let Some(chat) = ai::find_chat_by_name(&st.ai, &a.primary, req.name.trim()).await else {
        return bad_request("no such chat");
    };
    let Ok(history) = st.ai.history(&a.primary, &chat.id).await else {
        return store_error();
    };
    // Stored history is already capped at 40 turns, under the 50 asked.
    let msgs: Vec<_> = history
        .iter()
        .map(|m| json!({ "role": m.role, "content": m.content, "ts": m.ts }))
        .collect();
    Json(json!({ "prompt": chat.system_prompt, "messages": msgs })).into_response()
}

// -- host management ------------------------------------------------------------

#[derive(Deserialize)]
pub struct HostAddReq {
    name: String,
    url: String,
    #[serde(default)]
    key: String,
}

pub async fn host_add(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<HostAddReq>,
) -> Response {
    let a = match auth(&st, &headers).await {
        Ok(a) => a,
        Err(r) => return r,
    };
    let name = req.name.trim();
    let url = req.url.trim();
    if name.is_empty() || name.contains(char::is_whitespace) || name.chars().count() > 64 {
        return bad_request("bad host name");
    }
    if !(url.starts_with("http://") || url.starts_with("https://")) {
        return bad_request("URL must start with http:// or https://");
    }
    if ai::find_host_by_name(&st.ai, &a.primary, name)
        .await
        .is_some()
    {
        return bad_request("a host with that name already exists");
    }
    match ai::add_host_discovering(&st.ai, &a.primary, name, url, req.key.trim()).await {
        Ok((found, discover_err)) => Json(json!({
            "ok": true,
            "models": found,
            "discover_error": discover_err,
        }))
        .into_response(),
        Err(_) => store_error(),
    }
}

pub async fn host_del(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<NameReq>,
) -> Response {
    let a = match auth(&st, &headers).await {
        Ok(a) => a,
        Err(r) => return r,
    };
    let Some(host) = ai::find_host_by_name(&st.ai, &a.primary, req.name.trim()).await else {
        return bad_request("no such host");
    };
    match st.ai.remove_host(&a.primary, &host.id).await {
        Ok(()) => Json(json!({ "ok": true })).into_response(),
        Err(_) => store_error(),
    }
}

pub async fn host_refresh(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<NameReq>,
) -> Response {
    let a = match auth(&st, &headers).await {
        Ok(a) => a,
        Err(r) => return r,
    };
    let Some(host) = ai::find_host_by_name(&st.ai, &a.primary, req.name.trim()).await else {
        return bad_request("no such host");
    };
    match ai::refresh_host_models(&st.ai, &a.primary, &host).await {
        Ok(Ok(found)) => Json(json!({ "ok": true, "models": found })).into_response(),
        Ok(Err(reason)) => bad_request(&format!("discovery failed: {reason}")),
        Err(_) => store_error(),
    }
}

#[derive(Deserialize)]
pub struct InsecureReq {
    name: String,
    on: bool,
}

pub async fn host_insecure(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<InsecureReq>,
) -> Response {
    let a = match auth(&st, &headers).await {
        Ok(a) => a,
        Err(r) => return r,
    };
    match ai::set_host_insecure(&st.ai, &a.primary, req.name.trim(), req.on).await {
        Ok(true) => Json(json!({ "ok": true })).into_response(),
        Ok(false) => bad_request("no such host"),
        Err(_) => store_error(),
    }
}

// -- model management -----------------------------------------------------------

#[derive(Deserialize)]
pub struct ModelReq {
    host: String,
    model: String,
}

/// Map a model add/del/tag/untag outcome onto an HTTP response.
fn model_edit_response(outcome: foukoapi::Result<ai::ModelEdit>) -> Response {
    match outcome {
        Ok(ai::ModelEdit::Done) => Json(json!({ "ok": true })).into_response(),
        Ok(ai::ModelEdit::HostMissing) => bad_request("no such host"),
        Ok(ai::ModelEdit::ModelMissing) => bad_request("no such model on that host"),
        Err(_) => store_error(),
    }
}

pub async fn model_add(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<ModelReq>,
) -> Response {
    let a = match auth(&st, &headers).await {
        Ok(a) => a,
        Err(r) => return r,
    };
    let model = req.model.trim();
    if model.is_empty() || model.chars().count() > 128 {
        return bad_request("bad model name");
    }
    model_edit_response(ai::add_model(&st.ai, &a.primary, req.host.trim(), model).await)
}

pub async fn model_del(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<ModelReq>,
) -> Response {
    let a = match auth(&st, &headers).await {
        Ok(a) => a,
        Err(r) => return r,
    };
    model_edit_response(ai::del_model(&st.ai, &a.primary, req.host.trim(), req.model.trim()).await)
}

#[derive(Deserialize)]
pub struct TagReq {
    host: String,
    model: String,
    cap: String,
}

pub async fn model_tag(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<TagReq>,
) -> Response {
    let a = match auth(&st, &headers).await {
        Ok(a) => a,
        Err(r) => return r,
    };
    let Some(cap) = ai::cap_from_arg(&req.cap) else {
        return bad_request("unknown capability");
    };
    model_edit_response(
        ai::tag_model(&st.ai, &a.primary, req.host.trim(), req.model.trim(), cap).await,
    )
}

pub async fn model_untag(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<ModelReq>,
) -> Response {
    let a = match auth(&st, &headers).await {
        Ok(a) => a,
        Err(r) => return r,
    };
    model_edit_response(
        ai::untag_model(&st.ai, &a.primary, req.host.trim(), req.model.trim()).await,
    )
}

// -- settings ---------------------------------------------------------------------

#[derive(Deserialize)]
pub struct GenSetReq {
    cap: String,
    #[serde(default)]
    host: String,
    model: String,
}

pub async fn gen_set(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<GenSetReq>,
) -> Response {
    let a = match auth(&st, &headers).await {
        Ok(a) => a,
        Err(r) => return r,
    };
    let Some(cap) = ai::cap_from_arg(&req.cap) else {
        return bad_request("unknown capability");
    };
    if req.model.trim().eq_ignore_ascii_case("auto") {
        return match st.ai.clear_gen_pref(&a.primary, cap).await {
            Ok(()) => Json(json!({ "ok": true })).into_response(),
            Err(_) => store_error(),
        };
    }
    let host = req.host.trim();
    let model = req.model.trim();
    if crate::ai::tools::pinned_candidate(&st.ai, &a.primary, cap, host, model)
        .await
        .is_none()
    {
        return bad_request("that host/model can't produce this");
    }
    match st.ai.set_gen_pref(&a.primary, cap, host, model).await {
        Ok(()) => Json(json!({ "ok": true })).into_response(),
        Err(_) => store_error(),
    }
}

#[derive(Deserialize)]
pub struct ToolsSetReq {
    on: bool,
}

pub async fn tools_set(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<ToolsSetReq>,
) -> Response {
    let a = match auth(&st, &headers).await {
        Ok(a) => a,
        Err(r) => return r,
    };
    match crate::ai::tools::set_user_tools(&st.svc, &a.primary, req.on).await {
        Ok(()) => Json(json!({ "ok": true })).into_response(),
        Err(_) => store_error(),
    }
}

#[derive(Deserialize)]
pub struct VoiceSetReq {
    voice: String,
}

pub async fn voice_set(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<VoiceSetReq>,
) -> Response {
    let a = match auth(&st, &headers).await {
        Ok(a) => a,
        Err(r) => return r,
    };
    let voice = req.voice.trim().to_ascii_lowercase();
    if voice.chars().count() > 32 {
        return bad_request("bad voice name");
    }
    match ai::set_stored_voice(&st.svc.storage, &a.primary, &voice).await {
        Ok(()) => Json(json!({ "ok": true })).into_response(),
        Err(_) => store_error(),
    }
}

// -- /api/draw ----------------------------------------------------------------------

#[derive(Deserialize)]
pub struct DrawReq {
    prompt: String,
}

pub async fn draw(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<DrawReq>,
) -> Response {
    let a = match auth(&st, &headers).await {
        Ok(a) => a,
        Err(r) => return r,
    };
    let prompt = req.prompt.trim();
    if prompt.is_empty() {
        return bad_request("empty prompt");
    }
    if prompt.chars().count() > 2_000 {
        return bad_request("prompt too long");
    }
    let Some((host, model)) =
        crate::ai::tools::resolve_capable_host(&st.ai, &a.primary, ModelCaps::IMAGE).await
    else {
        return bad_request("no image model available");
    };
    if !gen_cooldown(&st, &a, "ai_draw", 30).await {
        return err(StatusCode::TOO_MANY_REQUESTS, "slow down");
    }
    match ai::image_generation(&host, &model, prompt).await {
        Ok(bytes) => Json(json!({ "image": image_data_url(&bytes) })).into_response(),
        Err(e) => err(StatusCode::BAD_GATEWAY, &format!("generation failed: {e}")),
    }
}
