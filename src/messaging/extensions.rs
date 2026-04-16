//! Well-known keys for `ChannelMessageEnvelope.extensions`.
//!
//! `extensions` is a `BTreeMap<String, serde_json::Value>` that carries
//! structured, provider-agnostic and provider-native data alongside the
//! envelope. Consumers should prefer these constants over string literals
//! to avoid typos and to allow grep-based auditing.

/// Well-known extension key names.
pub mod ext_keys {
    /// Adaptive Card JSON (object). Replaces the legacy
    /// `metadata["adaptive_card"]` stringly-typed convention.
    pub const ADAPTIVE_CARD: &str = "adaptive_card";

    /// Bot Framework / DirectLine channel-specific object.
    pub const CHANNEL_DATA: &str = "channel_data";

    /// Bot Framework / DirectLine entities array (e.g. mentions).
    pub const ENTITIES: &str = "entities";

    /// Provider-native attachments array (e.g. DirectLine attachments with
    /// inline `content`). Preferred over `envelope.attachments` when the
    /// target provider needs inline content that the URL-only `Attachment`
    /// struct cannot carry.
    pub const ATTACHMENTS: &str = "attachments";

    /// RAG component citations/context payload.
    pub const RAG: &str = "rag";

    /// Bot Framework suggestedActions object.
    pub const SUGGESTED_ACTIONS: &str = "suggested_actions";

    /// Bot Framework speak hint (SSML or plain text).
    pub const SPEAK: &str = "speak";

    /// Bot Framework inputHint hint ("acceptingInput", "expectingInput", ...).
    pub const INPUT_HINT: &str = "input_hint";
}
