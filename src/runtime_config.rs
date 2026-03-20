//! Runtime-resolved configuration shared across lifecycle and deployment flows.

use alloc::string::String;

#[cfg(feature = "schemars")]
use schemars::JsonSchema;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Runtime-resolved configuration values exposed to lifecycle tools.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub struct RuntimeConfig {
    /// Canonical public base URL currently resolved for the runtime, if any.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub public_base_url: Option<RuntimePublicBaseUrl>,
}

/// Canonical runtime public URL plus its resolution source.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub struct RuntimePublicBaseUrl {
    /// Resolved public base URL value.
    pub value: String,
    /// How the runtime obtained this URL.
    pub source: RuntimePublicBaseUrlSource,
}

/// Source of the effective runtime public URL.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub enum RuntimePublicBaseUrlSource {
    /// Value came from setup/static configuration.
    Configured,
    /// Value came from a runtime tunnel such as cloudflared or ngrok.
    Tunnel,
    /// Value was derived at runtime from the local listener.
    Derived,
}
