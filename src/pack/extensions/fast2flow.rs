//! Pack extension payload for Fast2Flow per-flow routing metadata.

use alloc::string::String;
use alloc::vec::Vec;

#[cfg(feature = "serde")]
use ciborium::{de::from_reader, ser::into_writer};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Pack extension identifier for Fast2Flow routing metadata (v1).
pub const EXT_FAST2FLOW_V1: &str = "greentic.ext.fast2flow.v1";

/// Fast2Flow extension payload (v1). Carried in the pack manifest under
/// [`EXT_FAST2FLOW_V1`] for packs that also declare the
/// `greentic.cap.fast2flow.v1` capability.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Fast2FlowExtensionV1 {
    /// Schema version for the payload.
    pub schema_version: u32,
    /// Per-flow routing entries projected into Fast2Flow's `flows.json`.
    pub flows: Vec<Fast2FlowFlowEntryV1>,
}

impl Fast2FlowExtensionV1 {
    /// Creates a new fast2flow payload.
    pub fn new(flows: Vec<Fast2FlowFlowEntryV1>) -> Self {
        Self {
            schema_version: 1,
            flows,
        }
    }

    /// Validates schema and per-entry invariants.
    pub fn validate(&self) -> Result<(), Fast2FlowExtensionError> {
        if self.schema_version != 1 {
            return Err(Fast2FlowExtensionError::UnsupportedSchemaVersion(
                self.schema_version,
            ));
        }
        for flow in &self.flows {
            if flow.id.trim().is_empty() {
                return Err(Fast2FlowExtensionError::EmptyFlowId);
            }
            if flow.target.trim().is_empty() {
                return Err(Fast2FlowExtensionError::EmptyTarget {
                    flow_id: flow.id.clone(),
                });
            }
        }
        Ok(())
    }

    /// Converts payload to an extension value suitable for `ExtensionInline::Other`.
    #[cfg(feature = "serde")]
    pub fn to_extension_value(&self) -> Result<serde_json::Value, Fast2FlowExtensionError> {
        serde_json::to_value(self)
            .map_err(|err| Fast2FlowExtensionError::Serialize(err.to_string()))
    }

    /// Parses payload from an extension value.
    #[cfg(feature = "serde")]
    pub fn from_extension_value(
        value: &serde_json::Value,
    ) -> Result<Self, Fast2FlowExtensionError> {
        let decoded: Self = serde_json::from_value(value.clone())
            .map_err(|err| Fast2FlowExtensionError::Deserialize(err.to_string()))?;
        decoded.validate()?;
        Ok(decoded)
    }
}

/// Single per-flow routing entry. Fields mirror Fast2Flow's `FlowDoc`
/// (`fast2flow-contracts::FlowDoc`); `pack_id` is supplied by the
/// materializer from the parent pack manifest.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Fast2FlowFlowEntryV1 {
    /// Flow identifier; matches `PackFlowEntry::id`.
    pub id: String,
    /// Routing target — `pack`, `pack/flow`, or `pack/flow/node`.
    pub target: String,
    /// Human-readable title.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "String::is_empty")
    )]
    pub title: String,
    /// Keywords fed to the routing index.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Vec::is_empty")
    )]
    pub tags: Vec<String>,
    /// Node identifiers inside the flow, for deep dispatch targets.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Vec::is_empty")
    )]
    pub node_ids: Vec<String>,
    /// Example utterances seeding the matcher.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Vec::is_empty")
    )]
    pub utterances: Vec<String>,
}

/// Errors produced while encoding/decoding fast2flow extension payloads.
#[derive(Debug, thiserror::Error)]
pub enum Fast2FlowExtensionError {
    /// Serialization failed.
    #[error("fast2flow extension serialize failed: {0}")]
    Serialize(String),
    /// Deserialization failed.
    #[error("fast2flow extension deserialize failed: {0}")]
    Deserialize(String),
    /// Unsupported schema version.
    #[error("unsupported fast2flow extension schema_version {0}")]
    UnsupportedSchemaVersion(u32),
    /// Flow entry has an empty id.
    #[error("fast2flow flow entry has empty id")]
    EmptyFlowId,
    /// Flow entry has an empty target — Fast2Flow can't route to it.
    #[error("fast2flow flow entry `{flow_id}` has empty target")]
    EmptyTarget {
        /// Flow id with the invalid target.
        flow_id: String,
    },
}

