//! Session identity and cursor helpers.

use alloc::borrow::ToOwned;
use alloc::string::String;

#[cfg(feature = "schemars")]
use schemars::JsonSchema;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::{FlowId, PackId, TenantCtx};

use sha2::{Digest, Sha256};

/// Unique key referencing a persisted session.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[cfg_attr(feature = "serde", serde(transparent))]
pub struct SessionKey(pub String);

impl SessionKey {
    /// Returns the session key as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Creates a new session key from the supplied string.
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    /// Generates a random session key using [`uuid`], when enabled.
    #[cfg(feature = "uuid")]
    pub fn generate() -> Self {
        Self(uuid::Uuid::new_v4().to_string())
    }
}

impl From<String> for SessionKey {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl From<&str> for SessionKey {
    fn from(value: &str) -> Self {
        Self(value.to_owned())
    }
}

impl core::fmt::Display for SessionKey {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[cfg(feature = "uuid")]
impl From<uuid::Uuid> for SessionKey {
    fn from(value: uuid::Uuid) -> Self {
        Self(value.to_string())
    }
}

const DEFAULT_CANONICAL_ANCHOR: &str = "conversation";
const DEFAULT_CANONICAL_USER: &str = "user";

/// Build the canonical `{tenant}:{provider}:{anchor}:{user}` session key.
///
/// All canonical adapters are expected to follow this format so pause/resume semantics remain
/// deterministic across ingress providers. The anchor defaults to `conversation` and the user
/// defaults to `user` when those fields are not supplied.
pub fn canonical_session_key(
    tenant: impl AsRef<str>,
    provider: impl AsRef<str>,
    anchor: Option<&str>,
    user: Option<&str>,
) -> SessionKey {
    SessionKey::new(format!(
        "{}:{}:{}:{}",
        tenant.as_ref(),
        provider.as_ref(),
        anchor.unwrap_or(DEFAULT_CANONICAL_ANCHOR),
        user.unwrap_or(DEFAULT_CANONICAL_USER)
    ))
}

/// Cursor pointing at a session's position in a flow graph.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub struct SessionCursor {
    /// Identifier of the node currently owning the session.
    pub node_pointer: String,
    /// Optional wait reason emitted by the node.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub wait_reason: Option<String>,
    /// Optional marker describing pending outbox operations.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub outbox_marker: Option<String>,
}

impl SessionCursor {
    /// Creates a new cursor pointing at the provided node identifier.
    pub fn new(node_pointer: impl Into<String>) -> Self {
        Self {
            node_pointer: node_pointer.into(),
            wait_reason: None,
            outbox_marker: None,
        }
    }

    /// Assigns a wait reason to the cursor.
    pub fn with_wait_reason(mut self, reason: impl Into<String>) -> Self {
        self.wait_reason = Some(reason.into());
        self
    }

    /// Assigns an outbox marker to the cursor.
    pub fn with_outbox_marker(mut self, marker: impl Into<String>) -> Self {
        self.outbox_marker = Some(marker.into());
        self
    }
}

/// Persisted session payload describing how to resume a flow.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub struct SessionData {
    /// Tenant context associated with the session.
    pub tenant_ctx: TenantCtx,
    /// Flow identifier being executed.
    pub flow_id: FlowId,
    /// Optional pack identifier tied to the session.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub pack_id: Option<PackId>,
    /// Cursor pinpointing where execution paused.
    pub cursor: SessionCursor,
    /// Serialized execution context/state snapshot.
    pub context_json: String,
}

/// Stable scope describing where a reply is anchored (conversation/thread/reply).
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub struct ReplyScope {
    /// Conversation identifier.
    pub conversation: String,
    /// Optional thread/topic identifier.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub thread: Option<String>,
    /// Optional reply-to identifier.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub reply_to: Option<String>,
    /// Optional correlation identifier.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub correlation: Option<String>,
}

/// Legacy alias for reply scope.
#[deprecated(since = "0.4.52", note = "use ReplyScope")]
pub type WaitScope = ReplyScope;

