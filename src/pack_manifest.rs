//! Canonical pack manifest (.gtpack) representation embedding flows and components.

use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;

use semver::Version;

use crate::pack::extensions::capabilities::{
    CapabilitiesExtensionError, CapabilitiesExtensionV1, EXT_CAPABILITIES_V1,
};
use crate::pack::extensions::component_sources::{
    ComponentSourcesError, ComponentSourcesV1, EXT_COMPONENT_SOURCES_V1,
};
use crate::{
    ComponentManifest, Flow, FlowId, FlowKind, PROVIDER_EXTENSION_ID, PackId,
    ProviderExtensionInline, SecretRequirement, SemverReq, Signature,
};

#[cfg(feature = "schemars")]
use schemars::JsonSchema;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg(feature = "schemars")]
fn empty_secret_requirements() -> Vec<SecretRequirement> {
    Vec::new()
}

pub(crate) fn extensions_is_empty(value: &Option<BTreeMap<String, ExtensionRef>>) -> bool {
    value.as_ref().is_none_or(BTreeMap::is_empty)
}

/// Hint describing the primary purpose of a pack.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub enum PackKind {
    /// Application packs.
    Application,
    /// Provider packs exporting components.
    Provider,
    /// Infrastructure packs.
    Infrastructure,
    /// Library packs.
    Library,
}

/// Pack manifest describing bundled flows and components.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(
    feature = "schemars",
    derive(JsonSchema),
    schemars(
        title = "Greentic PackManifest v1",
        description = "Canonical pack manifest embedding flows, components, dependencies and signatures.",
        rename = "greentic.pack-manifest.v1"
    )
)]
pub struct PackManifest {
    /// Schema version for the pack manifest.
    pub schema_version: String,
    /// Logical pack identifier.
    pub pack_id: PackId,
    /// Optional human-readable name from `pack.yaml`.
    #[cfg_attr(
        feature = "schemars",
        schemars(default, description = "Optional pack name")
    )]
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub name: Option<String>,
    /// Pack semantic version.
    #[cfg_attr(
        feature = "schemars",
        schemars(with = "String", description = "SemVer version")
    )]
    pub version: Version,
    /// Pack kind hint.
    pub kind: PackKind,
    /// Pack publisher.
    pub publisher: String,
    /// Component descriptors bundled within the pack.
    #[cfg_attr(feature = "serde", serde(default))]
    pub components: Vec<ComponentManifest>,
    /// Flow entries embedded in the pack.
    #[cfg_attr(feature = "serde", serde(default))]
    pub flows: Vec<PackFlowEntry>,
    /// Pack dependencies.
    #[cfg_attr(feature = "serde", serde(default))]
    pub dependencies: Vec<PackDependency>,
    /// Capability declarations for the pack.
    #[cfg_attr(feature = "serde", serde(default))]
    pub capabilities: Vec<ComponentCapability>,
    /// Pack-level secret requirements.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Vec::is_empty")
    )]
    #[cfg_attr(feature = "schemars", schemars(default = "empty_secret_requirements"))]
    pub secret_requirements: Vec<SecretRequirement>,
    /// Pack signatures.
    #[cfg_attr(feature = "serde", serde(default))]
    pub signatures: PackSignatures,
    /// Optional bootstrap/install hints for platform-controlled packs.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub bootstrap: Option<BootstrapSpec>,
    /// Optional extension descriptors for provider-specific metadata.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "extensions_is_empty")
    )]
    pub extensions: Option<BTreeMap<String, ExtensionRef>>,
}

/// Flow entry embedded in a pack.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub struct PackFlowEntry {
    /// Flow identifier.
    pub id: FlowId,
    /// Flow kind.
    pub kind: FlowKind,
    /// Flow definition.
    pub flow: Flow,
    /// Flow tags.
    #[cfg_attr(feature = "serde", serde(default))]
    pub tags: Vec<String>,
    /// Additional entrypoint identifiers for discoverability.
    #[cfg_attr(feature = "serde", serde(default))]
    pub entrypoints: Vec<String>,
}

