#![cfg(feature = "serde")]

use greentic_types::pack::extensions::component_sources::{
    ArtifactLocationV1, ComponentSourceEntryV1, ComponentSourcesV1, ResolvedComponentV1,
    decode_component_sources_v1_from_cbor_bytes, encode_component_sources_v1_to_cbor_bytes,
};
use greentic_types::{
    ComponentId, ComponentSourceRef, ComponentSourceRefError, PackId, PackKind, PackManifest,
    PackSignatures, decode_pack_manifest, encode_pack_manifest,
};
use semver::Version;
use serde_json::json;

#[test]
fn component_source_ref_parses_and_formats() {
    let value = "oci://ghcr.io/acme/search@sha256:abc";
    let parsed: ComponentSourceRef = value.parse().expect("parse oci ref");
    assert_eq!(parsed.to_string(), value);
    assert_eq!(parsed.scheme(), "oci");
    assert_eq!(parsed.reference(), "ghcr.io/acme/search@sha256:abc");

    assert_eq!(
        "store://vendor/search@1.2.3"
            .parse::<ComponentSourceRef>()
            .expect("parse store ref")
            .scheme(),
        "store"
    );
    let file_value = "file:///var/components/search.wasm";
    let parsed_file: ComponentSourceRef = file_value.parse().expect("parse file ref");
    assert_eq!(parsed_file.to_string(), file_value);
    assert_eq!(parsed_file.scheme(), "file");
    assert_eq!(parsed_file.reference(), "/var/components/search.wasm");

    assert_eq!(
        "oci://".parse::<ComponentSourceRef>().unwrap_err(),
        ComponentSourceRefError::MissingLocator
    );
    assert_eq!(
        "http://example.com"
            .parse::<ComponentSourceRef>()
            .unwrap_err(),
        ComponentSourceRefError::InvalidScheme
    );
    assert_eq!(
        "oci://bad ref".parse::<ComponentSourceRef>().unwrap_err(),
        ComponentSourceRefError::ContainsWhitespace
    );
}

#[test]
fn component_source_ref_detects_tag_and_digest() {
    let tagged: ComponentSourceRef = "oci://ghcr.io/acme/search:1.2.3".parse().unwrap();
    assert!(tagged.is_tag());
    assert!(!tagged.is_digest());
    assert_eq!(tagged.normalized(), "oci://ghcr.io/acme/search:1.2.3");

    let digested: ComponentSourceRef = "oci://ghcr.io/acme/search@sha256:abc".parse().unwrap();
    assert!(digested.is_digest());
    assert!(!digested.is_tag());
    assert_eq!(
        digested.normalized(),
        "oci://ghcr.io/acme/search@sha256:abc"
    );

    let with_port: ComponentSourceRef = "oci://registry.local:5000/acme/search:latest"
        .parse()
        .unwrap();
    assert!(with_port.is_tag());
    assert!(!with_port.is_digest());

    let port_without_tag: ComponentSourceRef =
        "oci://registry.local:5000/acme/search".parse().unwrap();
    assert!(!port_without_tag.is_tag());
    assert!(!port_without_tag.is_digest());

    let tagged_and_digested: ComponentSourceRef =
        "oci://registry.local:5000/acme/search:latest@sha256:abc"
            .parse()
            .unwrap();
    assert!(tagged_and_digested.is_digest());
    assert!(!tagged_and_digested.is_tag());
    assert_eq!(
        tagged_and_digested.normalized(),
        "oci://registry.local:5000/acme/search@sha256:abc"
    );

    let repo_ref: ComponentSourceRef = "repo://acme/search@1.2.3".parse().unwrap();
    assert!(!repo_ref.is_tag());
    assert!(!repo_ref.is_digest());
    assert_eq!(repo_ref.normalized(), "repo://acme/search@1.2.3");
}

