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
    let rest = type_str
        .strip_prefix(BUSINESS_EVENT_TYPE_PREFIX)
        .ok_or_else(|| {
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
        return Err(format!(
            "event type '{type_str}' has an empty domain or name segment"
        ));
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
            _ => errors.push(format!(
                "metadata['{key}'] is required and must be non-empty"
            )),
        }
    }
    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

/// Ergonomic builder for `greentic.business-event.v1` envelopes. Assembles the
/// `type`/`topic` from `domain`/`name`, places `producer`/`schema_version` into
/// metadata, and validates on `build`.
pub struct BusinessEventBuilder {
    domain: String,
    name: String,
    tenant: TenantCtx,
    source: String,
    producer: Option<String>,
    schema_version: Option<String>,
    payload: Value,
    id: Option<String>,
    subject: Option<String>,
    correlation_id: Option<String>,
    time: DateTime<Utc>,
}

impl BusinessEventBuilder {
    /// Create a new builder for the given `domain` and event `name` under a `tenant`.
    pub fn new(domain: impl Into<String>, name: impl Into<String>, tenant: TenantCtx) -> Self {
        Self {
            domain: domain.into(),
            name: name.into(),
            tenant,
            source: "greentic".to_string(),
            producer: None,
            schema_version: None,
            payload: Value::Null,
            id: None,
            subject: None,
            correlation_id: None,
            time: DateTime::UNIX_EPOCH,
        }
    }

    /// Override the event `source` (defaults to `"greentic"`).
    pub fn source(mut self, source: impl Into<String>) -> Self {
        self.source = source.into();
        self
    }

    /// Set the `producer` metadata field (required).
    pub fn producer(mut self, producer: impl Into<String>) -> Self {
        self.producer = Some(producer.into());
        self
    }

    /// Set the `schema_version` metadata field (required).
    pub fn schema_version(mut self, v: impl Into<String>) -> Self {
        self.schema_version = Some(v.into());
        self
    }

    /// Set the opaque JSON `payload`.
    pub fn payload(mut self, payload: Value) -> Self {
        self.payload = payload;
        self
    }

    /// Set the event `id`. Must be a valid identifier (alphanumeric + hyphens).
    pub fn id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }

    /// Set an optional `subject` string for the event.
    pub fn subject(mut self, subject: impl Into<String>) -> Self {
        self.subject = Some(subject.into());
        self
    }

    /// Set an optional `correlation_id` to link related messages.
    pub fn correlation_id(mut self, c: impl Into<String>) -> Self {
        self.correlation_id = Some(c.into());
        self
    }

    /// Set the event timestamp (defaults to [`DateTime::UNIX_EPOCH`]).
    pub fn time(mut self, time: DateTime<Utc>) -> Self {
        self.time = time;
        self
    }

    /// Assemble and validate. Returns all profile violations on failure.
    pub fn build(self) -> Result<EventEnvelope, Vec<String>> {
        let mut errors = Vec::new();
        let id_str = self.id.unwrap_or_default();
        let id = match EventId::new(&id_str) {
            Ok(id) => Some(id),
            Err(err) => {
                errors.push(format!("invalid event id '{id_str}': {err}"));
                None
            }
        };
        let mut metadata = EventMetadata::new();
        match self.producer {
            Some(p) if !p.is_empty() => {
                metadata.insert("producer".to_string(), p);
            }
            _ => errors.push("metadata['producer'] is required and must be non-empty".to_string()),
        }
        match self.schema_version {
            Some(v) if !v.is_empty() => {
                metadata.insert("schema_version".to_string(), v);
            }
            _ => errors
                .push("metadata['schema_version'] is required and must be non-empty".to_string()),
        }
        let id = match id {
            Some(id) => id,
            None => return Err(errors),
        };
        let event = EventEnvelope {
            id,
            topic: self.domain.clone(),
            r#type: format!("{BUSINESS_EVENT_TYPE_PREFIX}{}/{}", self.domain, self.name),
            source: self.source,
            tenant: self.tenant,
            subject: self.subject,
            time: self.time,
            correlation_id: self.correlation_id,
            payload: self.payload,
            metadata,
        };
        if !errors.is_empty() {
            return Err(errors);
        }
        validate_business_event(&event)?;
        Ok(event)
    }
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
        assert!(
            errors.iter().any(|m| m.contains("schema_version")),
            "{errors:?}"
        );
        assert!(errors.iter().any(|m| m.contains("producer")), "{errors:?}");
    }

    #[test]
    fn builder_produces_a_valid_event() {
        let event = BusinessEventBuilder::new("tenancy", "daily-rent-reminder", ctx())
            .producer("trigger:daily")
            .schema_version("1")
            .source("operala")
            .payload(serde_json::json!({"hello": "world"}))
            .id("evt-42")
            .time(chrono::DateTime::UNIX_EPOCH)
            .build()
            .expect("builds valid event");
        assert_eq!(
            event.r#type,
            "cap://greentic/events/tenancy/daily-rent-reminder"
        );
        assert_eq!(event.topic, "tenancy");
        assert_eq!(event.metadata.get("producer").unwrap(), "trigger:daily");
        validate_business_event(&event).expect("builder output validates");
    }

    #[test]
    fn builder_rejects_missing_producer() {
        let errors = BusinessEventBuilder::new("tenancy", "x", ctx())
            .schema_version("1")
            .id("evt-1")
            .build()
            .unwrap_err();
        assert!(errors.iter().any(|m| m.contains("producer")), "{errors:?}");
    }
}
