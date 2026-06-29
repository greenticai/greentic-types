#![cfg(feature = "serde")]

use std::collections::BTreeMap;

use greentic_types::{ProviderDecl, ProviderExtensionInline, ProviderManifest, ProviderRuntimeRef};

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
fn provider_manifest_roundtrips_and_skips_optional_state() {
    let manifest = ProviderManifest {
        provider_type: "vendor.logging".into(),
        capabilities: vec!["events".into(), "metrics".into()],
        ops: vec!["start".into(), "stop".into()],
        config_schema_ref: Some("schemas/logging-config.json".into()),
        state_schema_ref: None,
    };

    let value = serde_json::to_value(&manifest).expect("serialize");
    assert!(
        value.get("state_schema_ref").is_none(),
        "optional state schema should be omitted when None"
    );

    let roundtrip = roundtrip_json(&manifest);
    assert_eq!(roundtrip, manifest);
}

#[test]
fn provider_decl_roundtrips_and_omits_empty_fields() {
    let runtime = ProviderRuntimeRef {
        component_ref: "vendor.provider.runtime".into(),
        export: "greentic_provider".into(),
        world: "greentic:provider/runtime".into(),
    };

    let decl = ProviderDecl {
        provider_type: "vendor.db".into(),
        capabilities: vec!["database".into(), "backup".into()],
        ops: vec!["connect".into(), "query".into(), "close".into()],
        config_schema_ref: "schemas/db-config.json".into(),
        state_schema_ref: Some("schemas/db-state.json".into()),
        runtime,
        docs_ref: Some("docs/providers/db.md".into()),
    };

    let value = serde_json::to_value(&decl).expect("serialize");
    assert_eq!(
        value.get("docs_ref").and_then(|v| v.as_str()),
        Some("docs/providers/db.md")
    );

    let roundtrip = roundtrip_json(&decl);
    assert_eq!(roundtrip, decl);

    let mut stripped = decl.clone();
    stripped.capabilities.clear();
    stripped.ops.clear();
    stripped.state_schema_ref = None;
    stripped.docs_ref = None;
    let stripped_value = serde_json::to_value(&stripped).expect("serialize stripped");
    assert!(stripped_value.get("capabilities").is_none());
    assert!(stripped_value.get("ops").is_none());
    assert!(stripped_value.get("state_schema_ref").is_none());
    assert!(stripped_value.get("docs_ref").is_none());
}

#[test]
fn provider_extension_inline_roundtrips() {
    let providers = vec![
        ProviderDecl {
            provider_type: "vendor.search".into(),
            capabilities: vec!["index".into()],
            ops: vec!["ingest".into(), "query".into()],
            config_schema_ref: "schemas/search-config.json".into(),
            state_schema_ref: None,
            runtime: ProviderRuntimeRef {
                component_ref: "vendor.search.runtime".into(),
                export: "greentic_provider".into(),
                world: "greentic:provider/runtime".into(),
            },
            docs_ref: Some("docs/providers/search.md".into()),
        },
        ProviderDecl {
            provider_type: "vendor.cache".into(),
            capabilities: Vec::new(),
            ops: vec!["set".into(), "get".into()],
            config_schema_ref: "schemas/cache-config.json".into(),
            state_schema_ref: Some("schemas/cache-state.json".into()),
            runtime: ProviderRuntimeRef {
                component_ref: "vendor.cache.runtime".into(),
                export: "greentic_provider".into(),
                world: "greentic:provider/runtime".into(),
            },
            docs_ref: None,
        },
    ];

    let inline = ProviderExtensionInline {
        providers,
        additional_fields: BTreeMap::new(),
    };

    let roundtrip = roundtrip_json(&inline);
    assert_eq!(roundtrip.providers.len(), 2);
    assert_eq!(roundtrip.providers[0].provider_type, "vendor.search");
    assert_eq!(roundtrip.providers[1].provider_type, "vendor.cache");
}
