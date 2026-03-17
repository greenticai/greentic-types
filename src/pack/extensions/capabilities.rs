//! Extension payload for declarative capability offers.

use alloc::string::String;
use alloc::vec::Vec;

#[cfg(feature = "serde")]
use ciborium::{de::from_reader, ser::into_writer};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Pack extension identifier for capability offers (v1).
pub const EXT_CAPABILITIES_V1: &str = "greentic.ext.capabilities.v1";

/// Capabilities extension payload (v1).
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct CapabilitiesExtensionV1 {
    /// Schema version for the payload.
    pub schema_version: u32,
    /// Capability offers declared by the pack.
    pub offers: Vec<CapabilityOfferV1>,
}

impl CapabilitiesExtensionV1 {
    /// Creates a new capabilities payload.
    pub fn new(offers: Vec<CapabilityOfferV1>) -> Self {
        Self {
            schema_version: 1,
            offers,
        }
    }

    /// Validates schema and setup invariants.
    pub fn validate(&self) -> Result<(), CapabilitiesExtensionError> {
        if self.schema_version != 1 {
            return Err(CapabilitiesExtensionError::UnsupportedSchemaVersion(
                self.schema_version,
            ));
        }
        for offer in &self.offers {
            if offer.requires_setup && offer.setup.is_none() {
                return Err(CapabilitiesExtensionError::MissingSetup {
                    offer_id: offer.offer_id.clone(),
                });
            }
            if let Some(setup) = offer.setup.as_ref()
                && setup.qa_ref.trim().is_empty()
            {
                return Err(CapabilitiesExtensionError::InvalidSetupQaRef {
                    offer_id: offer.offer_id.clone(),
                });
            }
        }
        Ok(())
    }

    /// Converts payload to an extension value suitable for `ExtensionInline::Other`.
    #[cfg(feature = "serde")]
    pub fn to_extension_value(&self) -> Result<serde_json::Value, CapabilitiesExtensionError> {
        serde_json::to_value(self)
            .map_err(|err| CapabilitiesExtensionError::Serialize(err.to_string()))
    }

    /// Parses payload from an extension value.
    #[cfg(feature = "serde")]
    pub fn from_extension_value(
        value: &serde_json::Value,
    ) -> Result<Self, CapabilitiesExtensionError> {
        let decoded: Self = serde_json::from_value(value.clone())
            .map_err(|err| CapabilitiesExtensionError::Deserialize(err.to_string()))?;
        decoded.validate()?;
        Ok(decoded)
    }
}

/// Single capability offer in the extension payload.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct CapabilityOfferV1 {
    /// Stable offer identifier used for deterministic tie-breaks.
    pub offer_id: String,
    /// Capability identifier, e.g. `greentic.cap.memory.shortterm`.
    pub cap_id: String,
    /// Offer contract version, e.g. `v1` or semver.
    pub version: String,
    /// Provider operation reference.
    pub provider: CapabilityProviderRefV1,
    /// Optional scope restrictions.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub scope: Option<CapabilityScopeV1>,
    /// Deterministic selection priority (ascending).
    #[cfg_attr(feature = "serde", serde(default))]
    pub priority: i32,
    /// Whether the offer needs setup before runtime use.
    #[cfg_attr(feature = "serde", serde(default))]
    pub requires_setup: bool,
    /// Optional setup metadata.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub setup: Option<CapabilitySetupV1>,
    /// Optional hook applicability metadata.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub applies_to: Option<CapabilityHookAppliesToV1>,
}

/// Provider operation target for a capability offer.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct CapabilityProviderRefV1 {
    /// Component reference inside the pack.
    pub component_ref: String,
    /// Operation exported by the provider component.
    pub op: String,
}

/// Optional scope constraints for a capability offer.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct CapabilityScopeV1 {
    /// Allowed env ids.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Vec::is_empty")
    )]
    pub envs: Vec<String>,
    /// Allowed tenant ids.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Vec::is_empty")
    )]
    pub tenants: Vec<String>,
    /// Allowed team ids.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Vec::is_empty")
    )]
    pub teams: Vec<String>,
}

/// Setup metadata for capability offers requiring setup.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct CapabilitySetupV1 {
    /// Pack-relative QA spec reference.
    pub qa_ref: String,
}

/// Optional applicability constraints for hook capabilities.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct CapabilityHookAppliesToV1 {
    /// Exact operation names (v1).
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Vec::is_empty")
    )]
    pub op_names: Vec<String>,
}

/// Errors produced while encoding/decoding capabilities extension payloads.
#[derive(Debug, thiserror::Error)]
pub enum CapabilitiesExtensionError {
    /// Serialization failed.
    #[error("capabilities extension serialize failed: {0}")]
    Serialize(String),
    /// Deserialization failed.
    #[error("capabilities extension deserialize failed: {0}")]
    Deserialize(String),
    /// Unsupported schema version.
    #[error("unsupported capabilities extension schema_version {0}")]
    UnsupportedSchemaVersion(u32),
    /// Setup section is required when `requires_setup=true`.
    #[error("capabilities extension offer `{offer_id}` requires setup but setup is missing")]
    MissingSetup {
        /// Offer identifier that violated setup requirements.
        offer_id: String,
    },
    /// Setup QA reference must not be empty.
    #[error("capabilities extension offer `{offer_id}` has empty setup.qa_ref")]
    InvalidSetupQaRef {
        /// Offer identifier that contains an invalid QA reference.
        offer_id: String,
    },
    /// Extension payload missing inline data.
    #[error("capabilities extension missing inline payload")]
    MissingInline,
    /// Extension payload used an unexpected inline type.
    #[error("capabilities extension inline payload has unexpected type")]
    UnexpectedInline,
}

/// Serializes capabilities extension payload to CBOR bytes.
#[cfg(feature = "serde")]
pub fn encode_capabilities_extension_v1_to_cbor_bytes(
    payload: &CapabilitiesExtensionV1,
) -> Result<Vec<u8>, CapabilitiesExtensionError> {
    let mut buf = Vec::new();
    into_writer(payload, &mut buf)
        .map_err(|err| CapabilitiesExtensionError::Serialize(err.to_string()))?;
    Ok(buf)
}

/// Deserializes capabilities extension payload from CBOR bytes.
#[cfg(feature = "serde")]
pub fn decode_capabilities_extension_v1_from_cbor_bytes(
    bytes: &[u8],
) -> Result<CapabilitiesExtensionV1, CapabilitiesExtensionError> {
    let decoded: CapabilitiesExtensionV1 = from_reader(bytes)
        .map_err(|err| CapabilitiesExtensionError::Deserialize(err.to_string()))?;
    decoded.validate()?;
    Ok(decoded)
}
