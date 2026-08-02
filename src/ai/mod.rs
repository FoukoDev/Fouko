//! Private LLM integration: user-owned hosts, models, chats and history.
//!
//! Everything here is stored **encrypted** (via [`foukoapi::Secret`]) and
//! keyed off a user's *primary* identity, so it follows an account link
//! across platforms and never sits in the database as plaintext. The bot
//! only ever sends the model the user's own system prompt and their chat
//! history - no information about the bot, the user, or anyone else.

use foukoapi::{AnyStorage, Result, Secret};
use serde::{Deserialize, Serialize};

pub mod command;

/// Storage-key prefixes. Values under these are ciphertext.
const HOSTS_PREFIX: &str = "foukobot:ai:hosts:";
const CHATS_PREFIX: &str = "foukobot:ai:chats:";
const HISTORY_PREFIX: &str = "foukobot:ai:history:";
const ACTIVE_PREFIX: &str = "foukobot:ai:active:";
const WIZARD_PREFIX: &str = "foukobot:ai:wizard:";
const SHARES_PREFIX: &str = "foukobot:ai:shares:";
const SHARED_WITH_PREFIX: &str = "foukobot:ai:sharedwith:";
const PENDING_PREFIX: &str = "foukobot:ai:share_pending:";

/// How many past messages of a chat we keep and send as context.
pub const HISTORY_LIMIT: usize = 40;

/// An OpenAI-compatible endpoint the user added. `api_key` may be empty for
/// local servers (Ollama, LM Studio) that don't require one.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Host {
    pub id: String,
    pub name: String,
    pub base_url: String,
    #[serde(default)]
    pub api_key: String,
    /// Models the user registered on this host.
    #[serde(default)]
    pub models: Vec<String>,
}

/// A named conversation bound to a host + model, with its own system prompt.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chat {
    pub id: String,
    pub name: String,
    pub host_id: String,
    pub model: String,
    #[serde(default)]
    pub system_prompt: String,
}

/// One turn in a conversation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    /// `"system"`, `"user"` or `"assistant"`.
    pub role: String,
    pub content: String,
}

/// A grant of "family access": another user may use some of my hosts/models.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Share {
    /// Primary identity of the person granted access.
    pub grantee: String,
    /// Which of the owner's host ids are shared, and which of their models.
    #[serde(default)]
    pub hosts: Vec<SharedHost>,
}

/// The subset of a host exposed to a grantee.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SharedHost {
    pub host_id: String,
    /// Allowed model names. Empty means "no models" (effectively revoked).
    #[serde(default)]
    pub models: Vec<String>,
}

/// A pending family-access invitation waiting on the grantee's answer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingShare {
    /// Owner's primary identity.
    pub owner: String,
    /// What's being offered.
    pub hosts: Vec<SharedHost>,
    /// Unix timestamp when the invite was created (for the 24h expiry).
    pub created_at: i64,
}

/// Encrypted, per-user storage for the whole AI feature.
#[derive(Clone)]
pub struct AiStore {
    storage: AnyStorage,
    secret: Secret,
}

impl AiStore {
    pub fn new(storage: AnyStorage, secret: Secret) -> Self {
        Self { storage, secret }
    }

    // -- low-level encrypted JSON helpers -----------------------------------

    /// Read and decode an encrypted list. A missing record is a normal
    /// empty state (`Ok(vec![])`); a record that exists but fails to
    /// decrypt or parse is an error. Collapsing the two used to be a data
    /// eraser: a transient read failure looked like "no hosts", and the
    /// next read-modify-write persisted that emptiness.
    async fn load<T>(&self, key: &str) -> Result<Vec<T>>
    where
        T: for<'de> Deserialize<'de>,
    {
        let Some(sealed) = self.storage.get(key).await? else {
            return Ok(Vec::new());
        };
        let plain = self.secret.decrypt(&sealed)?;
        serde_json::from_str(&plain)
            .map_err(|e| foukoapi::Error::Other(format!("deserialize: {e}")))
    }

    async fn store<T: Serialize>(&self, key: &str, value: &[T]) -> Result<()> {
        let plain = serde_json::to_string(value)
            .map_err(|e| foukoapi::Error::Other(format!("serialize: {e}")))?;
        let sealed = self.secret.encrypt(&plain)?;
        self.storage.set(key, &sealed).await
    }

    // -- hosts --------------------------------------------------------------

    pub async fn hosts(&self, primary: &str) -> Result<Vec<Host>> {
        self.load(&format!("{HOSTS_PREFIX}{primary}")).await
    }

    pub async fn set_hosts(&self, primary: &str, hosts: &[Host]) -> Result<()> {
        self.store(&format!("{HOSTS_PREFIX}{primary}"), hosts).await
    }

