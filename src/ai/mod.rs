//! Private LLM integration: user-owned hosts, models, chats and history.
//!
//! Everything here is stored **encrypted** (via [`foukoapi::Secret`]) and
//! keyed off a user's *primary* identity, so it follows an account link
//! across platforms and never sits in the database as plaintext. The bot
//! only ever sends the model the user's own system prompt and their chat
//! history - no information about the bot, the user, or anyone else.

use foukoapi::{AnyStorage, Result, Secret};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// The OpenAI-compatible transport lives in the framework now. Re-export
// what command.rs uses so its imports stay `crate::ai::{...}`.
pub use foukoapi::genai::{
    caps_from_name, ChatOutcome, GenClient, GenError, ModelCaps, ProbeReport, ProbeVerdict,
    ToolCall, ToolSpec, KNOWN_VOICES,
};

pub mod command;
pub mod tools;

/// Last TTS voice a user picked, stored plaintext - a voice name is not
/// a secret. Keyed by primary identity. Shared by the command and the
/// web app.
const VOICE_PREFIX: &str = "foukobot:ai:voice:";

/// The voice a user last picked explicitly, if any.
pub async fn stored_voice(storage: &AnyStorage, primary: &str) -> Option<String> {
    storage
        .get(&format!("{VOICE_PREFIX}{primary}"))
        .await
        .ok()
        .flatten()
        .filter(|v| !v.is_empty())
}

/// Remember the user's voice pick. An empty name clears it.
pub async fn set_stored_voice(storage: &AnyStorage, primary: &str, voice: &str) -> Result<()> {
    let key = format!("{VOICE_PREFIX}{primary}");
    if voice.is_empty() {
        storage.del(&key).await
    } else {
        storage.set(&key, voice).await
    }
}

/// Storage-key prefixes. Values under these are ciphertext.
const HOSTS_PREFIX: &str = "foukobot:ai:hosts:";
const CHATS_PREFIX: &str = "foukobot:ai:chats:";
const HISTORY_PREFIX: &str = "foukobot:ai:history:";
const ACTIVE_PREFIX: &str = "foukobot:ai:active:";
const WIZARD_PREFIX: &str = "foukobot:ai:wizard:";
const SHARES_PREFIX: &str = "foukobot:ai:shares:";
const SHARED_WITH_PREFIX: &str = "foukobot:ai:sharedwith:";
const PENDING_PREFIX: &str = "foukobot:ai:share_pending:";

/// Pinned generation models, plaintext like the voice pref. Full key is
/// `foukobot:ai:gen:<cap>:<primary>` with `<cap>` = image|video|audio.
const GEN_PREF_PREFIX: &str = "foukobot:ai:gen:";

/// Separator inside a stored generation preference. Host names can't
/// contain whitespace, but nothing forbids '|' or ':' in a model name,
/// so use a control char nobody can type.
const GEN_PREF_SEP: char = '\u{1}';

/// Pack a host name + model into one preference value.
fn encode_gen_pref(host_name: &str, model: &str) -> String {
    format!("{host_name}{GEN_PREF_SEP}{model}")
}

/// Unpack a stored preference. `None` for garbage - the caller treats
/// that as "no preference".
fn decode_gen_pref(raw: &str) -> Option<(String, String)> {
    let (host, model) = raw.split_once(GEN_PREF_SEP)?;
    if host.is_empty() || model.is_empty() {
        return None;
    }
    Some((host.to_owned(), model.to_owned()))
}

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
    /// Capability masks per model name. Only non-text models are stored
    /// (text is the default), so old records simply have an empty map and
    /// fall back to the name heuristic.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub model_caps: HashMap<String, u8>,
    /// Accept the host's self-signed TLS certificate. Off by default;
    /// old records don't have the field and stay strict.
    #[serde(default, skip_serializing_if = "is_false")]
    pub insecure: bool,
}

/// serde helper: skip `false` so records stay compact.
fn is_false(b: &bool) -> bool {
    !*b
}

