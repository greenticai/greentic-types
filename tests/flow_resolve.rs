use std::collections::BTreeMap;
use std::path::Path;

use greentic_types::{
    ComponentSourceRefV1, FLOW_RESOLVE_SCHEMA_VERSION, FlowResolveV1, NodeResolveV1, ResolveModeV1,
    sidecar_path_for_flow, validate_flow_resolve,
};

fn roundtrip(source: ComponentSourceRefV1) {
    let doc = FlowResolveV1 {
        schema_version: FLOW_RESOLVE_SCHEMA_VERSION,
        flow: "main.ygtc".into(),
        nodes: BTreeMap::from([(
            "node".to_string(),
            NodeResolveV1 {
                source,
                mode: Some(ResolveModeV1::Pinned),
            },
        )]),
    };

    let json = serde_json::to_string_pretty(&doc).expect("serialize");
    let decoded: FlowResolveV1 = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(decoded, doc);
}

#[test]
fn flow_resolve_roundtrip_local() {
    roundtrip(ComponentSourceRefV1::Local {
        path: "components/demo.wasm".into(),
        digest: Some("sha256:deadbeef".into()),
    });
}

#[test]
fn flow_resolve_roundtrip_oci() {
    roundtrip(ComponentSourceRefV1::Oci {
        r#ref: "ghcr.io/greentic/demo/component:1.2.3".into(),
        digest: Some("sha256:deadbeef".into()),
    });
}

#[test]
fn flow_resolve_roundtrip_repo() {
    roundtrip(ComponentSourceRefV1::Repo {
        r#ref: "repo.example.com/greentic/demo/component@1.2.3".into(),
        digest: Some("sha256:deadbeef".into()),
    });
}

#[test]
fn flow_resolve_roundtrip_store() {
    roundtrip(ComponentSourceRefV1::Store {
        r#ref: "store://greentic/demo/component".into(),
        digest: Some("sha256:deadbeef".into()),
        license_hint: Some("team-license".into()),
        meter: Some(true),
    });
}

#[test]
fn flow_resolve_rejects_absolute_local_paths() {
    let doc = FlowResolveV1 {
        schema_version: FLOW_RESOLVE_SCHEMA_VERSION,
        flow: "main.ygtc".into(),
        nodes: BTreeMap::from([(
            "node".to_string(),
            NodeResolveV1 {
                source: ComponentSourceRefV1::Local {
                    path: "/abs/component.wasm".into(),
                    digest: None,
                },
                mode: None,
            },
        )]),
    };

    let err = validate_flow_resolve(&doc).expect_err("should reject absolute path");
    assert_eq!(err.code, greentic_types::ErrorCode::InvalidInput);
}

#[test]
fn sidecar_path_helper_uses_resolve_suffix() {
    let flow_path = Path::new("flows/main.ygtc");
    let sidecar = sidecar_path_for_flow(flow_path);
    assert_eq!(sidecar, Path::new("flows/main.ygtc.resolve.json"));
}

#[test]
fn flow_resolve_roundtrip_ext_with_digest() {
    roundtrip(ComponentSourceRefV1::Ext {
        r#ref: "ext://greentic.http#component".into(),
        digest: Some("sha256:abc".into()),
    });
}

#[test]
fn flow_resolve_ext_serializes_kind_and_ref() {
    let source = ComponentSourceRefV1::Ext {
        r#ref: "ext://greentic.http#component".into(),
        digest: Some("sha256:abc".into()),
    };
    let doc = FlowResolveV1 {
        schema_version: FLOW_RESOLVE_SCHEMA_VERSION,
        flow: "main.ygtc".into(),
        nodes: BTreeMap::from([(
            "node".to_string(),
            NodeResolveV1 {
                source,
                mode: Some(ResolveModeV1::Pinned),
            },
        )]),
    };
    let json = serde_json::to_string_pretty(&doc).expect("serialize");
    assert!(
        json.contains(r#""kind": "ext""#),
        "missing kind:ext in {json}"
    );
    assert!(
        json.contains(r#""ref": "ext://greentic.http#component""#),
        "missing ref in {json}"
    );
    assert!(
        json.contains(r#""digest": "sha256:abc""#),
        "missing digest in {json}"
    );
}

#[test]
fn flow_resolve_ext_without_digest_omits_digest_field() {
    let source = ComponentSourceRefV1::Ext {
        r#ref: "ext://greentic.http#component".into(),
        digest: None,
    };
    let doc = FlowResolveV1 {
        schema_version: FLOW_RESOLVE_SCHEMA_VERSION,
        flow: "main.ygtc".into(),
        nodes: BTreeMap::from([("node".to_string(), NodeResolveV1 { source, mode: None })]),
    };
    let json = serde_json::to_string_pretty(&doc).expect("serialize");
    assert!(
        !json.contains("digest"),
        "digest field should be omitted when None: {json}"
    );
    let decoded: FlowResolveV1 = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(decoded, doc);
}
