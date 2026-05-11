#![cfg(feature = "serde")]

use std::collections::BTreeMap;

use greentic_types::{
    CapabilitiesExtensionV1, CapabilityOfferV1, CapabilityProviderRefV1, CapabilitySetupV1,
    EXT_CAPABILITIES_V1, ExtensionInline, ExtensionRef, PackId, PackKind, PackManifest,
    PackSignatures, ProviderExtensionInline,
};
use semver::Version;

fn base_manifest() -> PackManifest {
    PackManifest {
        schema_version: "pack-v1".into(),
        pack_id: PackId::new("vendor.ext.capabilities").unwrap(),
        name: None,
        version: Version::parse("0.1.0").unwrap(),
        kind: PackKind::Provider,
        publisher: "vendor".into(),
        components: Vec::new(),
        flows: Vec::new(),
        dependencies: Vec::new(),
        capabilities: Vec::new(),
        secret_requirements: Vec::new(),
        signatures: PackSignatures::default(),
        bootstrap: None,
        extensions: None,
    }
}

#[test]
fn capabilities_extension_set_get_roundtrip() {
    let mut manifest = base_manifest();
    let payload = CapabilitiesExtensionV1::new(vec![CapabilityOfferV1 {
        offer_id: "hooks.pre.01".into(),
        cap_id: "greentic.cap.op_hook.pre".into(),
        version: "v1".into(),
        provider: CapabilityProviderRefV1 {
            component_ref: "policy-hook".into(),
            op: "hook.evaluate".into(),
        },
        scope: None,
        priority: 10,
        requires_setup: true,
        setup: Some(CapabilitySetupV1 {
            qa_ref: "qa/hooks/policy-setup.cbor".into(),
        }),
        applies_to: None,
    }]);

    manifest
        .set_capabilities_extension_v1(payload.clone())
        .expect("set capabilities extension");
    let loaded = manifest
        .get_capabilities_extension_v1()
        .expect("read capabilities extension")
        .expect("capabilities extension present");

    assert_eq!(loaded, payload);
}

#[test]
fn capabilities_extension_requires_setup_payload() {
    let payload = CapabilitiesExtensionV1::new(vec![CapabilityOfferV1 {
        offer_id: "memory.01".into(),
        cap_id: "greentic.cap.memory.shortterm".into(),
        version: "v1".into(),
        provider: CapabilityProviderRefV1 {
            component_ref: "memory-provider".into(),
            op: "cap.invoke".into(),
        },
        scope: None,
        priority: 0,
        requires_setup: true,
        setup: None,
        applies_to: None,
    }]);

    let err = payload.validate().expect_err("validation should fail");
    let text = err.to_string();
    assert!(text.contains("requires setup"));
}

#[test]
fn capabilities_extension_rejects_provider_inline_variant() {
    let mut manifest = base_manifest();
    manifest.extensions = Some(BTreeMap::from([(
        EXT_CAPABILITIES_V1.to_string(),
        ExtensionRef {
            kind: EXT_CAPABILITIES_V1.to_string(),
            version: "1.0.0".to_string(),
            digest: None,
            location: None,
            inline: Some(ExtensionInline::Provider(ProviderExtensionInline::default())),
        },
    )]));

    let err = manifest
        .get_capabilities_extension_v1()
        .expect_err("unexpected inline type should error");
    assert!(err.to_string().contains("unexpected type"));
}
