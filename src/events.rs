//! Canonical Greentic event envelope shared across repos.

use alloc::string::{String, ToString};
use alloc::vec::Vec;
use alloc::{collections::BTreeMap, format};
use core::fmt;
use core::str::FromStr;

use chrono::{DateTime, Utc};
#[cfg(feature = "schemars")]
use schemars::JsonSchema;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{GResult, GreenticError, TenantCtx, validate_identifier};

/// Map of metadata entries propagated with an event.
pub type EventMetadata = BTreeMap<String, String>;

/// Stable identifier for an event envelope.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[cfg_attr(feature = "serde", serde(try_from = "String", into = "String"))]
pub struct EventId(String);

impl EventId {
    /// Returns the identifier as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Validates and constructs the identifier from the provided value.
    pub fn new(value: impl AsRef<str>) -> GResult<Self> {
        value.as_ref().parse()
    }

    /// Consumes the identifier returning the owned string.
    pub fn into_inner(self) -> String {
        self.0
    }
}

impl From<EventId> for String {
    fn from(value: EventId) -> Self {
        value.0
    }
}

impl AsRef<str> for EventId {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl fmt::Display for EventId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for EventId {
    type Err = GreenticError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        validate_identifier(value, "EventId")?;
        Ok(Self(value.to_owned()))
    }
}

impl TryFrom<String> for EventId {
    type Error = GreenticError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        EventId::from_str(&value)
    }
}

impl TryFrom<&str> for EventId {
    type Error = GreenticError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        EventId::from_str(value)
    }
}

/// Generic envelope for cross-service events.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub struct EventEnvelope {
    /// Stable identifier for the event.
    pub id: EventId,
    /// Logical topic for routing (for example `greentic.repo.build.status`).
    pub topic: String,
    /// Fully qualified event type identifier (for example `com.greentic.repo.build.status.v1`).
    pub r#type: String,
    /// Originator of the event (DID, URI, or service identifier).
    pub source: String,
    /// Tenant context propagated with the event.
    pub tenant: TenantCtx,
    /// Optional subject tied to the event (for example `repo:my-service`).
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub subject: Option<String>,
    /// Event timestamp in UTC.
    #[cfg_attr(
        feature = "schemars",
        schemars(with = "String", description = "RFC3339 timestamp in UTC")
    )]
    pub time: DateTime<Utc>,
    /// Optional correlation identifier to link related messages.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub correlation_id: Option<String>,
    /// Opaque JSON payload representing the event body.
    pub payload: Value,
    /// Free-form metadata such as idempotency keys.
    #[cfg_attr(feature = "serde", serde(default))]
    pub metadata: EventMetadata,
}

/// Schema identifier for the business-event profile.
pub const BUSINESS_EVENT_SCHEMA: &str = "greentic.business-event.v1";

const BUSINESS_EVENT_TYPE_PREFIX: &str = "cap://greentic/events/";

/// Parse a business-event `type` of the form
/// `cap://greentic/events/{domain}/{name}` into `(domain, name)`.
pub fn parse_business_event_type(type_str: &str) -> Result<(String, String), String> {
    let rest = type_str.strip_prefix(BUSINESS_EVENT_TYPE_PREFIX).ok_or_else(|| {
        format!("event type '{type_str}' must start with '{BUSINESS_EVENT_TYPE_PREFIX}'")
    })?;
    let segments: Vec<&str> = rest.split('/').collect();
    if segments.len() != 2 {
        return Err(format!(
            "event type '{type_str}' must be cap://greentic/events/{{domain}}/{{name}} (got {} segment(s) after the prefix)",
            segments.len()
        ));
    }
    let domain = segments[0];
    let name = segments[1];
    if domain.is_empty() || name.is_empty() {
        return Err(format!("event type '{type_str}' has an empty domain or name segment"));
    }
    Ok((domain.to_string(), name.to_string()))
}

/// Validate an [`EventEnvelope`] against the `greentic.business-event.v1`
/// profile. Returns ALL violations so callers can correct everything at once.
pub fn validate_business_event(event: &EventEnvelope) -> Result<(), Vec<String>> {
    let mut errors: Vec<String> = Vec::new();
    match parse_business_event_type(&event.r#type) {
        Ok((domain, _name)) => {
            if event.topic != domain {
                errors.push(format!(
                    "topic '{}' must equal the domain '{}' from the event type",
                    event.topic, domain
                ));
            }
        }
        Err(message) => errors.push(message),
    }
    for key in ["schema_version", "producer"] {
        match event.metadata.get(key) {
            Some(value) if !value.is_empty() => {}
            _ => errors.push(format!("metadata['{key}'] is required and must be non-empty")),
        }
    }
    if errors.is_empty() { Ok(()) } else { Err(errors) }
}

#[cfg(all(test, feature = "std"))]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod business_event_tests {
    use super::*;
    use crate::{EnvId, TenantCtx, TenantId};
    use alloc::string::ToString;

    fn ctx() -> TenantCtx {
        TenantCtx::new(
            EnvId::try_from("prod").unwrap(),
            TenantId::try_from("tenant-1").unwrap(),
        )
    }

    fn valid_event() -> EventEnvelope {
        let mut metadata = EventMetadata::new();
        metadata.insert("schema_version".to_string(), "1".to_string());
        metadata.insert("producer".to_string(), "trigger:daily".to_string());
        EventEnvelope {
            id: EventId::new("evt-1").unwrap(),
            topic: "tenancy".to_string(),
            r#type: "cap://greentic/events/tenancy/daily-rent-reminder".to_string(),
            source: "operala".to_string(),
            tenant: ctx(),
            subject: None,
            time: chrono::DateTime::UNIX_EPOCH,
            correlation_id: None,
            payload: serde_json::json!({}),
            metadata,
        }
    }

    #[test]
    fn parses_well_formed_business_event_type() {
        let (domain, name) =
            parse_business_event_type("cap://greentic/events/tenancy/daily-rent-reminder").unwrap();
        assert_eq!(domain, "tenancy");
        assert_eq!(name, "daily-rent-reminder");
    }

    #[test]
    fn rejects_type_without_cap_prefix() {
        assert!(parse_business_event_type("greentic/events/x/y").is_err());
    }

    #[test]
    fn rejects_type_with_wrong_segment_count() {
        assert!(parse_business_event_type("cap://greentic/events/onlyone").is_err());
        assert!(parse_business_event_type("cap://greentic/events/a/b/c").is_err());
    }

    #[test]
    fn valid_event_passes_validation() {
        validate_business_event(&valid_event()).expect("valid event");
    }

    #[test]
    fn topic_must_equal_domain() {
        let mut e = valid_event();
        e.topic = "wrong".to_string();
        let errors = validate_business_event(&e).unwrap_err();
        assert!(errors.iter().any(|m| m.contains("topic")), "{errors:?}");
    }

    #[test]
    fn missing_metadata_keys_are_reported_together() {
        let mut e = valid_event();
        e.metadata.clear();
        let errors = validate_business_event(&e).unwrap_err();
        assert!(errors.iter().any(|m| m.contains("schema_version")), "{errors:?}");
        assert!(errors.iter().any(|m| m.contains("producer")), "{errors:?}");
    }
}
