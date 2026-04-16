//! Generic channel messaging envelope shared across providers.

use alloc::{collections::BTreeMap, string::String, vec::Vec};

#[cfg(feature = "schemars")]
use schemars::JsonSchema;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::{ReplyScope, TenantCtx};

/// Message actor (sender/initiator).
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub struct Actor {
    /// Actor identifier in provider space (e.g., slack user id, webex person id).
    pub id: String,
    /// Optional actor kind (e.g. "user", "bot", "system").
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub kind: Option<String>,
}

/// Outbound destination for egress providers.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub struct Destination {
    /// Destination identifier (provider specific; may be composite e.g. "teamId:channelId").
    pub id: String,
    /// Optional destination kind (e.g. "chat", "room", "user", "channel", "email", "phone").
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub kind: Option<String>,
}

/// Collection of metadata entries associated with a channel message.
pub type MessageMetadata = BTreeMap<String, String>;

/// Generic attachment referenced by a channel message.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub struct Attachment {
    /// MIME type of the attachment (for example `image/png`).
    pub mime_type: String,
    /// URL pointing at the attachment payload.
    pub url: String,
    /// Optional display name for the attachment.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub name: Option<String>,
    /// Optional attachment size in bytes.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub size_bytes: Option<u64>,
}

/// Envelope for channel messages exchanged with adapters.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub struct ChannelMessageEnvelope {
    /// Stable identifier for the message.
    pub id: String,
    /// Tenant context propagated with the message.
    pub tenant: TenantCtx,
    /// Abstract channel identifier or type.
    pub channel: String,
    /// Conversation or thread identifier.
    pub session_id: String,
    /// Optional reply scope that can be used for resumption.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub reply_scope: Option<ReplyScope>,
    /// Optional actor (sender/initiator) associated with the message (primarily ingress).
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub from: Option<Actor>,
    /// Outbound destinations for egress. Empty means “unspecified” and may be satisfied by provider config defaults.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Vec::is_empty")
    )]
    pub to: Vec<Destination>,
    /// Optional correlation identifier used by outbound adapters.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub correlation_id: Option<String>,
    /// Optional text content.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub text: Option<String>,
    /// Attachments included with the message.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Vec::is_empty")
    )]
    pub attachments: Vec<Attachment>,
    /// Free-form metadata for adapters and flows.
    #[cfg_attr(feature = "serde", serde(default))]
    pub metadata: MessageMetadata,
    /// Structured provider-agnostic and provider-native extension data.
    ///
    /// Use well-known keys from [`extensions::ext_keys`] to avoid typos.
    /// Values are typed JSON (`serde_json::Value`) — no re-serialization
    /// needed at provider boundaries.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "BTreeMap::is_empty")
    )]
    pub extensions: alloc::collections::BTreeMap<String, serde_json::Value>,
}

pub mod extensions;
pub mod rendering;
pub mod universal_dto;
