use greentic_types::secrets::{SecretKey, SecretKeyError};

// ── existing tests (must stay green) ────────────────────────────────────────

#[test]
fn parses_valid_keys() {
    assert!(SecretKey::parse("PRIMARY_TOKEN").is_ok());
    assert!(SecretKey::parse("nested/path-1._").is_ok());
    assert!(SecretKey::parse("a.b_c-d/e").is_ok());
}

#[test]
fn rejects_empty_keys() {
    assert!(matches!(SecretKey::parse(""), Err(SecretKeyError::Empty)));
}

#[test]
fn rejects_leading_slash() {
    assert!(matches!(
        SecretKey::parse("/ROOTED"),
        Err(SecretKeyError::LeadingSlash)
    ));
}

#[test]
fn rejects_dotdot_segment() {
    assert!(matches!(
        SecretKey::parse("valid/../bad"),
        Err(SecretKeyError::DotDotSegment)
    ));
}

#[test]
fn rejects_invalid_characters() {
    assert!(matches!(
        SecretKey::parse("bad space"),
        Err(SecretKeyError::InvalidChar { .. })
    ));
    assert!(matches!(
        SecretKey::parse("bad:colon"),
        Err(SecretKeyError::InvalidChar { .. })
    ));
}

// ── parse_canonical ─────────────────────────────────────────────────────────

#[test]
fn parse_canonical_accepts_valid_lowercase_namespaced() {
    assert!(SecretKey::parse_canonical("tavily/api_key").is_ok());
    assert!(SecretKey::parse_canonical("slack/bot_token").is_ok());
}

#[test]
fn parse_canonical_accepts_flat_lowercase() {
    // single-segment lowercase key is valid canonical
    assert!(SecretKey::parse_canonical("flat_lower").is_ok());
}

#[test]
fn parse_canonical_rejects_uppercase() {
    assert!(matches!(
        SecretKey::parse_canonical("SLACK_BOT_TOKEN"),
        Err(SecretKeyError::Uppercase)
    ));
}

#[test]
fn parse_canonical_rejects_mixed_case() {
    assert!(matches!(
        SecretKey::parse_canonical("Tavily/Api"),
        Err(SecretKeyError::Uppercase)
    ));
}

#[test]
fn parse_canonical_rejects_trailing_slash() {
    assert!(matches!(
        SecretKey::parse_canonical("acme/"),
        Err(SecretKeyError::TrailingSlash)
    ));
}

#[test]
fn parse_canonical_rejects_invalid_chars() {
    // colon — already rejected by base parse
    assert!(matches!(
        SecretKey::parse_canonical("secret://x"),
        Err(SecretKeyError::InvalidChar { .. })
    ));
    // asterisk — already rejected by base parse
    assert!(matches!(
        SecretKey::parse_canonical("*"),
        Err(SecretKeyError::InvalidChar { .. })
    ));
}

// ── normalize ────────────────────────────────────────────────────────────────

#[test]
fn normalize_uppercased_flat_key() {
    let key = SecretKey::normalize("SLACK_BOT_TOKEN").unwrap();
    assert_eq!(key.as_str(), "slack_bot_token");
}

#[test]
fn normalize_mixed_case_namespaced_key() {
    let key = SecretKey::normalize("Tavily/API_Key").unwrap();
    assert_eq!(key.as_str(), "tavily/api_key");
}

#[test]
fn normalize_already_lowercase_is_identity() {
    let key = SecretKey::normalize("tavily/api_key").unwrap();
    assert_eq!(key.as_str(), "tavily/api_key");
}

#[test]
fn normalize_rejects_invalid_chars_after_lowercasing() {
    // colons cannot be lowercased into valid chars
    assert!(matches!(
        SecretKey::normalize("secret://x"),
        Err(SecretKeyError::InvalidChar { .. })
    ));
}

#[test]
fn normalize_rejects_trailing_slash() {
    assert!(matches!(
        SecretKey::normalize("ACME/"),
        Err(SecretKeyError::TrailingSlash)
    ));
}