/// Dependency entry referencing another pack.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub struct PackDependency {
    /// Local alias for the dependency.
    pub alias: String,
    /// Referenced pack identifier.
    pub pack_id: PackId,
    /// Required version.
    pub version_req: SemverReq,
    /// Required capabilities.
    #[cfg_attr(feature = "serde", serde(default))]
    pub required_capabilities: Vec<String>,
}

/// Named capability advertised by a pack or component collection.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub struct ComponentCapability {
    /// Capability name.
    pub name: String,
    /// Optional description or metadata.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub description: Option<String>,
}

/// Signature bundle accompanying a pack manifest.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub struct PackSignatures {
    /// Optional detached signatures.
    #[cfg_attr(feature = "serde", serde(default))]
    pub signatures: Vec<Signature>,
}

/// Optional bootstrap/install hints for platform-managed packs.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub struct BootstrapSpec {
    /// Flow to run during initial install/bootstrap.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub install_flow: Option<String>,
    /// Flow to run when upgrading an existing install.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub upgrade_flow: Option<String>,
    /// Component responsible for install/upgrade orchestration.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub installer_component: Option<String>,
}

/// Inline payload for a pack extension entry.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(untagged))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub enum ExtensionInline {
    /// Provider extension payload embedding provider declarations.
    Provider(ProviderExtensionInline),
    /// Arbitrary inline payload for unknown extensions.
    Other(serde_json::Value),
}

impl ExtensionInline {
    /// Returns the provider inline payload if present.
    pub fn as_provider_inline(&self) -> Option<&ProviderExtensionInline> {
        match self {
            ExtensionInline::Provider(value) => Some(value),
            ExtensionInline::Other(_) => None,
        }
    }

    /// Returns a mutable provider inline payload if present.
    pub fn as_provider_inline_mut(&mut self) -> Option<&mut ProviderExtensionInline> {
        match self {
            ExtensionInline::Provider(value) => Some(value),
            ExtensionInline::Other(_) => None,
        }
    }
}

/// External extension reference embedded in a pack manifest.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub struct ExtensionRef {
    /// Extension kind identifier, e.g. `greentic.provider-extension.v1` (only this ID is supported for provider metadata; other keys are treated as unknown extensions).
    pub kind: String,
    /// Extension version as a string to avoid semver crate coupling.
    pub version: String,
    /// Optional digest pin for the referenced extension payload.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub digest: Option<String>,
    /// Optional remote or local location for the extension payload.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub location: Option<String>,
    /// Optional inline extension payload for small metadata blobs.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub inline: Option<ExtensionInline>,
}

impl PackManifest {
    /// Returns the inline provider extension payload if present.
    pub fn provider_extension_inline(&self) -> Option<&ProviderExtensionInline> {
        self.extensions
            .as_ref()
            .and_then(|extensions| extensions.get(PROVIDER_EXTENSION_ID))
            .and_then(|extension| extension.inline.as_ref())
            .and_then(ExtensionInline::as_provider_inline)
    }

    /// Returns a mutable inline provider extension payload if present.
    pub fn provider_extension_inline_mut(&mut self) -> Option<&mut ProviderExtensionInline> {
        self.extensions
            .as_mut()
            .and_then(|extensions| extensions.get_mut(PROVIDER_EXTENSION_ID))
            .and_then(|extension| extension.inline.as_mut())
            .map(|inline| {
                if let ExtensionInline::Other(value) = inline {
                    let parsed = serde_json::from_value(value.clone())
                        .unwrap_or_else(|_| ProviderExtensionInline::default());
                    *inline = ExtensionInline::Provider(parsed);
                }
                inline
            })
            .and_then(ExtensionInline::as_provider_inline_mut)
    }