impl Host {
    /// Capabilities of a model on this host: the stored mask when we have
    /// one, the name heuristic otherwise (old records, manual model add).
    pub fn caps_of(&self, model: &str) -> ModelCaps {
        match self.model_caps.get(model) {
            Some(&mask) => ModelCaps(mask),
            None => caps_from_name(model),
        }
    }
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
    /// Unix seconds when the turn was stored. Optional so records written
    /// before this field existed keep deserializing.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ts: Option<u64>,
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
    /// Stamps the turn with the current time unless the caller set one.
    pub async fn push_history(
        &self,
        primary: &str,
        chat_id: &str,
        mut message: ChatMessage,
    ) -> Result<()> {
        if message.ts.is_none() {
            message.ts = Some(now_secs());
        }
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

    // -- pinned generation models --------------------------------------------
    //
    // Plaintext, like the voice pref: a host name and a model name aren't
    // secrets. One key per capability.

    /// The user's pinned host + model for a capability, if any.
    pub async fn gen_pref(&self, primary: &str, cap: u8) -> Option<(String, String)> {
        let name = cap_name(cap)?;
        let raw = self
            .storage
            .get(&format!("{GEN_PREF_PREFIX}{name}:{primary}"))
            .await
            .ok()
            .flatten()?;
        decode_gen_pref(&raw)
    }

    /// Pin a host + model for a capability.
    pub async fn set_gen_pref(
        &self,
        primary: &str,
        cap: u8,
        host_name: &str,
        model: &str,
    ) -> Result<()> {
        let Some(name) = cap_name(cap) else {
            return Err(foukoapi::Error::Other("unknown capability".into()));
        };
        self.storage
            .set(
                &format!("{GEN_PREF_PREFIX}{name}:{primary}"),
                &encode_gen_pref(host_name, model),
            )
            .await
    }

    /// Drop the pin - auto-pick takes over again.
    pub async fn clear_gen_pref(&self, primary: &str, cap: u8) -> Result<()> {
        let Some(name) = cap_name(cap) else {
            return Err(foukoapi::Error::Other("unknown capability".into()));
        };
        self.storage
            .del(&format!("{GEN_PREF_PREFIX}{name}:{primary}"))
            .await
    }
}

/// Storage-key name of a capability bit.
pub fn cap_name(cap: u8) -> Option<&'static str> {
    match cap {
        ModelCaps::IMAGE => Some("image"),
        ModelCaps::VIDEO => Some("video"),
        ModelCaps::AUDIO => Some("audio"),
        _ => None,
    }
}

/// Parse a user-typed capability name into its bit.
pub fn cap_from_arg(arg: &str) -> Option<u8> {
    match arg.trim().to_ascii_lowercase().as_str() {
        "image" | "img" => Some(ModelCaps::IMAGE),
        "video" | "vid" => Some(ModelCaps::VIDEO),
        "audio" | "speech" | "tts" => Some(ModelCaps::AUDIO),
        _ => None,
    }
}

// ---------- shared operations (command.rs and the web app) -------------------

/// Find one of the user's own hosts by name.
pub async fn find_host_by_name(store: &AiStore, primary: &str, name: &str) -> Option<Host> {
    store
        .hosts(primary)
        .await
        .ok()?
        .into_iter()
        .find(|h| h.name == name)
}

/// Find one of the user's chats by name.
pub async fn find_chat_by_name(store: &AiStore, primary: &str, name: &str) -> Option<Chat> {
    store
        .chats(primary)
        .await
        .ok()?
        .into_iter()
        .find(|c| c.name == name)
}

