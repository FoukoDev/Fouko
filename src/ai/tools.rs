//! Tool calling: the function specs offered to the chat model so it can
//! trigger image/video/speech generation by itself, plus the plumbing
//! around them (per-user switch, hosts known to reject tools).

use super::{AiStore, Host, ModelCaps};
use crate::commands::Services;
use foukoapi::genai::{ToolSpec, KNOWN_VOICES};
use serde_json::json;
use std::collections::HashSet;
use std::sync::{Mutex, OnceLock};

/// Per-user tool switch, stored plaintext (it's just a preference).
/// The stored value is `"off"`; a missing record means enabled.
const TOOLS_PREFIX: &str = "foukobot:ai:tools:";

/// Hosts that rejected a request carrying tools, keyed by base URL.
/// Process-local on purpose: a restart re-probes, which costs one
/// failed request at worst.
static UNSUPPORTED: OnceLock<Mutex<HashSet<String>>> = OnceLock::new();

fn unsupported_set() -> &'static Mutex<HashSet<String>> {
    UNSUPPORTED.get_or_init(|| Mutex::new(HashSet::new()))
}

/// Stale pins we already warned about, so a broken pin logs once instead
/// of on every message (tool specs probe all caps per turn).
static WARNED_PINS: OnceLock<Mutex<HashSet<String>>> = OnceLock::new();

fn warned_pins() -> &'static Mutex<HashSet<String>> {
    WARNED_PINS.get_or_init(|| Mutex::new(HashSet::new()))
}

/// Remember that this host can't do tool calling.
pub fn mark_host_unsupported(url: &str) {
    if let Ok(mut set) = unsupported_set().lock() {
        set.insert(url.to_owned());
    }
}

/// Has this host already rejected tools?
pub fn host_unsupported(url: &str) -> bool {
    unsupported_set()
        .lock()
        .map(|set| set.contains(url))
        .unwrap_or(false)
}

/// Did the user leave tool calling on? Enabled unless they said "off".
pub async fn user_tools_enabled(svc: &Services, primary: &str) -> bool {
    svc.storage
        .get(&format!("{TOOLS_PREFIX}{primary}"))
        .await
        .ok()
        .flatten()
        .map(|v| v != "off")
        .unwrap_or(true)
}

/// Flip the per-user switch. "On" is the default, so it just drops the row.
pub async fn set_user_tools(svc: &Services, primary: &str, on: bool) -> foukoapi::Result<()> {
    let key = format!("{TOOLS_PREFIX}{primary}");
    if on {
        svc.storage.del(&key).await
    } else {
        svc.storage.set(&key, "off").await
    }
}

/// Can the active chat's host take tools? True when we can't tell (talk
/// will surface real errors itself) - only a known rejection disables.
pub async fn active_host_tools_ok(store: &AiStore, primary: &str) -> bool {
    let Some(active) = store.active_chat(primary).await else {
        return true;
    };
    let Some(chat) = store.chat(primary, &active).await else {
        return true;
    };
    let Some(host) = store.usable_host(primary, &chat.host_id, &chat.model).await else {
        return true;
    };
    !host_unsupported(&host.base_url)
}

/// Build the tool list for a user: a tool is offered only when they have
/// a model that can actually back it, so the model never promises what
/// the bot can't deliver.
pub async fn specs_for(store: &AiStore, primary: &str) -> Vec<ToolSpec> {
    let mut specs = Vec::new();
    if resolve_capable_host(store, primary, ModelCaps::IMAGE)
        .await
        .is_some()
    {
        specs.push(ToolSpec::new(
            "generate_image",
            "Generate an image from a text prompt. Use whenever the user asks to draw, \
             paint, render or create a picture.",
            json!({
                "type": "object",
                "properties": {
                    "prompt": {
                        "type": "string",
                        "description": "What to depict, as a detailed image prompt"
                    }
                },
                "required": ["prompt"]
            }),
        ));
    }
    if resolve_capable_host(store, primary, ModelCaps::VIDEO)
        .await
        .is_some()
    {
        specs.push(ToolSpec::new(
            "generate_video",
            "Generate a short video clip from a text prompt. Takes a couple of minutes.",
            json!({
                "type": "object",
                "properties": {
                    "prompt": {
                        "type": "string",
                        "description": "What to film, as a detailed video prompt"
                    }
                },
                "required": ["prompt"]
            }),
        ));
    }
    if resolve_capable_host(store, primary, ModelCaps::AUDIO)
        .await
        .is_some()
    {
        specs.push(ToolSpec::new(
            "speak",
            "Convert text to speech and send it as an audio message.",
            json!({
                "type": "object",
                "properties": {
                    "text": {
                        "type": "string",
                        "description": "The text to voice"
                    },
                    "voice": {
                        "type": "string",
                        "enum": KNOWN_VOICES,
                        "description": "Voice name, optional"
                    }
                },
                "required": ["text"]
            }),
        ));
    }
    specs
}

