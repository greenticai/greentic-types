use std::collections::BTreeMap;
use std::path::Path;

use greentic_types::{
    FLOW_RESOLVE_SUMMARY_SCHEMA_VERSION, FlowResolveSummaryManifestV1,
    FlowResolveSummarySourceRefV1, FlowResolveSummaryV1, NodeResolveSummaryV1,
    resolve_summary_path_for_flow, validate_flow_resolve_summary,
};
use semver::Version;

#[test]
fn flow_resolve_summary_roundtrip() {
    let doc = FlowResolveSummaryV1 {
        schema_version: FLOW_RESOLVE_SUMMARY_SCHEMA_VERSION,
        flow: "main.ygtc".into(),
        nodes: BTreeMap::from([(
            "node".to_string(),
            NodeResolveSummaryV1 {
                component_id: "greentic.demo.component".parse().expect("component id"),
                source: FlowResolveSummarySourceRefV1::Oci {
                    r#ref: "ghcr.io/greentic/demo/component:1.2.3".into(),
                },
                digest: "sha256:deadbeef".into(),
                manifest: Some(FlowResolveSummaryManifestV1 {
                    world: "greentic:component/world".into(),
                    version: Version::parse("1.2.3").expect("version"),
                }),
            },
        )]),
    };

    let json = serde_json::to_string_pretty(&doc).expect("serialize");
    let decoded: FlowResolveSummaryV1 = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(decoded, doc);
}

#[test]
fn flow_resolve_summary_rejects_absolute_local_paths() {
    let doc = FlowResolveSummaryV1 {
        schema_version: FLOW_RESOLVE_SUMMARY_SCHEMA_VERSION,
        flow: "main.ygtc".into(),
        nodes: BTreeMap::from([(
            "node".to_string(),
            NodeResolveSummaryV1 {
                component_id: "greentic.demo.component".parse().expect("component id"),
                source: FlowResolveSummarySourceRefV1::Local {
                    path: "/abs/component.wasm".into(),
                },
                digest: "sha256:deadbeef".into(),
                manifest: None,
            },
        )]),
    };

    let err = validate_flow_resolve_summary(&doc).expect_err("should reject absolute path");
    assert_eq!(err.code, greentic_types::ErrorCode::InvalidInput);
}

#[test]
fn resolve_summary_path_helper_uses_summary_suffix() {
    let flow_path = Path::new("flows/main.ygtc");
    let sidecar = resolve_summary_path_for_flow(flow_path);
    assert_eq!(sidecar, Path::new("flows/main.ygtc.resolve.summary.json"));
}

#[test]
fn flow_resolve_summary_roundtrip_ext() {
    let doc = FlowResolveSummaryV1 {
        schema_version: FLOW_RESOLVE_SUMMARY_SCHEMA_VERSION,
        flow: "main.ygtc".into(),
        nodes: BTreeMap::from([(
            "node".to_string(),
            NodeResolveSummaryV1 {
                component_id: "greentic.http.component".parse().expect("component id"),
                source: FlowResolveSummarySourceRefV1::Ext {
                    r#ref: "ext://greentic.http#component".into(),
                },
                digest: "sha256:abc".into(),
                manifest: None,
            },
        )]),
    };

    let json = serde_json::to_string_pretty(&doc).expect("serialize");
    let decoded: FlowResolveSummaryV1 = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(decoded, doc);
    assert!(
        json.contains(r#""kind": "ext""#),
        "missing kind:ext in {json}"
    );
}