/// Resolve a host by name across the user's own hosts and any shared with
/// them, returning its id and whether `model` is available. Own hosts win
/// on a name clash.
pub async fn resolve_host_for_chat(
    store: &AiStore,
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

/// Create a host, try model discovery and store it. Returns how many
/// models were found and the discovery error, if any (a failed discovery
/// doesn't block the add). Callers validate the name and URL first.
pub async fn add_host_discovering(
    store: &AiStore,
    primary: &str,
    name: &str,
    url: &str,
    key: &str,
) -> Result<(usize, Option<String>)> {
    let mut host = Host {
        id: new_id("h"),
        name: name.to_owned(),
        base_url: url.to_owned(),
        api_key: key.to_owned(),
        models: Vec::new(),
        model_caps: HashMap::new(),
        insecure: false,
    };
    let mut found = 0usize;
    let mut discover_err = None;
    match list_models(&host).await {
        Ok(discovered) => {
            found = discovered.len();
            let (models, caps) = split_discovered(discovered);
            host.models = models;
            host.model_caps = caps;
        }
        Err(e) => discover_err = Some(e),
    }
    store.add_host(primary, host).await?;
    Ok((found, discover_err))
}

/// Re-pull a host's model list. `Ok(Err(reason))` when discovery failed
/// (the stored list stays untouched); `Ok(Ok(0))` when the host answered
/// with an empty list (also untouched - an outage must not wipe models).
pub async fn refresh_host_models(
    store: &AiStore,
    primary: &str,
    host: &Host,
) -> Result<std::result::Result<usize, String>> {
    let discovered = match list_models(host).await {
        Ok(d) => d,
        Err(e) => return Ok(Err(e)),
    };
    if discovered.is_empty() {
        return Ok(Ok(0));
    }
    let found = discovered.len();
    let (models, caps) = split_discovered(discovered);
    let mut hosts = store.hosts(primary).await?;
    if let Some(h) = hosts.iter_mut().find(|h| h.id == host.id) {
        h.models = models;
        h.model_caps = caps;
    }
    store.set_hosts(primary, &hosts).await?;
    Ok(Ok(found))
}

/// Flip a host's "accept self-signed cert" switch. `Ok(false)` when no
/// host with that name exists.
pub async fn set_host_insecure(
    store: &AiStore,
    primary: &str,
    host_name: &str,
    on: bool,
) -> Result<bool> {
    let mut hosts = store.hosts(primary).await?;
    let Some(host) = hosts.iter_mut().find(|h| h.name == host_name) else {
        return Ok(false);
    };
    host.insecure = on;
    store.set_hosts(primary, &hosts).await?;
    Ok(true)
}

/// Outcome of a model add/del/tag/untag on one of the user's own hosts.
#[derive(Debug, PartialEq, Eq)]
pub enum ModelEdit {
    Done,
    HostMissing,
    ModelMissing,
}

/// Register a model by hand on an own host (idempotent).
pub async fn add_model(
    store: &AiStore,
    primary: &str,
    host_name: &str,
    model: &str,
) -> Result<ModelEdit> {
    let mut hosts = store.hosts(primary).await?;
    let Some(host) = hosts.iter_mut().find(|h| h.name == host_name) else {
        return Ok(ModelEdit::HostMissing);
    };
    if !host.models.iter().any(|m| m == model) {
        host.models.push(model.to_owned());
    }
    store.set_hosts(primary, &hosts).await?;
    Ok(ModelEdit::Done)
}

/// Drop a model (and its caps entry) from an own host.
pub async fn del_model(
    store: &AiStore,
    primary: &str,
    host_name: &str,
    model: &str,
) -> Result<ModelEdit> {
    let mut hosts = store.hosts(primary).await?;
    let Some(host) = hosts.iter_mut().find(|h| h.name == host_name) else {
        return Ok(ModelEdit::HostMissing);
    };
    host.models.retain(|m| m != model);
    host.model_caps.remove(model);
    store.set_hosts(primary, &hosts).await?;
    Ok(ModelEdit::Done)
}

/// Tag a model with a capability bit on top of whatever is already known
/// (stored mask or the name heuristic), so tagging never erases caps.
pub async fn tag_model(
    store: &AiStore,
    primary: &str,
    host_name: &str,
    model: &str,
    cap: u8,
) -> Result<ModelEdit> {
    let mut hosts = store.hosts(primary).await?;
    let Some(host) = hosts.iter_mut().find(|h| h.name == host_name) else {
        return Ok(ModelEdit::HostMissing);
    };
    if !host.models.iter().any(|m| m == model) {
        return Ok(ModelEdit::ModelMissing);
    }
    let mask = host.caps_of(model).0 | cap;
    host.model_caps.insert(model.to_owned(), mask);
    store.set_hosts(primary, &hosts).await?;
    Ok(ModelEdit::Done)
}

/// Drop a model's stored caps mask; the name heuristic takes over again.
pub async fn untag_model(
    store: &AiStore,
    primary: &str,
    host_name: &str,
    model: &str,
) -> Result<ModelEdit> {
    let mut hosts = store.hosts(primary).await?;
    let Some(host) = hosts.iter_mut().find(|h| h.name == host_name) else {
        return Ok(ModelEdit::HostMissing);
    };
    if !host.models.iter().any(|m| m == model) {
        return Ok(ModelEdit::ModelMissing);
    }
    host.model_caps.remove(model);
    store.set_hosts(primary, &hosts).await?;
    Ok(ModelEdit::Done)
}

/// Why creating a chat failed.
#[derive(Debug, PartialEq, Eq)]
pub enum ChatCreateError {
    HostMissing,
    ModelMissing,
}

/// Create a chat on an own or shared host and make it active.
pub async fn create_chat(
    store: &AiStore,
    primary: &str,
    name: &str,
    host_name: &str,
    model: &str,
) -> Result<std::result::Result<Chat, ChatCreateError>> {
    let Some((host_id, ok_model)) = resolve_host_for_chat(store, primary, host_name, model).await
    else {
        return Ok(Err(ChatCreateError::HostMissing));
    };
    if !ok_model {
        return Ok(Err(ChatCreateError::ModelMissing));
    }
    let chat = Chat {
        id: new_id("c"),
        name: name.to_owned(),
        host_id,
        model: model.to_owned(),
        system_prompt: String::new(),
    };
    store.add_chat(primary, chat.clone()).await?;
    store.set_active_chat(primary, &chat.id).await?;
    Ok(Ok(chat))
}

/// Current time as unix seconds. Zero on a pre-epoch clock, which is
/// harmless for message timestamps.
pub fn now_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// Generate a short, collision-unlikely id from the clock.
pub fn new_id(prefix: &str) -> String {
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    format!("{prefix}{ts:x}")
}

// ---------- OpenAI-compatible client (foukoapi::genai) -----------------------
//
// The transport (chat/images/videos/speech/model discovery, URL
// normalisation, size caps, job polling) lives in the framework now.
// These thin wrappers keep command.rs unchanged: they build a client
// from a Host and forward.

/// Build a [`GenClient`] for a host. An empty `api_key` means "no auth",
/// which is what local servers (Ollama, LM Studio) expect.
pub fn client_for(host: &Host) -> GenClient {
    let key = (!host.api_key.is_empty()).then(|| host.api_key.clone());
    GenClient::new(host.base_url.clone(), key).insecure(host.insecure)
}

/// Convert stored history turns into the framework's message type.
fn to_genai_messages(messages: &[ChatMessage]) -> Vec<foukoapi::genai::ChatMessage> {
    messages
        .iter()
        .map(|m| match m.role.as_str() {
            "system" => foukoapi::genai::ChatMessage::system(m.content.clone()),
            "assistant" => foukoapi::genai::ChatMessage::assistant(m.content.clone()),
            _ => foukoapi::genai::ChatMessage::user(m.content.clone()),
        })
        .collect()
}

/// Streamed chat with a model on this host: `on_delta` gets the
/// accumulated text so far on every SSE chunk. Only the given messages
/// are sent - nothing about the bot or the user is added. The optional
/// image goes onto the last (user) message, one-shot: sent with this
/// request only, never persisted. `tools` may be empty (the field is
/// then omitted entirely); `extra` carries wire-only messages (tool
/// results, assistant call echoes) appended after the stored history.
/// Returns the raw [`GenError`] so the caller can catch
/// [`GenError::NotSupported`] and retry without tools.
pub async fn chat_completion_stream_tools<F>(
    host: &Host,
    model: &str,
    messages: &[ChatMessage],
    extra: &[foukoapi::genai::ChatMessage],
    image: Option<Vec<u8>>,
    tools: &[ToolSpec],
    on_delta: F,
) -> std::result::Result<ChatOutcome, GenError>
where
    F: FnMut(&str) + Send,
{
    let mut msgs = to_genai_messages(messages);
    if let Some(bytes) = image {
        let mime = image_mime(&bytes);
        if let Some(last) = msgs.pop() {
            msgs.push(last.with_image(bytes, mime));
        }
    }
    msgs.extend(extra.iter().cloned());
    client_for(host)
        .chat_stream_tools(model, &msgs, tools, on_delta)
        .await
}

/// Generate an image and return the raw bytes.
pub async fn image_generation(
    host: &Host,
    model: &str,
    prompt: &str,
) -> std::result::Result<Vec<u8>, GenError> {
    client_for(host).image(model, prompt).await
}

/// Generate a short video (job-style API) and return the raw mp4 bytes.
pub async fn video_generation(
    host: &Host,
    model: &str,
    prompt: &str,
) -> std::result::Result<Vec<u8>, GenError> {
    client_for(host).video(model, prompt).await
}

/// Synthesize speech and return the raw mp3 bytes. `voice: None` uses the
/// server default ("alloy"). The voice is not validated here - proxies may
/// serve their own voices.
pub async fn speech_generation(
    host: &Host,
    model: &str,
    text: &str,
    voice: Option<&str>,
) -> std::result::Result<Vec<u8>, GenError> {
    client_for(host).speech(model, text, voice).await
}

/// Ask a host which models it serves. The error carries the reason as
/// text so callers can show it - a silent empty list used to mask cert
/// and connection problems as "0 models". Capability masks come from
/// host metadata when present, the name heuristic otherwise (handled
/// inside the framework).
pub async fn list_models(host: &Host) -> std::result::Result<Vec<(String, ModelCaps)>, String> {
    client_for(host)
        .list_models()
        .await
        .map(|models| models.into_iter().map(|m| (m.id, m.caps)).collect())
        .map_err(|e| e.to_string())
}

// ---------- model capability helpers (bot-side) -------------------------------

/// Name fragments that mark a text model as vision-capable (image
/// understanding, not generation - caps_from_name covers the latter).
const VISION_MARKERS: &[&str] = &[
    "gpt-4o",
    "gpt-4.1",
    "gpt-5",
    "claude",
    "gemini",
    "llava",
    "vision",
    "-vl",
    "qwen2-vl",
    "qwen2.5-vl",
    "pixtral",
    "moondream",
    "minicpm-v",
];

/// Can this chat model understand images? Heuristic on the name: vision is
/// a property of text models, so anything the caps heuristic flags as a
/// generation model (DALL-E, TTS, ...) is excluded up front.
pub fn model_sees_images(model: &str) -> bool {
    if !caps_from_name(model).is_text() {
        return false;
    }
    let low = model.to_lowercase();
    if low.contains("dall-e") || low.contains("tts") || low.contains("embed") {
        return false;
    }
    VISION_MARKERS.iter().any(|m| low.contains(m))
}

/// Sniff an image MIME type from magic bytes. JPEG is the safe default:
/// that's what messengers re-encode photos to anyway.
pub fn image_mime(bytes: &[u8]) -> &'static str {
    match bytes {
        [0x89, 0x50, 0x4E, 0x47, ..] => "image/png",
        [0xFF, 0xD8, ..] => "image/jpeg",
        [0x47, 0x49, 0x46, 0x38, ..] => "image/gif",
        [0x52, 0x49, 0x46, 0x46, _, _, _, _, 0x57, 0x45, 0x42, 0x50, ..] => "image/webp",
        _ => "image/jpeg",
    }
}

