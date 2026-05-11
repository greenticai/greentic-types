#![cfg(feature = "serde")]

use std::collections::BTreeMap;

use greentic_types::{
    ExtensionInline, ExtensionRef, PROVIDER_EXTENSION_ID, PackId, PackKind, PackManifest,
    PackSignatures, ProviderDecl, ProviderExtensionInline, ProviderRuntimeRef,
    decode_pack_manifest, encode_pack_manifest,
};
use semver::Version;
use serde_json::json;

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
fn deserializes_without_extensions() {
    let manifest: PackManifest =
        serde_json::from_str(include_str!("fixtures/manifest_no_ext.json")).expect("fixture");
    assert!(
        manifest.extensions.is_none(),
        "extensions should default to None"
    );

    let value = serde_json::to_value(&manifest).expect("serialize");
    assert!(
        value.get("extensions").is_none(),
        "extensions should be omitted when empty"
    );
}

#[test]
fn parses_manifest_with_provider_extension() {
    let manifest: PackManifest =
        serde_json::from_str(include_str!("fixtures/manifest_with_provider_ext.json"))
            .expect("fixture");

    let extensions = manifest.extensions.as_ref().expect("extensions present");
    assert_eq!(extensions.len(), 1);

    let provider = extensions
        .get(PROVIDER_EXTENSION_ID)
        .expect("provider extension");
    assert_eq!(provider.kind, PROVIDER_EXTENSION_ID);
    assert_eq!(provider.version, "1.0.0");
    assert_eq!(provider.digest.as_deref(), Some("sha256:abc123"));
    assert_eq!(
        provider.location.as_deref(),
        Some("https://example.com/provider-ext.json")
    );
    let inline = provider
        .inline
        .as_ref()
        .and_then(ExtensionInline::as_provider_inline)
        .expect("provider inline payload");
    assert_eq!(inline.providers.len(), 1);
    assert_eq!(inline.providers[0].provider_type, "vendor.search");
    assert_eq!(
        inline
            .additional_fields
            .get("notes")
            .and_then(|value| value.as_str()),
        Some("supports embedded providers")
    );
}

#[test]
fn parses_manifest_with_unknown_extension() {
    let manifest: PackManifest =
        serde_json::from_str(include_str!("fixtures/manifest_with_unknown_ext.json"))
            .expect("fixture");

    let extensions = manifest.extensions.as_ref().expect("extensions present");
    assert_eq!(extensions.len(), 1);

    let unknown = extensions
        .get("acme.ext.logging")
        .expect("unknown extension");
    assert_eq!(unknown.kind, "acme.ext.logging");
    assert_eq!(unknown.version, "0.3.0");
    match unknown.inline.as_ref() {
        Some(ExtensionInline::Other(value)) => {
            assert_eq!(
                value.get("level").and_then(|level| level.as_str()),
                Some("info")
            );
        }
        _ => panic!("unknown extension inline payload should be preserved as raw value"),
    }
}

#[test]
fn extension_refs_roundtrip_json_yaml_and_cbor() {
    let mut extensions = BTreeMap::new();
    extensions.insert(
        PROVIDER_EXTENSION_ID.to_string(),
        ExtensionRef {
            kind: PROVIDER_EXTENSION_ID.into(),
            version: "1.2.3".into(),
            digest: Some("sha256:def456".into()),
            location: Some("file://extensions/provider.json".into()),
            inline: Some(ExtensionInline::Provider(ProviderExtensionInline {
                providers: vec![ProviderDecl {
                    provider_type: "vendor.search".into(),
                    capabilities: vec!["query".into()],
                    ops: vec!["index".into()],
                    config_schema_ref: "schemas/search.json".into(),
                    state_schema_ref: None,
                    runtime: ProviderRuntimeRef {
                        component_ref: "vendor.search.runtime".into(),
                        export: "greentic_provider".into(),
                        world: "greentic:provider/runtime".into(),
                    },
                    docs_ref: None,
                }],
                additional_fields: BTreeMap::new(),
            })),
        },
    );

    let manifest = PackManifest {
        schema_version: "pack-v1".into(),
        pack_id: PackId::new("vendor.ext.demo").unwrap(),
        name: None,
        version: Version::parse("0.2.0").unwrap(),
        kind: PackKind::Application,
        publisher: "vendor".into(),
        components: Vec::new(),
        flows: Vec::new(),
        dependencies: Vec::new(),
        capabilities: Vec::new(),
        secret_requirements: Vec::new(),
        signatures: PackSignatures::default(),
        bootstrap: None,
        extensions: Some(extensions),
    };

    let json_value = serde_json::to_value(&manifest).expect("serialize");
    assert!(
        json_value.get("extensions").is_some(),
        "non-empty extensions should serialize"
    );

    let json_roundtrip = roundtrip_json(&manifest);
    assert_eq!(json_roundtrip.extensions, manifest.extensions);

    let yaml = serde_yaml_bw::to_string(&manifest).expect("serialize yaml");
    let yaml_roundtrip: PackManifest = serde_yaml_bw::from_str(&yaml).expect("yaml roundtrip");
    assert_eq!(yaml_roundtrip.extensions, manifest.extensions);

    let cbor = encode_pack_manifest(&manifest).expect("encode cbor");
    let decoded = decode_pack_manifest(&cbor).expect("decode cbor");
    assert_eq!(decoded.extensions, manifest.extensions);
}