    /// Ensures the provider extension entry exists and returns its inline payload.
    pub fn ensure_provider_extension_inline(&mut self) -> &mut ProviderExtensionInline {
        let extensions = self.extensions.get_or_insert_with(BTreeMap::new);
        let entry = extensions
            .entry(PROVIDER_EXTENSION_ID.to_string())
            .or_insert_with(|| ExtensionRef {
                kind: PROVIDER_EXTENSION_ID.to_string(),
                version: "1.0.0".to_string(),
                digest: None,
                location: None,
                inline: Some(ExtensionInline::Provider(ProviderExtensionInline::default())),
            });
        if entry.inline.is_none() {
            entry.inline = Some(ExtensionInline::Provider(ProviderExtensionInline::default()));
        }
        let inline = entry
            .inline
            .get_or_insert_with(|| ExtensionInline::Provider(ProviderExtensionInline::default()));
        if let ExtensionInline::Other(value) = inline {
            let parsed = serde_json::from_value(value.clone())
                .unwrap_or_else(|_| ProviderExtensionInline::default());
            *inline = ExtensionInline::Provider(parsed);
        }
        match inline {
            ExtensionInline::Provider(inline) => inline,
            ExtensionInline::Other(_) => unreachable!("provider inline should be initialised"),
        }
    }

    /// Returns the component sources extension payload if present.
    #[cfg(feature = "serde")]
    pub fn get_component_sources_v1(
        &self,
    ) -> Result<Option<ComponentSourcesV1>, ComponentSourcesError> {
        let extension = self
            .extensions
            .as_ref()
            .and_then(|extensions| extensions.get(EXT_COMPONENT_SOURCES_V1));
        let inline = match extension.and_then(|entry| entry.inline.as_ref()) {
            Some(ExtensionInline::Other(value)) => value,
            Some(_) => return Err(ComponentSourcesError::UnexpectedInline),
            None => return Ok(None),
        };
        let payload = ComponentSourcesV1::from_extension_value(inline)?;
        Ok(Some(payload))
    }

    /// Sets the component sources extension payload.
    #[cfg(feature = "serde")]
    pub fn set_component_sources_v1(
        &mut self,
        sources: ComponentSourcesV1,
    ) -> Result<(), ComponentSourcesError> {
        sources.validate_schema_version()?;
        let inline = sources.to_extension_value()?;
        let extensions = self.extensions.get_or_insert_with(BTreeMap::new);
        extensions.insert(
            EXT_COMPONENT_SOURCES_V1.to_string(),
            ExtensionRef {
                kind: EXT_COMPONENT_SOURCES_V1.to_string(),
                version: "1.0.0".to_string(),
                digest: None,
                location: None,
                inline: Some(ExtensionInline::Other(inline)),
            },
        );
        Ok(())
    }

    /// Returns the capabilities extension payload if present.
    #[cfg(feature = "serde")]
    pub fn get_capabilities_extension_v1(
        &self,
    ) -> Result<Option<CapabilitiesExtensionV1>, CapabilitiesExtensionError> {
        let extension = self
            .extensions
            .as_ref()
            .and_then(|extensions| extensions.get(EXT_CAPABILITIES_V1));
        let inline = match extension.and_then(|entry| entry.inline.as_ref()) {
            Some(ExtensionInline::Other(value)) => value,
            Some(_) => return Err(CapabilitiesExtensionError::UnexpectedInline),
            None => return Ok(None),
        };
        let payload = CapabilitiesExtensionV1::from_extension_value(inline)?;
        Ok(Some(payload))
    }

    /// Sets the capabilities extension payload.
    #[cfg(feature = "serde")]
    pub fn set_capabilities_extension_v1(
        &mut self,
        capabilities: CapabilitiesExtensionV1,
    ) -> Result<(), CapabilitiesExtensionError> {
        capabilities.validate()?;
        let inline = capabilities.to_extension_value()?;
        let extensions = self.extensions.get_or_insert_with(BTreeMap::new);
        extensions.insert(
            EXT_CAPABILITIES_V1.to_string(),
            ExtensionRef {
                kind: EXT_CAPABILITIES_V1.to_string(),
                version: "1.0.0".to_string(),
                digest: None,
                location: None,
                inline: Some(ExtensionInline::Other(inline)),
            },
        );
        Ok(())
    }
}