/// First model on the host with the given capability bit. Stored caps
/// win, the name heuristic covers old records and manual `model add`.
pub fn model_with_cap(host: &Host, cap: u8) -> Option<&str> {
    host.models
        .iter()
        .map(String::as_str)
        .find(|m| host.caps_of(m).0 & cap != 0)
}

/// First model in a bare list with the given capability bit - heuristic
/// only, since a grantee doesn't see the owner's caps map.
pub fn model_with_cap_in(models: &[String], cap: u8) -> Option<&str> {
    models
        .iter()
        .map(String::as_str)
        .find(|m| caps_from_name(m).0 & cap != 0)
}

/// Split a discovery result into what [`Host`] stores: the plain model
/// list plus a caps map holding only the non-text entries.
pub fn split_discovered(
    discovered: Vec<(String, ModelCaps)>,
) -> (Vec<String>, HashMap<String, u8>) {
    let mut names = Vec::with_capacity(discovered.len());
    let mut caps = HashMap::new();
    for (name, c) in discovered {
        if !c.is_text() {
            caps.insert(name.clone(), c.0);
        }
        names.push(name);
    }
    (names, caps)
}

#[cfg(test)]
mod tests {
    use super::*;

    // Transport tests (URL normalisation, caps heuristic, metadata parsing)
    // moved to foukoapi::genai with the code. Only bot-side helpers remain.

