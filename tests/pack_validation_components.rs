#![cfg(feature = "serde")]

use std::collections::BTreeMap;

use greentic_types::pack::extensions::component_sources::{
    ArtifactLocationV1, ComponentSourceEntryV1, ComponentSourcesV1, EXT_COMPONENT_SOURCES_V1,
    ResolvedComponentV1,
};
use greentic_types::pack_manifest::{ExtensionInline, ExtensionRef};
use greentic_types::{
    ComponentCapabilities, ComponentManifest, ComponentOperation, ComponentProfiles, Flow,
    FlowComponentRef, FlowId, FlowKind, FlowMetadata, InputMapping, Node, OutputMapping,
    PackFlowEntry, PackId, PackKind, PackManifest, PackSignatures, ResourceHints, Routing,
    TelemetryHints, validate_pack_manifest_core,
};
use indexmap::IndexMap;
use semver::Version;
use serde_json::Value;

fn flow_with_component(component_id: &str) -> PackFlowEntry {
    let mut nodes: IndexMap<_, _, greentic_types::flow::FlowHasher> = IndexMap::default();
    nodes.insert(
        "start".parse().unwrap(),
        Node {
            id: "start".parse().unwrap(),
            component: FlowComponentRef {
                id: component_id.parse().unwrap(),
                pack_alias: None,
                operation: None,
            },
            input: InputMapping {
                mapping: Value::Null,
            },
            output: OutputMapping {
                mapping: Value::Null,
            },
            routing: Routing::End,
            telemetry: TelemetryHints::default(),
        },
    );

    let flow = Flow {
        schema_version: "flow-v1".into(),
        id: FlowId::new("main").unwrap(),
        kind: FlowKind::Messaging,
        entrypoints: BTreeMap::from([("default".into(), Value::Null)]),
        nodes,
        metadata: FlowMetadata::default(),
    };

    PackFlowEntry {
        id: FlowId::new("main").unwrap(),
        kind: FlowKind::Messaging,
        flow,
        tags: Vec::new(),
        entrypoints: vec!["default".into()],
    }
}

fn base_manifest() -> PackManifest {
    PackManifest {
        schema_version: "pack-v1".into(),
        pack_id: PackId::new("dev.local.validation").unwrap(),
        name: None,
        version: Version::parse("0.1.0").unwrap(),
        kind: PackKind::Application,
        publisher: "tests".into(),
        components: Vec::new(),
        flows: Vec::new(),
        dependencies: Vec::new(),
        capabilities: Vec::new(),
        secret_requirements: Vec::new(),
        signatures: PackSignatures {
            signatures: Vec::new(),
        },
        bootstrap: None,
        extensions: None,
    }
}

fn sample_component(id: &str) -> ComponentManifest {
    ComponentManifest {
        id: id.parse().unwrap(),
        version: Version::parse("1.0.0").unwrap(),
        supports: vec![FlowKind::Messaging],
        world: "test:world@1.0.0".into(),
        profiles: ComponentProfiles {
            default: Some("default".into()),
            supported: vec!["default".into()],
        },
        capabilities: ComponentCapabilities::default(),
        configurators: None,
        operations: vec![ComponentOperation {
            name: "handle".into(),
            input_schema: Value::Null,
            output_schema: Value::Null,
        }],
        config_schema: None,
        resources: ResourceHints::default(),
        dev_flows: BTreeMap::new(),
    }
}

#[test]
fn flow_component_resolves_via_component_sources() {
    let mut manifest = base_manifest();
    manifest.flows = vec![flow_with_component("templates")];

    let sources = ComponentSourcesV1::new(vec![ComponentSourceEntryV1 {
        name: "templates".into(),
        component_id: None,
        source: "oci://ghcr.io/example/templates@sha256:abc"
            .parse()
            .unwrap(),
        resolved: ResolvedComponentV1 {
            digest: "sha256:abc".into(),
            signature: None,
            signed_by: None,
        },
        artifact: ArtifactLocationV1::Inline {
            wasm_path: "components/templates.wasm".into(),
            manifest_path: None,
        },
        licensing_hint: None,
        metering_hint: None,
    }]);
    let payload = sources.to_extension_value().expect("extension payload");
    manifest.extensions = Some(BTreeMap::from([(
        EXT_COMPONENT_SOURCES_V1.to_string(),
        ExtensionRef {
            kind: EXT_COMPONENT_SOURCES_V1.to_string(),
            version: "1.0.0".into(),
            digest: None,
            location: None,
            inline: Some(ExtensionInline::Other(payload)),
        },
    )]));

    let diagnostics = validate_pack_manifest_core(&manifest);
    assert!(
        diagnostics
            .iter()
            .all(|diag| diag.code != "PACK_FLOW_COMPONENT_MISSING"),
        "component sources should satisfy component references"
    );
    assert!(
        diagnostics
            .iter()
            .any(|diag| diag.code == "PACK_COMPONENT_NOT_EXPLICIT"),
        "should warn when component is resolved via component sources only"
    );
}

#[test]
fn flow_component_missing_without_sources_or_components() {
    let mut manifest = base_manifest();
    manifest.flows = vec![flow_with_component("missing")];

    let diagnostics = validate_pack_manifest_core(&manifest);
    assert!(
        diagnostics
            .iter()
            .any(|diag| diag.code == "PACK_FLOW_COMPONENT_MISSING"),
        "missing component references should be rejected"
    );
}

#[test]
fn flow_component_resolves_via_manifest_components() {
    let mut manifest = base_manifest();
    manifest.components = vec![sample_component("explicit")];
    manifest.flows = vec![flow_with_component("explicit")];

    let diagnostics = validate_pack_manifest_core(&manifest);
    assert!(
        diagnostics
            .iter()
            .all(|diag| diag.code != "PACK_FLOW_COMPONENT_MISSING"),
        "explicit components should satisfy component references"
    );
    assert!(
        diagnostics
            .iter()
            .all(|diag| diag.code != "PACK_COMPONENT_NOT_EXPLICIT"),
        "explicit components should not warn"
    );
}
