//! Provider manifest and index data structures.
//!
//! These types model the JSON returned by provider-core `describe()` and the provider index
//! entries used by store, deployer, and runner components.

use alloc::collections::{BTreeMap, BTreeSet};
use alloc::{format, string::String, vec::Vec};

#[cfg(feature = "schemars")]
use schemars::JsonSchema;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{ErrorCode, GResult, GreenticError};

/// Canonical provider extension identifier stored in pack manifests.
pub const PROVIDER_EXTENSION_ID: &str = "greentic.provider-extension.v1";

/// Manifest describing a provider returned by `describe()`.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub struct ProviderManifest {
    /// Provider type identifier (string-based to avoid enum coupling).
    pub provider_type: String,
    /// Capabilities advertised by the provider.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Vec::is_empty")
    )]
    pub capabilities: Vec<String>,
    /// Operations exposed by the provider.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Vec::is_empty")
    )]
    pub ops: Vec<String>,
    /// Optional JSON Schema reference for configuration.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub config_schema_ref: Option<String>,
    /// Optional JSON Schema reference for provider state.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub state_schema_ref: Option<String>,
}

/// Runtime binding for a provider implementation.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub struct ProviderRuntimeRef {
    /// Component identifier for the provider runtime.
    pub component_ref: String,
    /// Exported function implementing the provider runtime.
    pub export: String,
    /// WIT world for the provider runtime.
    pub world: String,
}

/// Provider declaration stored in indexes and extension payloads.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub struct ProviderDecl {
    /// Provider type identifier (string-based to avoid enum coupling).
    pub provider_type: String,
    /// Capabilities advertised by the provider.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Vec::is_empty")
    )]
    pub capabilities: Vec<String>,
    /// Operations exposed by the provider.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Vec::is_empty")
    )]
    pub ops: Vec<String>,
    /// JSON Schema reference for configuration.
    pub config_schema_ref: String,
    /// Optional JSON Schema reference for provider state.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub state_schema_ref: Option<String>,
    /// Runtime binding information for the provider.
    pub runtime: ProviderRuntimeRef,
    /// Optional documentation reference.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub docs_ref: Option<String>,
}

/// Inline extension payload embedding provider declarations.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub struct ProviderExtensionInline {
    /// Providers included in the extension payload.
    pub providers: Vec<ProviderDecl>,
    /// Additional fields preserved for forward-compatibility.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "BTreeMap::is_empty", flatten)
    )]
    pub additional_fields: BTreeMap<String, Value>,
}

impl ProviderExtensionInline {
    /// Performs basic structural validation without provider-specific semantics.
    pub fn validate_basic(&self) -> GResult<()> {
        let mut seen = BTreeSet::new();
        for provider in &self.providers {
            if provider.provider_type.is_empty() {
                return Err(GreenticError::new(
                    ErrorCode::InvalidInput,
                    "ProviderDecl.provider_type must not be empty",
                ));
            }
            if !seen.insert(&provider.provider_type) {
                return Err(GreenticError::new(
                    ErrorCode::InvalidInput,
                    format!(
                        "duplicate provider_type '{}' in ProviderExtensionInline",
                        provider.provider_type
                    ),
                ));
            }
            if provider.runtime.component_ref.trim().is_empty()
                || provider.runtime.export.trim().is_empty()
                || provider.runtime.world.trim().is_empty()
            {
                return Err(GreenticError::new(
                    ErrorCode::InvalidInput,
                    format!(
                        "runtime fields must be set for provider '{}'",
                        provider.provider_type
                    ),
                ));
            }
        }
        Ok(())
    }
}