    #[test]
    fn vision_model_heuristic() {
        for m in [
            "gpt-4o",
            "GPT-4.1-mini",
            "claude-sonnet-4",
            "gemini-2.0-flash",
            "llava:13b",
            "qwen2.5-vl-7b",
            "pixtral-12b",
            "moondream2",
            "minicpm-v-2.6",
        ] {
            assert!(model_sees_images(m), "{m} should see images");
        }
        for m in [
            "llama3",
            "mistral",
            "dall-e-3",
            "gpt-4o-mini-tts",
            "text-embedding-3-small",
        ] {
            assert!(!model_sees_images(m), "{m} should not see images");
        }
    }

    #[test]
    fn mime_sniffing() {
        assert_eq!(image_mime(&[0xFF, 0xD8, 0xFF, 0xE0]), "image/jpeg");
        assert_eq!(image_mime(&[0x89, 0x50, 0x4E, 0x47, 0x0D]), "image/png");
        assert_eq!(image_mime(b"GIF89a"), "image/gif");
        assert_eq!(image_mime(b"RIFF\x00\x00\x00\x00WEBPVP8 "), "image/webp");
        assert_eq!(image_mime(b"??"), "image/jpeg");
    }

    #[test]
    fn image_model_heuristic() {
        let models = vec!["llama3".to_owned(), "DALL-E-3".to_owned()];
        assert_eq!(
            model_with_cap_in(&models, ModelCaps::IMAGE),
            Some("DALL-E-3")
        );
        let none = vec!["llama3".to_owned(), "mistral".to_owned()];
        assert_eq!(model_with_cap_in(&none, ModelCaps::IMAGE), None);
    }

