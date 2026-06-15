//! Component description schema (v0.6.0).
use alloc::{collections::BTreeMap, string::String, vec::Vec};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use ciborium::value::Value;

use crate::i18n_text::I18nText;
use crate::schemas::common::schema_ir::SchemaIr;

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
    /// Emitted outcomes vocabulary — the routing events a flow may wire from a
    /// node backed by this component (e.g. `on_success`, `on_error`,
    /// `on_submit`). Additive in 0.6.x: descriptors predating this field decode
    /// to an empty list, so older components keep working unchanged.
    #[cfg_attr(feature = "serde", serde(default))]
    pub outcomes: Vec<String>,
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

#[cfg(all(test, feature = "std", feature = "serde"))]
mod tests {
    use super::*;
    use crate::schemas::common::schema_ir::{AdditionalProperties, SchemaIr};

    fn minimal_descriptor() -> ComponentDescribe {
        ComponentDescribe {
            info: ComponentInfo {
                id: "component-x".into(),
                version: "1.0.0".into(),
                role: "runtime".into(),
                display_name: None,
            },
            provided_capabilities: Vec::new(),
            required_capabilities: Vec::new(),
            metadata: BTreeMap::new(),
            operations: Vec::new(),
            config_schema: SchemaIr::Object {
                properties: BTreeMap::new(),
                required: Vec::new(),
                additional: AdditionalProperties::Allow,
            },
            outcomes: Vec::new(),
        }
    }

    // Cascade safety: descriptors produced before `outcomes` existed omit the
    // field entirely. They MUST decode (to an empty list), not error — this is
    // what makes the field additive across every downstream component.
    #[test]
    fn outcomes_absent_decodes_to_empty() -> Result<(), Box<dyn std::error::Error>> {
        let legacy = serde_json::json!({
            "info": { "id": "component-x", "version": "1.0.0", "role": "runtime", "display_name": null },
            "provided_capabilities": [],
            "required_capabilities": [],
            "metadata": {},
            "operations": [],
            "config_schema": { "type": "object" }
        });
        let decoded: ComponentDescribe = serde_json::from_value(legacy)?;
        assert!(decoded.outcomes.is_empty());
        Ok(())
    }

    // Populated outcomes survive the canonical CBOR round trip used by
    // `describe()` (component → bytes → greentic-types).
    #[test]
    fn outcomes_round_trip_through_canonical_cbor() -> Result<(), Box<dyn std::error::Error>> {
        let mut descriptor = minimal_descriptor();
        descriptor.outcomes = vec!["on_success".into(), "on_error".into()];
        let bytes = canonical::to_canonical_cbor_allow_floats(&descriptor)?;
        let decoded: ComponentDescribe = canonical::from_cbor(&bytes)?;
        assert_eq!(
            decoded.outcomes,
            vec!["on_success".to_string(), "on_error".to_string()]
        );
        assert_eq!(decoded, descriptor);
        Ok(())
    }
}
