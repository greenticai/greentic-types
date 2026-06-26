#![cfg(feature = "serde")]

use greentic_types::{AuthCaps, AuthKind, Capabilities, OAuthSpec, TokenAuthStyle};

#[test]
fn auth_kind_wire_strings_are_lowercase() {
    assert_eq!(serde_json::to_value(AuthKind::None).unwrap(), "none");
    assert_eq!(serde_json::to_value(AuthKind::ApiKey).unwrap(), "apikey");
    assert_eq!(serde_json::to_value(AuthKind::OAuth).unwrap(), "oauth");
}

#[test]
fn auth_kind_defaults_to_none() {
    assert_eq!(AuthKind::default(), AuthKind::None);
}

#[test]
fn oauth_spec_round_trips_in_camel_case() {
    let mut extra = std::collections::BTreeMap::new();
    extra.insert("access_type".to_string(), "offline".to_string());
    let mut spec = OAuthSpec::new(
        "https://accounts.google.com/o/oauth2/v2/auth",
        "https://oauth2.googleapis.com/token",
    );
    spec.scopes = vec!["openid".to_string(), "email".to_string()];
    spec.pkce = true;
    spec.extra_auth_params = extra;
    spec.token_auth_style = TokenAuthStyle::Body;

    let value = serde_json::to_value(&spec).expect("serialize");
    assert_eq!(
        value["authorizeUrl"],
        "https://accounts.google.com/o/oauth2/v2/auth"
    );
    assert_eq!(value["tokenUrl"], "https://oauth2.googleapis.com/token");
    assert_eq!(value["pkce"], true);
    assert_eq!(value["extraAuthParams"]["access_type"], "offline");
    assert_eq!(value["tokenAuthStyle"], "body");

    let back: OAuthSpec = serde_json::from_value(value).expect("deserialize");
    assert_eq!(back, spec);
}

#[test]
fn auth_caps_oauth_round_trips() {
    let json = r#"{
        "kind": "oauth",
        "oauth": {
            "authorizeUrl": "https://example.com/auth",
            "tokenUrl": "https://example.com/token",
            "scopes": ["chat:write"],
            "pkce": true
        }
    }"#;
    let caps: AuthCaps = serde_json::from_str(json).expect("deserialize");
    assert_eq!(caps.kind, AuthKind::OAuth);
    let oauth = caps.oauth.as_ref().expect("oauth present");
    assert_eq!(oauth.scopes, vec!["chat:write".to_string()]);
    // token_auth_style defaults to basic when absent.
    assert_eq!(oauth.token_auth_style, TokenAuthStyle::Basic);
    // extra_auth_params defaults to empty when absent.
    assert!(oauth.extra_auth_params.is_empty());
}

#[test]
fn auth_caps_apikey_omits_oauth() {
    let mut caps = AuthCaps::new();
    caps.kind = AuthKind::ApiKey;
    let value = serde_json::to_value(&caps).expect("serialize");
    assert_eq!(value["kind"], "apikey");
    assert!(value.get("oauth").is_none(), "None oauth must be skipped");
}

#[test]
fn capabilities_without_auth_field_deserializes_to_none() {
    // A pre-existing manifest that predates the auth field must still load.
    let caps: Capabilities = serde_json::from_str(r#"{"secrets":{"required":[]}}"#)
        .expect("legacy capabilities deserialize");
    assert!(caps.auth.is_none());

    // Round-trip a default Capabilities: auth must be omitted when None.
    let value = serde_json::to_value(Capabilities::new()).expect("serialize");
    assert!(value.get("auth").is_none(), "None auth must be skipped");
}

#[test]
fn capabilities_is_empty_tracks_auth() {
    let mut caps = Capabilities::new();
    assert!(caps.is_empty());
    let mut auth = AuthCaps::new();
    auth.kind = AuthKind::OAuth;
    caps.auth = Some(auth);
    assert!(!caps.is_empty(), "auth presence must make capabilities non-empty");
}