    #[test]
    fn video_and_audio_model_picks() {
        let host = Host {
            id: "h1".into(),
            name: "x".into(),
            base_url: "http://x".into(),
            api_key: String::new(),
            models: vec!["llama3".into(), "sora-2".into(), "gpt-4o-mini-tts".into()],
            model_caps: HashMap::new(),
            insecure: false,
        };
        assert_eq!(model_with_cap(&host, ModelCaps::VIDEO), Some("sora-2"));
        assert_eq!(
            model_with_cap(&host, ModelCaps::AUDIO),
            Some("gpt-4o-mini-tts")
        );
        assert_eq!(
            model_with_cap_in(&host.models, ModelCaps::VIDEO),
            Some("sora-2")
        );
    }

    #[test]
    fn stored_caps_win_over_name() {
        let mut host = Host {
            id: "h1".into(),
            name: "x".into(),
            base_url: "http://x".into(),
            api_key: String::new(),
            models: vec!["custom-img".into()],
            model_caps: HashMap::new(),
            insecure: false,
        };
        // Name alone says text; the stored mask says image.
        assert!(host.caps_of("custom-img").is_text());
        host.model_caps
            .insert("custom-img".into(), ModelCaps::IMAGE);
        assert!(host.caps_of("custom-img").image());
        assert_eq!(model_with_cap(&host, ModelCaps::IMAGE), Some("custom-img"));
    }

    #[test]
    fn gen_pref_round_trips() {
        let raw = encode_gen_pref("myhost", "weird|model:v2");
        assert_eq!(
            decode_gen_pref(&raw),
            Some(("myhost".to_owned(), "weird|model:v2".to_owned()))
        );
        // Garbage and half-empty values read as "no preference".
        assert_eq!(decode_gen_pref("no-separator"), None);
        assert_eq!(decode_gen_pref("\u{1}model"), None);
        assert_eq!(decode_gen_pref("host\u{1}"), None);
    }

    #[test]
    fn cap_names() {
        assert_eq!(cap_name(ModelCaps::IMAGE), Some("image"));
        assert_eq!(cap_name(ModelCaps::VIDEO), Some("video"));
        assert_eq!(cap_name(ModelCaps::AUDIO), Some("audio"));
        assert_eq!(cap_name(0), None);
        assert_eq!(cap_name(ModelCaps::IMAGE | ModelCaps::VIDEO), None);
    }
}
