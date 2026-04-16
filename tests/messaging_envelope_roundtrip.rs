#![cfg(feature = "serde")]

use greentic_types::{
    Actor, Attachment, ChannelMessageEnvelope, Destination, MessageMetadata, TenantCtx,
};
use serde::Serialize;
use serde::de::DeserializeOwned;
use std::fmt::Debug;

fn assert_roundtrip<T>(value: &T)
where
    T: Serialize + DeserializeOwned + PartialEq + Debug,
{
    let json = serde_json::to_string_pretty(value).expect("serialize");
    let roundtrip: T = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(&roundtrip, value, "{json}");
}

#[test]
fn text_only_message_roundtrip() {
    let ctx = TenantCtx::new("prod".parse().unwrap(), "tenant-1".parse().unwrap());
    let envelope = ChannelMessageEnvelope {
        id: "msg-1".into(),
        tenant: ctx,
        channel: "generic-channel".into(),
        session_id: "thread-1".into(),
        reply_scope: None,
        from: Some(Actor {
            id: "user-1".into(),
            kind: Some("user".into()),
        }),
        to: vec![Destination {
            id: "room-1".into(),
            kind: Some("room".into()),
        }],
        correlation_id: None,
        text: Some("hello world".into()),
        attachments: Vec::new(),
        metadata: MessageMetadata::new(),
        extensions: Default::default(),
    };

    assert_roundtrip(&envelope);
}

#[test]
fn message_with_attachments_and_metadata_roundtrip() {
    let ctx = TenantCtx::new("prod".parse().unwrap(), "tenant-9".parse().unwrap())
        .with_team(Some("team-42".parse().unwrap()))
        .with_user(Some("user-22".parse().unwrap()));
    let mut metadata = MessageMetadata::new();
    metadata.insert("correlation_id".into(), "corr-9".into());
    metadata.insert("adapter".into(), "test-adapter".into());
    let attachments = vec![Attachment {
        mime_type: "image/png".into(),
        url: "https://example.test/image.png".into(),
        name: Some("diagram.png".into()),
        size_bytes: Some(1_024),
    }];
    let envelope = ChannelMessageEnvelope {
        id: "msg-attachment".into(),
        tenant: ctx,
        channel: "channel-attachments".into(),
        session_id: "session-44".into(),
        reply_scope: None,
        from: Some(Actor {
            id: "user-22".into(),
            kind: Some("user".into()),
        }),
        to: vec![Destination {
            id: "session-room".into(),
            kind: Some("room".into()),
        }],
        correlation_id: None,
        text: Some("see attachment".into()),
        attachments,
        metadata,
        extensions: Default::default(),
    };

    assert_roundtrip(&envelope);
}

#[test]
fn extensions_round_trip_through_serde() {
    use greentic_types::messaging::extensions::ext_keys;
    use serde_json::json;
    use std::collections::BTreeMap;

    let mut extensions = BTreeMap::new();
    extensions.insert(
        ext_keys::ADAPTIVE_CARD.to_string(),
        json!({"type": "AdaptiveCard", "body": []}),
    );
    extensions.insert(
        ext_keys::CHANNEL_DATA.to_string(),
        json!({"webchat": {"feature": "x"}}),
    );

    let env_id = greentic_types::EnvId::try_from("default").unwrap();
    let tenant_id = greentic_types::TenantId::try_from("default").unwrap();
    let envelope = greentic_types::ChannelMessageEnvelope {
        id: "test-1".into(),
        tenant: greentic_types::TenantCtx::new(env_id, tenant_id),
        channel: "webchat".into(),
        session_id: "sess-1".into(),
        reply_scope: None,
        from: None,
        to: Vec::new(),
        correlation_id: None,
        text: None,
        attachments: Vec::new(),
        metadata: Default::default(),
        extensions,
    };

    let json_bytes = serde_json::to_vec(&envelope).unwrap();
    let round_tripped: greentic_types::ChannelMessageEnvelope =
        serde_json::from_slice(&json_bytes).unwrap();

    assert_eq!(
        round_tripped.extensions.get(ext_keys::ADAPTIVE_CARD),
        Some(&json!({"type": "AdaptiveCard", "body": []}))
    );
    assert_eq!(
        round_tripped.extensions.get(ext_keys::CHANNEL_DATA),
        Some(&json!({"webchat": {"feature": "x"}}))
    );
}

#[test]
fn empty_extensions_omitted_from_serialized_output() {
    let env_id = greentic_types::EnvId::try_from("default").unwrap();
    let tenant_id = greentic_types::TenantId::try_from("default").unwrap();
    let envelope = greentic_types::ChannelMessageEnvelope {
        id: "test-2".into(),
        tenant: greentic_types::TenantCtx::new(env_id, tenant_id),
        channel: "webchat".into(),
        session_id: "sess-2".into(),
        reply_scope: None,
        from: None,
        to: Vec::new(),
        correlation_id: None,
        text: None,
        attachments: Vec::new(),
        metadata: Default::default(),
        extensions: Default::default(),
    };
    let json = serde_json::to_string(&envelope).unwrap();
    assert!(
        !json.contains("\"extensions\""),
        "empty extensions should be omitted (skip_serializing_if), got: {json}"
    );
}

#[test]
fn missing_extensions_deserializes_to_empty_map() {
    // Forward-compat: JSON produced by older clients (no `extensions` field)
    // must deserialize cleanly to an empty extensions map.
    // TenantCtx fields: env (EnvId), tenant (TenantId), tenant_id (TenantId), attempt (u32)
    let legacy_json = r#"{
        "id": "legacy-1",
        "tenant": {"env": "default", "tenant": "default", "tenant_id": "default", "attempt": 0},
        "channel": "webchat",
        "session_id": "sess-1"
    }"#;
    let envelope: greentic_types::ChannelMessageEnvelope =
        serde_json::from_str(legacy_json).unwrap();
    assert!(envelope.extensions.is_empty());
}
