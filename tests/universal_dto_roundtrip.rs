#![cfg(feature = "serde")]

use greentic_types::{
    AuthUserRefV1, ChannelMessageEnvelope, EncodeInV1, Header, HttpInV1, HttpOutV1,
    ProviderPayloadV1, RenderPlanInV1, RenderPlanOutV1, SendPayloadInV1, SendPayloadResultV1,
    SubscriptionDeleteInV1, SubscriptionDeleteOutV1, SubscriptionEnsureInV1,
    SubscriptionEnsureOutV1, SubscriptionRenewInV1, SubscriptionRenewOutV1, TenantCtx,
};
use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::{Value, json};
use std::collections::BTreeMap;
use std::fmt::Debug;

fn assert_roundtrip<T>(value: &T)
where
    T: Serialize + DeserializeOwned + PartialEq + Debug,
{
    let json = serde_json::to_string_pretty(value).expect("serialize");
    let roundtrip: T = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(&roundtrip, value, "{json}");
}

fn sample_envelope() -> ChannelMessageEnvelope {
    let ctx = TenantCtx::new("prod".parse().unwrap(), "tenant-1".parse().unwrap());
    ChannelMessageEnvelope {
        id: "msg-1".into(),
        tenant: ctx,
        channel: "universal".into(),
        session_id: "session-1".into(),
        reply_scope: None,
        from: None,
        to: Vec::new(),
        correlation_id: None,
        text: Some("hello".into()),
        attachments: Vec::new(),
        metadata: BTreeMap::new(),
        extensions: Default::default(),
    }
}

fn sample_metadata() -> BTreeMap<String, Value> {
    let mut metadata = BTreeMap::new();
    metadata.insert("origin".into(), json!("tests"));
    metadata
}

#[test]
fn http_dtos_roundtrip() {
    let http_in = HttpInV1 {
        method: "POST".into(),
        path: "/ingress".into(),
        query: Some("q=1".into()),
        headers: vec![Header {
            name: "content-type".into(),
            value: "application/json".into(),
        }],
        body_b64: "ZGF0YQ==".into(),
        route_hint: Some("route".into()),
        binding_id: Some("binding".into()),
        config: Some(json!({"mode": "test"})),
    };
    let http_out = HttpOutV1 {
        status: 202,
        headers: vec![Header {
            name: "x-trace".into(),
            value: "trace-1".into(),
        }],
        body_b64: "b2s=".into(),
        events: vec![sample_envelope()],
    };

    assert_roundtrip(&http_in);
    assert_roundtrip(&http_out);
}

#[test]
fn render_plan_dtos_roundtrip() {
    let plan_in = RenderPlanInV1 {
        message: sample_envelope(),
        metadata: sample_metadata(),
    };
    let plan_out = RenderPlanOutV1 {
        plan_json: "{\"plan\":1}".into(),
    };

    assert_roundtrip(&plan_in);
    assert_roundtrip(&plan_out);
}

#[test]
fn send_payload_dtos_roundtrip() {
    let payload = ProviderPayloadV1 {
        content_type: "application/json".into(),
        body_b64: "e30=".into(),
        metadata: sample_metadata(),
    };
    let encode = EncodeInV1 {
        message: sample_envelope(),
        plan: RenderPlanInV1 {
            message: sample_envelope(),
            metadata: sample_metadata(),
        },
    };
    let send = SendPayloadInV1 {
        provider_type: "email".into(),
        tenant_id: Some("tenant-1".into()),
        auth_user: Some(AuthUserRefV1 {
            user_id: "user-1".into(),
            token_key: "token".into(),
            tenant_id: Some("tenant-1".into()),
            email: Some("user@example.test".into()),
            display_name: Some("User".into()),
        }),
        payload,
    };
    let send_result = SendPayloadResultV1 {
        ok: true,
        message: Some("accepted".into()),
        retryable: false,
    };

    assert_roundtrip(&encode);
    assert_roundtrip(&send);
    assert_roundtrip(&send_result);
}

#[test]
fn subscription_dtos_roundtrip() {
    let user = AuthUserRefV1 {
        user_id: "user-1".into(),
        token_key: "token".into(),
        tenant_id: Some("tenant-1".into()),
        email: None,
        display_name: None,
    };
    let ensure_in = SubscriptionEnsureInV1 {
        v: 1,
        provider: "teams".into(),
        tenant_hint: Some("tenant-1".into()),
        team_hint: Some("team-1".into()),
        binding_id: Some("binding".into()),
        resource: "/chats".into(),
        change_types: vec!["created".into()],
        notification_url: "https://example.test/hook".into(),
        expiration_minutes: Some(30),
        expiration_target_unix_ms: Some(1_700_000_000_000),
        client_state: Some("state".into()),
        metadata: Some(json!({"key":"value"})),
        user: user.clone(),
    };
    let ensure_out = SubscriptionEnsureOutV1 {
        v: 1,
        subscription_id: "sub-1".into(),
        expiration_unix_ms: 1_700_000_100_000,
        resource: "/chats".into(),
        change_types: vec!["created".into()],
        client_state: Some("state".into()),
        metadata: Some(json!({"key":"value"})),
        binding_id: Some("binding".into()),
        user: user.clone(),
    };
    let renew_in = SubscriptionRenewInV1 {
        v: 1,
        provider: "teams".into(),
        subscription_id: "sub-1".into(),
        expiration_minutes: Some(30),
        expiration_target_unix_ms: None,
        metadata: None,
        user: user.clone(),
    };
    let renew_out = SubscriptionRenewOutV1 {
        v: 1,
        subscription_id: "sub-1".into(),
        expiration_unix_ms: 1_700_000_200_000,
        metadata: None,
        user: user.clone(),
    };
    let delete_in = SubscriptionDeleteInV1 {
        v: 1,
        provider: "teams".into(),
        subscription_id: "sub-1".into(),
        user: user.clone(),
    };
    let delete_out = SubscriptionDeleteOutV1 {
        v: 1,
        subscription_id: "sub-1".into(),
        user,
    };

    assert_roundtrip(&ensure_in);
    assert_roundtrip(&ensure_out);
    assert_roundtrip(&renew_in);
    assert_roundtrip(&renew_out);
    assert_roundtrip(&delete_in);
    assert_roundtrip(&delete_out);
}
