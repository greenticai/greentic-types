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
        provider_id: None,
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
            provider_id: None,
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
            provider_id: Some("vendor.cache.primary".into()),
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
    assert_eq!(roundtrip.providers[0].provider_id, None);
    assert_eq!(roundtrip.providers[1].provider_type, "vendor.cache");
    assert_eq!(
        roundtrip.providers[1].provider_id.as_deref(),
        Some("vendor.cache.primary")
    );
}

fn decl(provider_type: &str, provider_id: Option<&str>) -> ProviderDecl {
    ProviderDecl {
        provider_type: provider_type.into(),
        provider_id: provider_id.map(str::to_owned),
        capabilities: Vec::new(),
        ops: Vec::new(),
        config_schema_ref: "schemas/x.json".into(),
        state_schema_ref: None,
        runtime: ProviderRuntimeRef {
            component_ref: "vendor.runtime".into(),
            export: "greentic_provider".into(),
            world: "greentic:provider/runtime".into(),
        },
        docs_ref: None,
    }
}

#[test]
fn provider_id_is_optional_and_defaults_to_none_when_absent_in_json() {
    let json = r#"{
        "provider_type": "vendor.legacy",
        "config_schema_ref": "schemas/legacy.json",
        "runtime": {
            "component_ref": "vendor.runtime",
            "export": "greentic_provider",
            "world": "greentic:provider/runtime"
        }
    }"#;
    let decoded: ProviderDecl = serde_json::from_str(json).expect("legacy provider decl");
    assert_eq!(decoded.provider_id, None);
}

#[test]
fn validate_basic_accepts_distinct_provider_ids() {
    let inline = ProviderExtensionInline {
        providers: vec![
            decl("vendor.a", Some("vendor.a.primary")),
            decl("vendor.b", Some("vendor.b.primary")),
        ],
        additional_fields: BTreeMap::new(),
    };
    inline.validate_basic().expect("distinct ids accepted");
}

#[test]
fn validate_basic_rejects_duplicate_provider_id() {
    let inline = ProviderExtensionInline {
        providers: vec![
            decl("vendor.a", Some("shared")),
            decl("vendor.b", Some("shared")),
        ],
        additional_fields: BTreeMap::new(),
    };
    let err = inline.validate_basic().expect_err("duplicate id rejected");
    assert!(
        err.to_string().contains("duplicate provider_id 'shared'"),
        "unexpected message: {err}"
    );
}

#[test]
fn validate_basic_rejects_empty_provider_id_when_set() {
    let inline = ProviderExtensionInline {
        providers: vec![decl("vendor.a", Some(""))],
        additional_fields: BTreeMap::new(),
    };
    let err = inline
        .validate_basic()
        .expect_err("empty provider_id rejected");
    assert!(
        err.to_string().contains("provider_id must not be empty"),
        "unexpected message: {err}"
    );
}

#[test]
fn validate_basic_still_rejects_duplicate_provider_type() {
    let inline = ProviderExtensionInline {
        providers: vec![decl("vendor.a", None), decl("vendor.a", None)],
        additional_fields: BTreeMap::new(),
    };
    let err = inline
        .validate_basic()
        .expect_err("duplicate type rejected");
    assert!(
        err.to_string()
            .contains("duplicate provider_type 'vendor.a'"),
        "unexpected message: {err}"
    );
}

#[test]
fn validate_basic_still_rejects_duplicate_type_even_with_distinct_ids() {
    // Same-type-different-id within ONE pack is not supported in M1; multi-instance
    // happens at the environment level via ProviderRegistry::load_instance.
    let inline = ProviderExtensionInline {
        providers: vec![
            decl("vendor.a", Some("vendor.a.one")),
            decl("vendor.a", Some("vendor.a.two")),
        ],
        additional_fields: BTreeMap::new(),
    };
    let err = inline
        .validate_basic()
        .expect_err("duplicate type rejected even with ids");
    assert!(
        err.to_string()
            .contains("duplicate provider_type 'vendor.a'"),
        "unexpected message: {err}"
    );
}

#[test]
fn validate_basic_rejects_provider_id_colliding_with_another_decls_type() {
    // The post-M1.1b runner-host resolves provider_id by first probing
    // load_instance, then scanning inline by provider_id match. If a Slack
    // decl declares provider_id="teams" while a Teams decl exists with no
    // provider_id, a lookup keyed "teams" would silently bind to the Slack
    // runtime. Reject at validation time.
    let inline = ProviderExtensionInline {
        providers: vec![decl("teams", None), decl("slack", Some("teams"))],
        additional_fields: BTreeMap::new(),
    };
    let err = inline
        .validate_basic()
        .expect_err("cross-namespace collision rejected");
    let msg = err.to_string();
    assert!(
        msg.contains("provider_id 'teams'") && msg.contains("provider_type"),
        "unexpected message: {msg}"
    );
}

#[test]
fn validate_basic_permits_provider_id_aliasing_its_own_provider_type() {
    // A decl declaring provider_id = its own provider_type is a harmless
    // explicit alias (no other decl is shadowed), and is allowed.
    let inline = ProviderExtensionInline {
        providers: vec![decl("teams", Some("teams"))],
        additional_fields: BTreeMap::new(),
    };
    inline
        .validate_basic()
        .expect("self-aliasing provider_id accepted");
}
