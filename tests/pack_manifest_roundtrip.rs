#![cfg(feature = "serde")]

use std::collections::BTreeMap;

use greentic_types::{
    BootstrapSpec, ComponentCapabilities, ComponentCapability, ComponentManifest,
    ComponentOperation, ComponentProfiles, DeploymentPlan, Flow, FlowComponentRef, FlowId,
    FlowKind, FlowMetadata, InputMapping, Node, OutputMapping, PackDependency, PackFlowEntry,
    PackId, PackKind, PackManifest, PackSignatures, ResourceHints, Routing, SecretFormat,
    SecretRequirement, SecretScope, TelemetryHints, decode_pack_manifest, encode_pack_manifest,
};
use indexmap::IndexMap;
use semver::Version;
use serde_json::Value;

fn sample_flow() -> Flow {
    let mut nodes: IndexMap<_, _, greentic_types::flow::FlowHasher> = IndexMap::default();
    nodes.insert(
        "start".parse().unwrap(),
        Node {
            id: "start".parse().unwrap(),
            component: FlowComponentRef {
                id: "component.router".parse().unwrap(),
                pack_alias: None,
                operation: Some("route".into()),
            },
            input: InputMapping {
                mapping: serde_json::json!({"input": "value"}),
            },
            output: OutputMapping {
                mapping: Value::Null,
            },
            routing: Routing::Branch {
                on_status: BTreeMap::from([("ok".to_string(), "handler".parse().unwrap())]),
                default: Some("end".parse().unwrap()),
            },
            telemetry: TelemetryHints::default(),
        },
    );
    nodes.insert(
        "handler".parse().unwrap(),
        Node {
            id: "handler".parse().unwrap(),
            component: FlowComponentRef {
                id: "component.handler".parse().unwrap(),
                pack_alias: None,
                operation: None,
            },
            input: InputMapping {
                mapping: Value::Null,
            },
            output: OutputMapping {
                mapping: Value::Null,
            },
            routing: Routing::Reply,
            telemetry: TelemetryHints::default(),
        },
    );
    nodes.insert(
        "end".parse().unwrap(),
        Node {
            id: "end".parse().unwrap(),
            component: FlowComponentRef {
                id: "component.end".parse().unwrap(),
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

    Flow {
        schema_version: "flow-v1".into(),
        id: "demo.flow".parse().unwrap(),
        kind: FlowKind::Messaging,
        entrypoints: BTreeMap::from([("default".into(), Value::Null)]),
        nodes,
        metadata: FlowMetadata::default(),
    }
}

fn sample_component(id: &str, supports: Vec<FlowKind>) -> ComponentManifest {
    ComponentManifest {
        id: id.parse().unwrap(),
        version: Version::parse("1.0.0").unwrap(),
        supports,
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

fn sample_secret_requirement() -> SecretRequirement {
    let mut requirement = SecretRequirement::default();
    requirement.key = "TEST_API_KEY".into();
    requirement.required = true;
    requirement.description = Some("API token for integration flows".into());
    requirement.scope = Some(SecretScope {
        env: "staging".into(),
        tenant: "tenant-a".into(),
        team: None,
    });
    requirement.format = Some(SecretFormat::Text);
    requirement.examples = vec!["sk-test-123".into()];
    requirement
}

fn sample_pack_manifest() -> PackManifest {
    let flow = sample_flow();
    PackManifest {
        schema_version: "pack-v1".into(),
        pack_id: PackId::new("vendor.demo.pack").unwrap(),
        name: None,
        version: Version::parse("0.1.0").unwrap(),
        kind: PackKind::Application,
        publisher: "vendor".into(),
        components: vec![
            sample_component("component.router", vec![FlowKind::Messaging]),
            sample_component("component.handler", vec![FlowKind::Messaging]),
            sample_component("component.end", vec![FlowKind::Messaging]),
        ],
        flows: vec![PackFlowEntry {
            id: FlowId::new("demo.flow").unwrap(),
            kind: FlowKind::Messaging,
            flow,
            tags: vec!["demo".into()],
            entrypoints: vec!["default".into()],
        }],
        dependencies: vec![PackDependency {
            alias: "provider.messaging".into(),
            pack_id: PackId::new("vendor.messaging").unwrap(),
            version_req: greentic_types::SemverReq::parse("^1.0").unwrap(),
            required_capabilities: vec!["messaging".into()],
        }],
        capabilities: vec![ComponentCapability {
            name: "messaging".into(),
            description: Some("messaging surface".into()),
        }],
        secret_requirements: vec![sample_secret_requirement()],
        signatures: PackSignatures { signatures: vec![] },
        bootstrap: None,
        extensions: None,
    }
}

fn roundtrip_json<T>(value: &T) -> T
where
    T: serde::Serialize + serde::de::DeserializeOwned + PartialEq + core::fmt::Debug,
{
    let json = serde_json::to_string_pretty(value).expect("serialize json");
    let decoded: T = serde_json::from_str(&json).expect("json roundtrip");
    assert_eq!(&decoded, value);
    decoded
}

#[test]
fn pack_manifest_roundtrip_json_and_cbor() {
    let manifest = sample_pack_manifest();
    let json_value = serde_json::to_value(&manifest).expect("serialize");
    assert!(
        json_value.get("bootstrap").is_none(),
        "bootstrap should be omitted when absent"
    );
    let json_roundtrip = roundtrip_json(&manifest);
    assert_eq!(json_roundtrip, manifest);

    let bytes = encode_pack_manifest(&manifest).expect("encode");
    let decoded = decode_pack_manifest(&bytes).expect("decode");
    assert_eq!(decoded, manifest);
}

fn manifest_with_bootstrap() -> PackManifest {
    let mut manifest = sample_pack_manifest();
    manifest.pack_id = PackId::new("greentic.platform.pack").unwrap();
    manifest.version = Version::parse("0.4.0").unwrap();
    manifest.publisher = "greentic".into();
    manifest.bootstrap = Some(BootstrapSpec {
        install_flow: Some("platform_install".into()),
        upgrade_flow: Some("platform_upgrade".into()),
        installer_component: Some("installer".into()),
    });
    manifest.extensions = None;
    manifest
}

#[test]
fn pack_manifest_roundtrip_with_bootstrap_json_and_cbor() {
    let manifest = manifest_with_bootstrap();
    let json_roundtrip = roundtrip_json(&manifest);
    assert_eq!(json_roundtrip.bootstrap, manifest.bootstrap);

    let bytes = encode_pack_manifest(&manifest).expect("encode");
    let decoded = decode_pack_manifest(&bytes).expect("decode");
    assert_eq!(decoded.bootstrap, manifest.bootstrap);
}

#[test]
fn pack_manifest_cbor_encoding_is_deterministic() {
    let manifest = sample_pack_manifest();
    let first = encode_pack_manifest(&manifest).expect("encode");
    let second = encode_pack_manifest(&manifest).expect("encode");
    assert_eq!(first, second);
}

#[test]
fn pack_manifest_yaml_roundtrip_without_bootstrap() {
    let manifest: PackManifest = serde_yaml_bw::from_str(include_str!(
        "fixtures/pack_manifest_without_bootstrap.yaml"
    ))
    .expect("fixture deserializes");
    assert!(
        manifest.bootstrap.is_none(),
        "bootstrap should default to None"
    );

    let json_value = serde_json::to_value(&manifest).expect("serialize");
    assert!(
        json_value.get("bootstrap").is_none(),
        "bootstrap should stay omitted on serialization"
    );

    let yaml = serde_yaml_bw::to_string(&manifest).expect("serialize yaml");
    let roundtrip: PackManifest = serde_yaml_bw::from_str(&yaml).expect("yaml roundtrip");
    assert_eq!(roundtrip, manifest);
}

#[test]
fn pack_manifest_yaml_roundtrip_with_bootstrap() {
    let manifest: PackManifest =
        serde_yaml_bw::from_str(include_str!("fixtures/pack_manifest_with_bootstrap.yaml"))
            .expect("fixture deserializes");

    let bootstrap = manifest.bootstrap.as_ref().expect("bootstrap present");
    assert_eq!(bootstrap.install_flow.as_deref(), Some("platform_install"));
    assert_eq!(bootstrap.upgrade_flow.as_deref(), Some("platform_upgrade"));
    assert_eq!(bootstrap.installer_component.as_deref(), Some("installer"));

    let yaml = serde_yaml_bw::to_string(&manifest).expect("serialize yaml");
    let roundtrip: PackManifest = serde_yaml_bw::from_str(&yaml).expect("yaml roundtrip");
    assert_eq!(roundtrip.bootstrap, manifest.bootstrap);

    let json_value = serde_json::to_value(&roundtrip).expect("serialize to json");
    let bootstrap_value = json_value
        .get("bootstrap")
        .and_then(|value| value.as_object())
        .expect("bootstrap should serialize");
    assert_eq!(
        bootstrap_value.get("install_flow"),
        Some(&serde_json::Value::String("platform_install".into()))
    );
    assert_eq!(
        bootstrap_value.get("upgrade_flow"),
        Some(&serde_json::Value::String("platform_upgrade".into()))
    );
    assert_eq!(
        bootstrap_value.get("installer_component"),
        Some(&serde_json::Value::String("installer".into()))
    );
}

#[test]
fn deployment_plan_roundtrip_json() {
    let doc = r#"
{
  "pack_id": "vendor.demo",
  "pack_version": "1.2.3",
  "tenant": "tenant-a",
  "environment": "staging",
  "runners": [
    {
      "name": "demo-runner",
      "replicas": 2,
      "capabilities": {
        "can_run_flows": ["flow-a"]
      }
    }
  ],
  "messaging": {
    "logical_cluster": "cluster-1",
    "subjects": [
      {
        "name": "events",
        "purpose": "eventing",
        "durable": true,
        "extra": {}
      }
    ],
    "extra": {}
  },
  "channels": [
    {
      "name": "webchat",
      "flow_id": "demo.flow",
      "kind": "webchat",
      "config": {}
    }
  ],
  "secrets": [
    {
      "key": "API_KEY",
      "required": true,
      "description": "primary api key",
      "scope": {
        "env": "staging",
        "tenant": "tenant-a",
        "team": null
      },
      "format": "text"
    }
  ],
  "oauth": [
    {
      "provider_id": "generic",
      "logical_client_id": "client-a",
      "redirect_path": "/oauth/callback",
      "extra": {}
    }
  ],
  "telemetry": {
    "required": true,
    "suggested_endpoint": "https://telemetry.local",
    "extra": {}
  },
  "extra": {}
}
"#;

    let plan: DeploymentPlan = serde_json::from_str(doc).expect("valid json");
    let roundtrip = roundtrip_json(&plan);
    assert_eq!(roundtrip.channels.len(), 1);
    let first_secret = serde_json::to_value(&roundtrip.secrets[0]).expect("serialize secret");
    assert_eq!(first_secret["key"], "API_KEY");
}

#[test]
fn component_manifest_deserializes_without_dev_flows() {
    let manifest = sample_component("component.devless", vec![FlowKind::Messaging]);
    let json = serde_json::to_string_pretty(&manifest).expect("serialize");
    assert!(
        !json.contains("dev_flows"),
        "empty dev_flows should be skipped"
    );

    let decoded: ComponentManifest = serde_json::from_str(&json).expect("deserialize");
    assert!(decoded.dev_flows.is_empty());
}

#[test]
fn component_manifest_with_dev_flows_roundtrips() {
    let manifest_json = serde_json::json!({
        "id": "component.devflow",
        "version": "1.0.0",
        "supports": ["messaging"],
        "world": "test:world@1.0.0",
        "profiles": {
            "default": "default",
            "supported": ["default"]
        },
        "capabilities": {
            "wasi": {},
            "host": {}
        },
        "operations": [
            {
                "name": "handle",
                "input_schema": null,
                "output_schema": null
            }
        ],
        "resources": {},
        "dev_flows": {
            "default": {
                "graph": {
                    "schema_version": "flow-ir-1"
                }
            }
        }
    });

    let decoded: ComponentManifest =
        serde_json::from_value(manifest_json).expect("manifest with dev_flows");
    let flow = decoded
        .dev_flows
        .get(&FlowId::new("default").unwrap())
        .expect("dev flow present");
    assert_eq!(flow.format, "flow-ir-json");
    assert_eq!(
        flow.graph,
        serde_json::json!({"schema_version": "flow-ir-1"})
    );

    let roundtrip = roundtrip_json(&decoded);
    assert_eq!(roundtrip.dev_flows.len(), 1);
}