/// Serializes fast2flow extension payload to CBOR bytes.
#[cfg(feature = "serde")]
pub fn encode_fast2flow_extension_v1_to_cbor_bytes(
    payload: &Fast2FlowExtensionV1,
) -> Result<Vec<u8>, Fast2FlowExtensionError> {
    let mut buf = Vec::new();
    into_writer(payload, &mut buf)
        .map_err(|err| Fast2FlowExtensionError::Serialize(err.to_string()))?;
    Ok(buf)
}

/// Deserializes fast2flow extension payload from CBOR bytes.
#[cfg(feature = "serde")]
pub fn decode_fast2flow_extension_v1_from_cbor_bytes(
    bytes: &[u8],
) -> Result<Fast2FlowExtensionV1, Fast2FlowExtensionError> {
    let decoded: Fast2FlowExtensionV1 =
        from_reader(bytes).map_err(|err| Fast2FlowExtensionError::Deserialize(err.to_string()))?;
    decoded.validate()?;
    Ok(decoded)
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::unwrap_used)]
mod tests {
    use super::*;

    fn sample_entry() -> Fast2FlowFlowEntryV1 {
        Fast2FlowFlowEntryV1 {
            id: "refund_flow".into(),
            target: "support/refund_flow".into(),
            title: "Refund request".into(),
            tags: alloc::vec!["refund".into(), "billing".into()],
            node_ids: alloc::vec!["confirm".into()],
            utterances: alloc::vec!["I want a refund".into()],
        }
    }

    #[cfg(feature = "serde")]
    #[test]
    fn round_trip_via_extension_value() {
        let payload = Fast2FlowExtensionV1::new(alloc::vec![sample_entry()]);
        let value = payload.to_extension_value().expect("serialize");
        let decoded = Fast2FlowExtensionV1::from_extension_value(&value).expect("decode");
        assert_eq!(payload, decoded);
    }

    #[cfg(feature = "serde")]
    #[test]
    fn round_trip_via_cbor() {
        let payload = Fast2FlowExtensionV1::new(alloc::vec![Fast2FlowFlowEntryV1 {
            id: "main".into(),
            target: "support".into(),
            title: String::new(),
            tags: Vec::new(),
            node_ids: Vec::new(),
            utterances: Vec::new(),
        }]);
        let bytes = encode_fast2flow_extension_v1_to_cbor_bytes(&payload).expect("encode");
        let decoded = decode_fast2flow_extension_v1_from_cbor_bytes(&bytes).expect("decode");
        assert_eq!(payload, decoded);
    }

    #[test]
    fn validate_rejects_empty_flow_id() {
        let payload = Fast2FlowExtensionV1::new(alloc::vec![Fast2FlowFlowEntryV1 {
            id: "  ".into(),
            target: "x".into(),
            title: String::new(),
            tags: Vec::new(),
            node_ids: Vec::new(),
            utterances: Vec::new(),
        }]);
        assert!(matches!(
            payload.validate(),
            Err(Fast2FlowExtensionError::EmptyFlowId)
        ));
    }

    #[test]
    fn validate_rejects_empty_target() {
        let payload = Fast2FlowExtensionV1::new(alloc::vec![Fast2FlowFlowEntryV1 {
            id: "ok".into(),
            target: "  ".into(),
            title: String::new(),
            tags: Vec::new(),
            node_ids: Vec::new(),
            utterances: Vec::new(),
        }]);
        match payload.validate() {
            Err(Fast2FlowExtensionError::EmptyTarget { flow_id }) => assert_eq!(flow_id, "ok"),
            other => panic!("expected EmptyTarget, got {other:?}"),
        }
    }

    #[test]
    fn validate_rejects_unsupported_schema_version() {
        let mut payload = Fast2FlowExtensionV1::new(Vec::new());
        payload.schema_version = 99;
        assert!(matches!(
            payload.validate(),
            Err(Fast2FlowExtensionError::UnsupportedSchemaVersion(99))
        ));
    }
}