#[test]
fn provider_extension_helpers_roundtrip_and_validate() {
    let runtime = ProviderRuntimeRef {
        component_ref: "vendor.cache.runtime".into(),
        export: "greentic_provider".into(),
        world: "greentic:provider/runtime".into(),
    };
    let provider = ProviderDecl {
        provider_type: "vendor.cache".into(),
        capabilities: vec!["cache".into()],
        ops: vec!["put".into(), "get".into()],
        config_schema_ref: "schemas/cache.json".into(),
        state_schema_ref: None,
        runtime: runtime.clone(),
        docs_ref: None,
    };

    let mut extensions = BTreeMap::new();
    extensions.insert(
        PROVIDER_EXTENSION_ID.to_string(),
        ExtensionRef {
            kind: PROVIDER_EXTENSION_ID.into(),
            version: "1.0.0".into(),
            digest: None,
            location: None,
            inline: Some(ExtensionInline::Provider(ProviderExtensionInline {
                providers: vec![provider.clone()],
                additional_fields: BTreeMap::from([("notes".into(), json!("kept"))]),
            })),
        },
    );
    extensions.insert(
        "acme.ext.logging".to_string(),
        ExtensionRef {
            kind: "acme.ext.logging".into(),
            version: "0.3.0".into(),
            digest: None,
            location: None,
            inline: Some(ExtensionInline::Other(json!({ "level": "debug" }))),
        },
    );

    let manifest = PackManifest {
        schema_version: "pack-v1".into(),
        pack_id: PackId::new("vendor.ext.demo").unwrap(),
        name: None,
        version: Version::parse("0.3.0").unwrap(),
        kind: PackKind::Provider,
        publisher: "vendor".into(),
        components: Vec::new(),
        flows: Vec::new(),
        dependencies: Vec::new(),
        capabilities: Vec::new(),
        secret_requirements: Vec::new(),
        signatures: PackSignatures::default(),
        bootstrap: None,
        extensions: Some(extensions),
    };

    let json_roundtrip: PackManifest =
        serde_json::from_str(&serde_json::to_string_pretty(&manifest).expect("serialize json"))
            .expect("json roundtrip");
    assert_eq!(json_roundtrip.extensions, manifest.extensions);
    let yaml_roundtrip: PackManifest =
        serde_yaml_bw::from_str(&serde_yaml_bw::to_string(&manifest).expect("serialize yaml"))
            .expect("yaml roundtrip");
    assert_eq!(yaml_roundtrip.extensions, manifest.extensions);

    let mut empty_manifest = manifest.clone();
    empty_manifest.extensions = None;
    {
        let inline = empty_manifest.ensure_provider_extension_inline();
        assert!(inline.providers.is_empty());
        inline.providers.push(provider.clone());
    }
    assert_eq!(
        empty_manifest
            .provider_extension_inline()
            .map(|ext| ext.providers.len()),
        Some(1)
    );

    let inline = empty_manifest
        .provider_extension_inline_mut()
        .expect("provider inline present");
    inline.providers.push(provider);
    assert!(inline.validate_basic().is_err());
}

#[test]
fn empty_extensions_are_skipped_on_serialization() {
    let manifest = PackManifest {
        schema_version: "pack-v1".into(),
        pack_id: PackId::new("vendor.ext.empty").unwrap(),
        name: None,
        version: Version::parse("0.1.0").unwrap(),
        kind: PackKind::Library,
        publisher: "vendor".into(),
        components: Vec::new(),
        flows: Vec::new(),
        dependencies: Vec::new(),
        capabilities: Vec::new(),
        secret_requirements: Vec::new(),
        signatures: PackSignatures::default(),
        bootstrap: None,
        extensions: Some(BTreeMap::new()),
    };

    let value = serde_json::to_value(&manifest).expect("serialize");
    assert!(
        value.get("extensions").is_none(),
        "empty extensions map should be skipped"
    );
}
