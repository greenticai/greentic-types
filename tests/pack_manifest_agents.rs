#![cfg(feature = "serde")]

use std::collections::BTreeMap;

use greentic_types::{
    PackId, PackKind, PackManifest, PackSignatures, decode_pack_manifest, encode_pack_manifest,
};
use semver::Version;

#[test]
fn pack_manifest_agents_blob_round_trips() {
    let mut manifest = PackManifest {
        schema_version: "pack-v1".into(),
        pack_id: PackId::new("vendor.agents.demo").unwrap(),
        name: None,
        version: Version::parse("0.1.0").unwrap(),
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
        agents: BTreeMap::new(),
    };

    let mut agents = BTreeMap::new();
    agents.insert(
        "greeter".to_string(),
        serde_json::json!({
            "agent_id": "greeter",
            "system_prompt": "hi",
            "tools": [],
            "llm": { "provider": "openai", "model": "gpt-4o-mini" }
        }),
    );
    manifest.agents = agents.clone();

    let encoded = encode_pack_manifest(&manifest).expect("encode");
    let decoded = decode_pack_manifest(&encoded).expect("decode");
    assert_eq!(decoded.agents.get("greeter"), agents.get("greeter"));
}