#[test]
fn component_sources_payload_roundtrips_and_matches_shape() {
    let sources = ComponentSourcesV1::new(vec![
        ComponentSourceEntryV1 {
            name: "search".into(),
            component_id: Some(ComponentId::new("vendor.search").unwrap()),
            source: "oci://ghcr.io/acme/search@sha256:abc".parse().unwrap(),
            resolved: ResolvedComponentV1 {
                digest: "sha256:abc".into(),
                signature: Some("sig:ed25519:deadbeef".into()),
                signed_by: Some("acme".into()),
            },
            artifact: ArtifactLocationV1::Inline {
                wasm_path: "components/search.wasm".into(),
                manifest_path: Some("components/search.manifest.cbor".into()),
            },
            licensing_hint: None,
            metering_hint: None,
        },
        ComponentSourceEntryV1 {
            name: "cache".into(),
            component_id: None,
            source: "store://vendor/cache@1.2.0".parse().unwrap(),
            resolved: ResolvedComponentV1 {
                digest: "sha256:def".into(),
                signature: None,
                signed_by: None,
            },
            artifact: ArtifactLocationV1::Remote,
            licensing_hint: Some("license:pro".into()),
            metering_hint: Some("metered:requests".into()),
        },
    ]);

    let value = sources.to_extension_value().expect("to extension value");
    let expected = json!({
        "schema_version": 1,
        "components": [
            {
                "name": "search",
                "component_id": "vendor.search",
                "source": "oci://ghcr.io/acme/search@sha256:abc",
                "resolved": {
                    "digest": "sha256:abc",
                    "signature": "sig:ed25519:deadbeef",
                    "signed_by": "acme"
                },
                "artifact": {
                    "inline": {
                        "wasm_path": "components/search.wasm",
                        "manifest_path": "components/search.manifest.cbor"
                    }
                }
            },
            {
                "name": "cache",
                "source": "store://vendor/cache@1.2.0",
                "resolved": {
                    "digest": "sha256:def"
                },
                "artifact": "remote",
                "licensing_hint": "license:pro",
                "metering_hint": "metered:requests"
            }
        ]
    });
    assert_eq!(value, expected);

    let parsed = ComponentSourcesV1::from_extension_value(&value).expect("from extension value");
    assert_eq!(parsed, sources);

    let cbor = encode_component_sources_v1_to_cbor_bytes(&sources).expect("encode cbor");
    let decoded =
        decode_component_sources_v1_from_cbor_bytes(&cbor).expect("decode component sources");
    assert_eq!(decoded, sources);
}

#[test]
fn pack_manifest_component_sources_helpers_work() {
    let mut manifest = PackManifest {
        schema_version: "pack-v1".into(),
        pack_id: PackId::new("vendor.pack").unwrap(),
        name: None,
        version: Version::parse("1.2.0").unwrap(),
        kind: PackKind::Application,
        publisher: "vendor".into(),
        components: Vec::new(),
        flows: Vec::new(),
        dependencies: Vec::new(),
        capabilities: Vec::new(),
        secret_requirements: Vec::new(),
        signatures: PackSignatures::default(),
        bootstrap: None,
        extensions: None,
    };

    let sources = ComponentSourcesV1::new(vec![ComponentSourceEntryV1 {
        name: "search".into(),
        component_id: Some(ComponentId::new("vendor.search").unwrap()),
        source: "oci://ghcr.io/acme/search@sha256:abc".parse().unwrap(),
        resolved: ResolvedComponentV1 {
            digest: "sha256:abc".into(),
            signature: None,
            signed_by: None,
        },
        artifact: ArtifactLocationV1::Remote,
        licensing_hint: None,
        metering_hint: None,
    }]);

    manifest
        .set_component_sources_v1(sources.clone())
        .expect("set component sources");
    let roundtrip = manifest
        .get_component_sources_v1()
        .expect("get component sources");
    assert_eq!(roundtrip, Some(sources));
}

#[test]
fn pack_manifest_without_component_sources_still_decodes() {
    let manifest = PackManifest {
        schema_version: "pack-v1".into(),
        pack_id: PackId::new("vendor.pack").unwrap(),
        name: None,
        version: Version::parse("1.0.0").unwrap(),
        kind: PackKind::Application,
        publisher: "vendor".into(),
        components: Vec::new(),
        flows: Vec::new(),
        dependencies: Vec::new(),
        capabilities: Vec::new(),
        secret_requirements: Vec::new(),
        signatures: PackSignatures::default(),
        bootstrap: None,
        extensions: None,
    };

    let cbor = encode_pack_manifest(&manifest).expect("encode pack manifest");
    let decoded = decode_pack_manifest(&cbor).expect("decode pack manifest");
    assert!(decoded.extensions.is_none());
}