    pub async fn add_host(&self, primary: &str, host: Host) -> Result<()> {
        let mut hosts = self.hosts(primary).await?;
        hosts.push(host);
        self.set_hosts(primary, &hosts).await
    }

    pub async fn remove_host(&self, primary: &str, host_id: &str) -> Result<()> {
        let mut hosts = self.hosts(primary).await?;
        hosts.retain(|h| h.id != host_id);
        self.set_hosts(primary, &hosts).await
    }

    pub async fn host(&self, primary: &str, host_id: &str) -> Option<Host> {
        self.hosts(primary)
            .await
            .ok()?
            .into_iter()
            .find(|h| h.id == host_id)
    }

    // -- chats --------------------------------------------------------------

    pub async fn chats(&self, primary: &str) -> Result<Vec<Chat>> {
        self.load(&format!("{CHATS_PREFIX}{primary}")).await
    }

    pub async fn set_chats(&self, primary: &str, chats: &[Chat]) -> Result<()> {
        self.store(&format!("{CHATS_PREFIX}{primary}"), chats).await
    }

    pub async fn add_chat(&self, primary: &str, chat: Chat) -> Result<()> {
        let mut chats = self.chats(primary).await?;
        chats.push(chat);
        self.set_chats(primary, &chats).await
    }

    pub async fn chat(&self, primary: &str, chat_id: &str) -> Option<Chat> {
        self.chats(primary)
            .await
            .ok()?
            .into_iter()
            .find(|c| c.id == chat_id)
    }

    pub async fn remove_chat(&self, primary: &str, chat_id: &str) -> Result<()> {
        let mut chats = self.chats(primary).await?;
        chats.retain(|c| c.id != chat_id);
        self.set_chats(primary, &chats).await?;
        // Drop its history too.
        let _ = self
            .storage
            .del(&format!("{HISTORY_PREFIX}{primary}:{chat_id}"))
            .await;
        Ok(())
    }

    // -- active chat --------------------------------------------------------

    pub async fn active_chat(&self, primary: &str) -> Option<String> {
        self.storage
            .get(&format!("{ACTIVE_PREFIX}{primary}"))
            .await
            .ok()
            .flatten()
            .filter(|s| !s.is_empty())
    }

    pub async fn set_active_chat(&self, primary: &str, chat_id: &str) -> Result<()> {
        self.storage
            .set(&format!("{ACTIVE_PREFIX}{primary}"), chat_id)
            .await
    }

    // -- setup wizard state ---------------------------------------------------

