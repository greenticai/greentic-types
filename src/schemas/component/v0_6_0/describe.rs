//! Component description schema (v0.6.0).
use alloc::{collections::BTreeMap, string::String, vec::Vec};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use ciborium::value::Value;

use crate::component::{
    ComponentCapabilities, ComponentConfigurators, ComponentProfiles, ResourceHints,
};
use crate::flow::FlowKind;
use crate::i18n_text::I18nText;
use crate::schemas::common::schema_ir::SchemaIr;
use crate::secrets::SecretRequirement;

#[cfg(all(feature = "std", feature = "serde"))]
use crate::cbor::canonical;
#[cfg(all(feature = "std", feature = "serde"))]
use sha2::{Digest, Sha256};

/// Component metadata.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub struct ComponentInfo {
    /// Stable component identifier.
    pub id: String,
    /// Semantic version string.
    pub version: String,
    /// Component role (runtime/provider/tool/etc.).
    pub role: String,
    /// Optional display name.
    pub display_name: Option<I18nText>,
}

/// Component description payload.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub struct ComponentDescribe {
    /// Core component metadata.
    pub info: ComponentInfo,
    /// Capabilities provided by the component.
    pub provided_capabilities: Vec<String>,
    /// Capabilities required by the component.
    pub required_capabilities: Vec<String>,
    /// Optional metadata payload.
    pub metadata: BTreeMap<String, Value>,
    /// MCP-equivalent tool list.
    pub operations: Vec<ComponentOperation>,
    /// Component-level config schema (authoritative).
    pub config_schema: SchemaIr,

    // -- Fields below are populated from describe() to replace component.manifest.json --
    /// Flow kinds this component can participate in.
    #[cfg_attr(feature = "serde", serde(default))]
    pub supports: Vec<FlowKind>,

    /// Structured capability contract (WASI + host).
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub capabilities: Option<ComponentCapabilities>,

    /// Profile metadata (default + supported list).
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub profiles: Option<ComponentProfiles>,

    /// Optional configurator flows.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub configurators: Option<ComponentConfigurators>,

    /// Resource usage hints for deployers/schedulers.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub resources: Option<ResourceHints>,

    /// Secret requirements declared by the component.
    #[cfg_attr(feature = "serde", serde(default))]
    pub secret_requirements: Vec<SecretRequirement>,
}

/// Component operation entry.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub struct ComponentOperation {
    /// Operation identifier (e.g. "run").
    pub id: String,
    /// Optional display name.
    pub display_name: Option<I18nText>,
    /// Input schema.
    pub input: ComponentRunInput,
    /// Output schema.
    pub output: ComponentRunOutput,
    /// Default values.
    #[cfg_attr(feature = "serde", serde(default))]
    pub defaults: BTreeMap<String, Value>,
    /// Redaction rules.
    #[cfg_attr(feature = "serde", serde(default))]
    pub redactions: Vec<RedactionRule>,
    /// Validation constraints.
    #[cfg_attr(feature = "serde", serde(default))]
    pub constraints: BTreeMap<String, Value>,
    /// Stable hash computed from typed SchemaIR values.
    pub schema_hash: String,
}

/// Run input schema wrapper.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub struct ComponentRunInput {
    /// Input schema.
    pub schema: SchemaIr,
}

/// Run output schema wrapper.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub struct ComponentRunOutput {
    /// Output schema.
    pub schema: SchemaIr,
}

/// Redaction rule.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RedactionRule {
    /// JSON Pointer path (e.g. "/api_key").
    pub json_pointer: String,
    /// Redaction kind.
    pub kind: RedactionKind,
}

/// Redaction kind.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
pub enum RedactionKind {
    /// Never display; treat as secret material.
    Secret,
    /// Display masked.
    Mask,
    /// Remove field from output/logs.
    Drop,
}

/// Compute the schema hash for an operation, including config schema.
#[cfg(all(feature = "std", feature = "serde"))]
pub fn schema_hash(
    input: &SchemaIr,
    output: &SchemaIr,
    config: &SchemaIr,
) -> canonical::Result<String> {
    #[derive(Serialize)]
    struct HashMaterial<'a> {
        input: &'a SchemaIr,
        output: &'a SchemaIr,
        config: &'a SchemaIr,
    }

    let material = HashMaterial {
        input,
        output,
        config,
    };
    let bytes = canonical::to_canonical_cbor_allow_floats(&material)?;
    let digest = Sha256::digest(bytes.as_slice());
    let mut hex = String::with_capacity(digest.len() * 2);
    for byte in digest {
        hex.push_str(&format!("{byte:02x}"));
    }
    Ok(hex)
}