impl ReplyScope {
    /// Returns a deterministic hash for the scope.
    pub fn scope_hash(&self) -> String {
        let mut canonical = String::new();
        canonical.push_str(self.conversation.as_str());
        canonical.push('\n');
        canonical.push_str(self.thread.as_deref().unwrap_or(""));
        canonical.push('\n');
        canonical.push_str(self.reply_to.as_deref().unwrap_or(""));
        canonical.push('\n');
        canonical.push_str(self.correlation.as_deref().unwrap_or(""));

        let digest = Sha256::digest(canonical.as_bytes());
        hex_encode(digest.as_slice())
    }
}

fn hex_encode(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len() * 2);
    for &byte in bytes {
        out.push(HEX[(byte >> 4) as usize] as char);
        out.push(HEX[(byte & 0x0f) as usize] as char);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(feature = "serde")]
    use serde_json::Value;

    #[test]
    fn canonical_session_key_includes_components() {
        let key = canonical_session_key("tenant", "webhook", Some("room-1"), Some("user-5"));
        assert_eq!(key.as_str(), "tenant:webhook:room-1:user-5");
    }

    #[test]
    fn canonical_session_key_defaults_anchor_and_user() {
        let key = canonical_session_key("tenant", "webhook", None, None);
        assert_eq!(key.as_str(), "tenant:webhook:conversation:user");
    }

    #[test]
    fn reply_scope_hash_is_deterministic() {
        let scope = ReplyScope {
            conversation: "chat-1".to_owned(),
            thread: Some("topic-9".to_owned()),
            reply_to: Some("msg-3".to_owned()),
            correlation: Some("cid-7".to_owned()),
        };

        assert_eq!(
            scope.scope_hash(),
            "5251e2c9eb975466809eecbe51be7aa8dc7785ad05cb393a8f47a3a2f42cc544"
        );
    }

    #[test]
    fn reply_scope_hash_changes_with_fields() {
        let base = ReplyScope {
            conversation: "chat-1".to_owned(),
            thread: Some("topic-9".to_owned()),
            reply_to: Some("msg-3".to_owned()),
            correlation: Some("cid-7".to_owned()),
        };

        let mut altered = base.clone();
        altered.reply_to = Some("msg-4".to_owned());

        assert_ne!(base.scope_hash(), altered.scope_hash());
    }

    #[cfg(feature = "serde")]
    #[test]
    fn reply_scope_roundtrip() {
        let scope = ReplyScope {
            conversation: "chat-2".to_owned(),
            thread: None,
            reply_to: Some("msg-9".to_owned()),
            correlation: None,
        };

        let value = serde_json::to_value(&scope)
            .unwrap_or_else(|err| panic!("serialize reply scope failed: {err}"));
        let roundtrip: ReplyScope = serde_json::from_value(value)
            .unwrap_or_else(|err| panic!("deserialize reply scope failed: {err}"));

        assert_eq!(roundtrip, scope);
    }

    #[cfg(feature = "serde")]
    #[test]
    fn session_data_pack_id_is_optional() {
        let data = SessionData {
            tenant_ctx: TenantCtx::new(
                "env"
                    .parse()
                    .unwrap_or_else(|err| panic!("parse env failed: {err}")),
                "tenant"
                    .parse()
                    .unwrap_or_else(|err| panic!("parse tenant failed: {err}")),
            ),
            flow_id: "flow-1"
                .parse()
                .unwrap_or_else(|err| panic!("parse flow failed: {err}")),
            pack_id: None,
            cursor: SessionCursor::new("node-1"),
            context_json: "{}".to_owned(),
        };

        let value = serde_json::to_value(&data)
            .unwrap_or_else(|err| panic!("serialize session failed: {err}"));
        assert!(
            value.get("pack_id").is_none(),
            "pack_id should be omitted when None"
        );

        let mut data_with_pack = data.clone();
        data_with_pack.pack_id = Some(
            "greentic.demo.pack"
                .parse()
                .unwrap_or_else(|err| panic!("parse pack id failed: {err}")),
        );

        let value = serde_json::to_value(&data_with_pack)
            .unwrap_or_else(|err| panic!("serialize session failed: {err}"));
        assert!(value.get("pack_id").is_some());

        let object = value
            .as_object()
            .cloned()
            .unwrap_or_else(|| panic!("expected session value to be a JSON object"));
        let roundtrip: SessionData = serde_json::from_value(Value::Object(object))
            .unwrap_or_else(|err| panic!("deserialize session failed: {err}"));
        assert_eq!(roundtrip.pack_id, data_with_pack.pack_id);
    }
}