    /// The step-by-step setup wizard keeps its progress here so a user can
    /// answer one question per message. Value format is wizard-defined
    /// (encrypted like everything else, since it may hold a URL or key).
    ///
    /// States expire after 15 minutes: an abandoned wizard must not keep
    /// swallowing every DM the user sends forever.
    pub async fn wizard_state(&self, primary: &str) -> Option<String> {
        let sealed = self
            .storage
            .get(&format!("{WIZARD_PREFIX}{primary}"))
            .await
            .ok()
            .flatten()?;
        let raw = self
            .secret
            .decrypt(&sealed)
            .ok()
            .filter(|s| !s.is_empty())?;
        // Stored as "<unix_ts>\n<state>".
        let (ts, state) = raw.split_once('\n')?;
        let ts: i64 = ts.parse().ok()?;
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);
        if now - ts > 15 * 60 {
            let _ = self.clear_wizard(primary).await;
            return None;
        }
        Some(state.to_owned())
    }

    pub async fn set_wizard_state(&self, primary: &str, state: &str) -> Result<()> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        let sealed = self.secret.encrypt(&format!("{now}\n{state}"))?;
        self.storage
            .set(&format!("{WIZARD_PREFIX}{primary}"), &sealed)
            .await
    }

    pub async fn clear_wizard(&self, primary: &str) -> Result<()> {
        self.storage.del(&format!("{WIZARD_PREFIX}{primary}")).await
    }

    // -- history ------------------------------------------------------------

    pub async fn history(&self, primary: &str, chat_id: &str) -> Result<Vec<ChatMessage>> {
        self.load(&format!("{HISTORY_PREFIX}{primary}:{chat_id}"))
            .await
    }

    pub async fn set_history(
        &self,
        primary: &str,
        chat_id: &str,
        messages: &[ChatMessage],
    ) -> Result<()> {
        self.store(&format!("{HISTORY_PREFIX}{primary}:{chat_id}"), messages)
            .await
    }

    /// Append a turn, trimming the stored history to [`HISTORY_LIMIT`].
    pub async fn push_history(
        &self,
        primary: &str,
        chat_id: &str,
        message: ChatMessage,
    ) -> Result<()> {
        let mut history = self.history(primary, chat_id).await?;
        history.push(message);
        if history.len() > HISTORY_LIMIT {
            let drop = history.len() - HISTORY_LIMIT;
            history.drain(0..drop);
        }
        self.set_history(primary, chat_id, &history).await
    }

    pub async fn clear_history(&self, primary: &str, chat_id: &str) -> Result<()> {
        self.storage
            .del(&format!("{HISTORY_PREFIX}{primary}:{chat_id}"))
            .await
    }

    // -- family access: shares granted BY an owner --------------------------

    pub async fn shares(&self, owner: &str) -> Result<Vec<Share>> {
        self.load(&format!("{SHARES_PREFIX}{owner}")).await
    }

    pub async fn set_shares(&self, owner: &str, shares: &[Share]) -> Result<()> {
        self.store(&format!("{SHARES_PREFIX}{owner}"), shares).await
    }

    /// Grant or update a grantee's access to some of the owner's hosts.
    pub async fn upsert_share(&self, owner: &str, share: Share) -> Result<()> {
        let grantee = share.grantee.clone();
        let mut shares = self.shares(owner).await?;
        match shares.iter_mut().find(|s| s.grantee == share.grantee) {
            Some(existing) => existing.hosts = share.hosts,
            None => shares.push(share),
        }
        self.set_shares(owner, &shares).await?;
        // Keep a reverse index so a grantee can find who shared with them
        // without scanning every user.
        self.add_shared_with(&grantee, owner).await
    }

    pub async fn revoke_share(&self, owner: &str, grantee: &str) -> Result<()> {
        let mut shares = self.shares(owner).await?;
        shares.retain(|s| s.grantee != grantee);
        self.set_shares(owner, &shares).await?;
        self.remove_shared_with(grantee, owner).await
    }

    // -- reverse index: which owners shared with a grantee ------------------

    async fn shared_with(&self, grantee: &str) -> Result<Vec<String>> {
        self.load(&format!("{SHARED_WITH_PREFIX}{grantee}")).await
    }

    async fn add_shared_with(&self, grantee: &str, owner: &str) -> Result<()> {
        let mut owners = self.shared_with(grantee).await?;
        if !owners.iter().any(|o| o == owner) {
            owners.push(owner.to_owned());
        }
        self.store(&format!("{SHARED_WITH_PREFIX}{grantee}"), &owners)
            .await
    }

    async fn remove_shared_with(&self, grantee: &str, owner: &str) -> Result<()> {
        let mut owners = self.shared_with(grantee).await?;
        owners.retain(|o| o != owner);
        self.store(&format!("{SHARED_WITH_PREFIX}{grantee}"), &owners)
            .await
    }

    /// Every host a grantee may use through family access, as
    /// `(owner_primary, Host, allowed_models)`. The returned `Host` carries
    /// the owner's real credentials (needed to call the API) but callers
    /// must never surface those to the grantee.
    pub async fn shared_hosts_for(&self, grantee: &str) -> Vec<(String, Host, Vec<String>)> {
        let mut out = Vec::new();
        for owner in self.shared_with(grantee).await.unwrap_or_default() {
            let shares = self.shares(&owner).await.unwrap_or_default();
            let Some(share) = shares.into_iter().find(|s| s.grantee == grantee) else {
                continue;
            };
            let owner_hosts = self.hosts(&owner).await.unwrap_or_default();
            for sh in share.hosts {
                if sh.models.is_empty() {
                    continue; // no models allowed = effectively revoked
                }
                if let Some(host) = owner_hosts.iter().find(|h| h.id == sh.host_id) {
                    out.push((owner.clone(), host.clone(), sh.models));
                }
            }
        }
        out
    }

    /// Resolve a host the caller may use by its id, for a specific model:
    /// their own host, or a shared one where that model is still allowed.
    /// Returns the ready-to-call [`Host`]. `None` if not owned/shared or the
    /// model was revoked - which is exactly how a changed permission stops
    /// working immediately.
    pub async fn usable_host(&self, user: &str, host_id: &str, model: &str) -> Option<Host> {
        if let Some(h) = self.host(user, host_id).await {
            return Some(h);
        }
        for (_owner, host, models) in self.shared_hosts_for(user).await {
            if host.id == host_id && models.iter().any(|m| m == model) {
                return Some(host);
            }
        }
        None
    }

    // -- pending invitations (stored on the grantee) ------------------------

    pub async fn pending(&self, grantee: &str) -> Result<Vec<PendingShare>> {
        self.load(&format!("{PENDING_PREFIX}{grantee}")).await
    }

    pub async fn set_pending(&self, grantee: &str, pend: &[PendingShare]) -> Result<()> {
        self.store(&format!("{PENDING_PREFIX}{grantee}"), pend)
            .await
    }

    pub async fn add_pending(&self, grantee: &str, pend: PendingShare) -> Result<()> {
        let mut list = self.pending(grantee).await?;
        // One pending invite per owner; a new one replaces the old.
        list.retain(|p| p.owner != pend.owner);
        list.push(pend);
        self.set_pending(grantee, &list).await
    }

    pub async fn remove_pending(&self, grantee: &str, owner: &str) -> Result<()> {
        let mut list = self.pending(grantee).await?;
        list.retain(|p| p.owner != owner);
        self.set_pending(grantee, &list).await
    }
}