/// Find a host + model with the given capability the user may use. A
/// pinned pick (`/ai gen`) wins while it's still valid; otherwise the
/// active chat's host gets first shot, then their own hosts, then hosts
/// shared with them (where the granted model list decides, mirroring how
/// `talk` resolves access).
pub async fn resolve_capable_host(store: &AiStore, p: &str, cap: u8) -> Option<(Host, String)> {
    // A stale pin (host gone, model dropped, access revoked) logs and
    // falls through - generation keeps working on auto-pick.
    if let Some((host_name, model)) = store.gen_pref(p, cap).await {
        if let Some(hit) = pinned_candidate(store, p, cap, &host_name, &model).await {
            return Some(hit);
        }
        // Same broken pin gets hit on every message (tool specs alone
        // probe all three caps), so warn once per pin, not per call.
        let key = format!("{p}:{cap}:{host_name}:{model}");
        let fresh = {
            let mut seen = warned_pins().lock().unwrap_or_else(|e| e.into_inner());
            if seen.len() > 256 {
                seen.clear();
            }
            seen.insert(key)
        };
        if fresh {
            tracing::warn!(
                host = %host_name,
                model = %model,
                cap,
                "pinned generation model unavailable, using auto-pick"
            );
        }
    }
    // The active chat's host gets first shot.
    if let Some(active) = store.active_chat(p).await {
        if let Some(chat) = store.chat(p, &active).await {
            if let Some(host) = store.usable_host(p, &chat.host_id, &chat.model).await {
                if let Some(model) = super::model_with_cap(&host, cap) {
                    return Some((host.clone(), model.to_owned()));
                }
            }
        }
    }
    // Then any of the user's own hosts.
    for host in store.hosts(p).await.unwrap_or_default() {
        if let Some(model) = super::model_with_cap(&host, cap) {
            let model = model.to_owned();
            return Some((host, model));
        }
    }
    // Finally family-shared hosts - only among the models actually granted.
    for (_owner, host, models) in store.shared_hosts_for(p).await {
        if let Some(model) = super::model_with_cap_in(&models, cap) {
            let model = model.to_owned();
            return Some((host, model));
        }
    }
    None
}

/// Check a host name + model against what the user can actually reach:
/// their own hosts first, then shared ones where the model is still
/// granted. Caps come from `Host::caps_of` - stored tags when present,
/// the name heuristic otherwise (a shared clone carries the owner's
/// map, so tags stay accurate there too). Returns the ready-to-call
/// pair when everything lines up, `None` otherwise.
pub async fn pinned_candidate(
    store: &AiStore,
    p: &str,
    cap: u8,
    host_name: &str,
    model: &str,
) -> Option<(Host, String)> {
    for host in store.hosts(p).await.unwrap_or_default() {
        if host.name == host_name
            && host.models.iter().any(|m| m == model)
            && host.caps_of(model).0 & cap != 0
        {
            return Some((host, model.to_owned()));
        }
    }
    for (_owner, host, models) in store.shared_hosts_for(p).await {
        if host.name == host_name
            && models.iter().any(|m| m == model)
            && host.caps_of(model).0 & cap != 0
        {
            return Some((host, model.to_owned()));
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unsupported_hosts_are_remembered() {
        assert!(!host_unsupported("http://tools-test.example"));
        mark_host_unsupported("http://tools-test.example");
        assert!(host_unsupported("http://tools-test.example"));
        assert!(!host_unsupported("http://other.example"));
    }
}
