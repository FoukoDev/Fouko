//! Telegram Mini App backend: a small axum server exposing the whole /ai
//! feature over JSON + SSE. Enabled when WEBAPP_URL is set (the public
//! https URL behind a reverse proxy), listening on WEBAPP_BIND.
//!
//! Every /api request must carry an X-Init-Data header with the Mini App's
//! signed init data; the signature check maps it to the same primary
//! identity the bot uses, so the app and the bot see one dataset.

mod api;

use crate::ai::AiStore;
use crate::commands::Services;
use axum::extract::DefaultBodyLimit;
use axum::http::{header, HeaderValue};
use axum::response::Response;
use axum::routing::{get, post};
use axum::Router;
use std::collections::{HashMap, HashSet};
use std::net::SocketAddr;
use std::sync::{Arc, Mutex, PoisonError};
use std::time::{Duration, Instant};

/// Reject api bodies over this size. Prompts are capped far lower anyway.
const MAX_BODY_BYTES: usize = 256 * 1024;

/// Minimal gap between two mutating api requests from one user. Reads
/// (state, history) are exempt, so a refresh right after an action never
/// trips the gate.
const MIN_REQUEST_GAP: Duration = Duration::from_millis(300);

/// Everything the api handlers share.
pub struct AppState {
    pub svc: Services,
    pub ai: AiStore,
    pub bot_token: String,
    /// Per-user flood gate: last request time by Telegram user id.
    last_hit: Mutex<HashMap<i64, Instant>>,
    /// Chats with a streamed answer in flight, as "primary:chat_id".
    /// A second send into the same chat would race the history write.
    busy_chats: Mutex<HashSet<String>>,
}

impl AppState {
    /// One request per [`MIN_REQUEST_GAP`] per user; `false` means "too
    /// fast, go away". The map is pruned when it grows past a sane size.
    pub fn rate_ok(&self, user_id: i64) -> bool {
        let mut map = self.last_hit.lock().unwrap_or_else(PoisonError::into_inner);
        let now = Instant::now();
        if map.len() > 4096 {
            map.retain(|_, t| now.duration_since(*t) < Duration::from_secs(60));
        }
        match map.get(&user_id) {
            Some(t) if now.duration_since(*t) < MIN_REQUEST_GAP => false,
            _ => {
                map.insert(user_id, now);
                true
            }
        }
    }

    /// Mark a chat as having a streamed turn in flight. `None` when one is
    /// already running there; the returned guard frees the slot on drop.
    pub fn claim_chat(self: &Arc<Self>, primary: &str, chat_id: &str) -> Option<ChatClaim> {
        let key = format!("{primary}:{chat_id}");
        let mut busy = self
            .busy_chats
            .lock()
            .unwrap_or_else(PoisonError::into_inner);
        if !busy.insert(key.clone()) {
            return None;
        }
        Some(ChatClaim {
            state: Arc::clone(self),
            key,
        })
    }
}

/// RAII marker for a chat turn in flight; dropping it frees the chat.
pub struct ChatClaim {
    state: Arc<AppState>,
    key: String,
}

impl Drop for ChatClaim {
    fn drop(&mut self) {
        self.state
            .busy_chats
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .remove(&self.key);
    }
}

/// Run the Mini App server until the process exits. Spawned from main
/// when WEBAPP_URL is set, the AI feature is on and a Telegram token
/// exists (init data is signed with it).
pub async fn serve(svc: Services, ai: AiStore, bot_token: String, bind: SocketAddr) {
    let state = Arc::new(AppState {
        svc,
        ai,
        bot_token,
        last_hit: Mutex::new(HashMap::new()),
        busy_chats: Mutex::new(HashSet::new()),
    });

    let app = Router::new()
        .route("/", get(index_html))
        .route("/app.js", get(app_js))
        .route("/style.css", get(style_css))
        .route("/api/state", post(api::state))
        .route("/api/chat/send", post(api::chat_send))
        .route("/api/chat/new", post(api::chat_new))
        .route("/api/chat/use", post(api::chat_use))
        .route("/api/chat/del", post(api::chat_del))
        .route("/api/chat/clear", post(api::chat_clear))
        .route("/api/chat/prompt", post(api::chat_prompt))
        .route("/api/chat/model", post(api::chat_model))
        .route("/api/chat/history", post(api::chat_history))
        .route("/api/host/add", post(api::host_add))
        .route("/api/host/del", post(api::host_del))
        .route("/api/host/refresh", post(api::host_refresh))
        .route("/api/host/insecure", post(api::host_insecure))
        .route("/api/model/add", post(api::model_add))
        .route("/api/model/del", post(api::model_del))
        .route("/api/model/tag", post(api::model_tag))
        .route("/api/model/untag", post(api::model_untag))
        .route("/api/gen/set", post(api::gen_set))
        .route("/api/tools/set", post(api::tools_set))
        .route("/api/voice/set", post(api::voice_set))
        .route("/api/draw", post(api::draw))
        .layer(DefaultBodyLimit::max(MAX_BODY_BYTES))
        .layer(axum::middleware::map_response(security_headers))
        .with_state(state);

    let listener = match tokio::net::TcpListener::bind(bind).await {
        Ok(l) => l,
        Err(e) => {
            tracing::error!(addr = %bind, error = %e, "webapp: bind failed");
            return;
        }
    };
    tracing::info!(addr = %bind, "webapp server listening");
    if let Err(e) = axum::serve(listener, app).await {
        tracing::error!(error = %e, "webapp server stopped");
    }
}

/// CSP allows only our own origin plus telegram.org for the Mini App SDK
/// script; inline styles stay allowed for the theme variables Telegram
/// injects.
async fn security_headers(mut resp: Response) -> Response {
    let headers = resp.headers_mut();
    headers.insert(
        header::CONTENT_SECURITY_POLICY,
        HeaderValue::from_static(
            "default-src 'self'; script-src 'self' https://telegram.org; \
             style-src 'self' 'unsafe-inline'; connect-src 'self'",
        ),
    );
    headers.insert(
        header::X_CONTENT_TYPE_OPTIONS,
        HeaderValue::from_static("nosniff"),
    );
    resp
}

async fn index_html() -> ([(header::HeaderName, &'static str); 1], &'static str) {
    (
        [(header::CONTENT_TYPE, "text/html; charset=utf-8")],
        include_str!("static/index.html"),
    )
}

async fn app_js() -> ([(header::HeaderName, &'static str); 1], &'static str) {
    (
        [(header::CONTENT_TYPE, "text/javascript; charset=utf-8")],
        include_str!("static/app.js"),
    )
}

async fn style_css() -> ([(header::HeaderName, &'static str); 1], &'static str) {
    (
        [(header::CONTENT_TYPE, "text/css; charset=utf-8")],
        include_str!("static/style.css"),
    )
}