/// Generate a short, collision-unlikely id from the clock.
pub fn new_id(prefix: &str) -> String {
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    format!("{prefix}{ts:x}")
}

// ---------- OpenAI-compatible client ----------------------------------------

/// Call an OpenAI-compatible `/v1/chat/completions` endpoint and return the
/// assistant's reply text. Works with LiteLLM, Ollama, LM Studio, vLLM,
/// OpenRouter and anything else speaking that dialect.
///
/// Only the given messages are sent - nothing about the bot or the user is
/// added. `base_url` may or may not already include `/v1`; we normalise it.
pub async fn chat_completion(
    host: &Host,
    model: &str,
    messages: &[ChatMessage],
) -> std::result::Result<String, String> {
    #[derive(Serialize)]
    struct Req<'a> {
        model: &'a str,
        messages: &'a [ChatMessage],
    }
    #[derive(Deserialize)]
    struct Resp {
        choices: Vec<Choice>,
    }
    #[derive(Deserialize)]
    struct Choice {
        message: RespMessage,
    }
    #[derive(Deserialize)]
    struct RespMessage {
        content: String,
    }

    let url = completions_url(&host.base_url);
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(120))
        .build()
        .map_err(|e| format!("client: {e}"))?;

    let mut req = client.post(&url).json(&Req { model, messages });
    if !host.api_key.is_empty() {
        req = req.bearer_auth(&host.api_key);
    }

    let resp = req
        .send()
        .await
        .map_err(|_| "host unreachable".to_owned())?;
    let status = resp.status();
    if !status.is_success() {
        return Err(format!("host returned HTTP {}", status.as_u16()));
    }
    let parsed: Resp = resp
        .json()
        .await
        .map_err(|_| "bad response from host".to_owned())?;
    parsed
        .choices
        .into_iter()
        .next()
        .map(|c| c.message.content)
        .ok_or_else(|| "empty response".to_owned())
}

/// Turn a user-supplied base URL into a full completions endpoint, being
/// forgiving about trailing slashes and a missing `/v1`.
fn completions_url(base: &str) -> String {
    let base = base.trim_end_matches('/');
    if base.ends_with("/chat/completions") {
        base.to_owned()
    } else if base.ends_with("/v1") {
        format!("{base}/chat/completions")
    } else {
        format!("{base}/v1/chat/completions")
    }
}

/// Ask a host which models it serves, via the OpenAI-compatible
/// `GET /v1/models`. Ollama, LiteLLM, LM Studio and vLLM all answer it,
/// which spares the user from typing model names by hand. Returns an
/// empty list when the endpoint isn't there - manual `/ai model add`
/// still works as the fallback.
pub async fn list_models(host: &Host) -> Vec<String> {
    #[derive(Deserialize)]
    struct ModelsResp {
        #[serde(default)]
        data: Vec<ModelEntry>,
    }
    #[derive(Deserialize)]
    struct ModelEntry {
        id: String,
    }

    let base = host.base_url.trim_end_matches('/');
    let url = if base.ends_with("/v1") {
        format!("{base}/models")
    } else {
        format!("{base}/v1/models")
    };
    let Ok(client) = reqwest::Client::builder()
        .user_agent("FoukoBot/1.0")
        .timeout(std::time::Duration::from_secs(10))
        .build()
    else {
        return Vec::new();
    };
    let mut req = client.get(&url);
    if !host.api_key.is_empty() {
        req = req.bearer_auth(&host.api_key);
    }
    let Ok(resp) = req.send().await else {
        return Vec::new();
    };
    if !resp.status().is_success() {
        return Vec::new();
    }
    let Ok(parsed) = resp.json::<ModelsResp>().await else {
        return Vec::new();
    };
    let mut models: Vec<String> = parsed.data.into_iter().map(|m| m.id).collect();
    models.sort();
    models.truncate(50); // a proxy can expose hundreds; keep the UI sane
    models
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn completions_url_normalises() {
        assert_eq!(
            completions_url("http://x:11434"),
            "http://x:11434/v1/chat/completions"
        );
        assert_eq!(
            completions_url("http://x:11434/"),
            "http://x:11434/v1/chat/completions"
        );
        assert_eq!(
            completions_url("http://x/v1"),
            "http://x/v1/chat/completions"
        );
        assert_eq!(
            completions_url("http://x/v1/chat/completions"),
            "http://x/v1/chat/completions"
        );
    }
}
