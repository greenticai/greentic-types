//! State capability contract types for `greentic.cap.state.kv.v1`.
//!
//! Defines the operation enum, payloads, and results used by state provider
//! packs and the operator's native dispatch pipeline.

use alloc::string::String;
use alloc::vec::Vec;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Well-known capability identifier for key-value state providers.
pub const CAP_STATE_KV_V1: &str = "greentic.cap.state.kv.v1";

/// State operation kind.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
pub enum StateOp {
    /// Read a single key.
    Get,
    /// Write a single key.
    Put,
    /// Delete a single key.
    Delete,
    /// List keys matching a prefix.
    List,
    /// Compare-and-swap: update only if the current version matches.
    Cas,
}

impl StateOp {
    /// Returns the canonical operation name string.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Get => "state.get",
            Self::Put => "state.put",
            Self::Delete => "state.delete",
            Self::List => "state.list",
            Self::Cas => "state.cas",
        }
    }
}

impl core::fmt::Display for StateOp {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Payload for a state operation request.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct StateOpPayload {
    /// Namespace scoping the key, typically `{env}::{tenant}::{team}`.
    pub namespace: String,
    /// Key within the namespace.
    pub key: String,
    /// Value bytes for `Put` / `Cas` operations.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub value: Option<Vec<u8>>,
    /// Optional time-to-live in seconds.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub ttl_seconds: Option<u32>,
    /// Expected version for compare-and-swap.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub cas_version: Option<u64>,
    /// Optional key prefix for `List` operations.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub prefix: Option<String>,
}

impl StateOpPayload {
    /// Creates a `Get` payload.
    pub fn get(namespace: impl Into<String>, key: impl Into<String>) -> Self {
        Self {
            namespace: namespace.into(),
            key: key.into(),
            value: None,
            ttl_seconds: None,
            cas_version: None,
            prefix: None,
        }
    }

    /// Creates a `Put` payload.
    pub fn put(namespace: impl Into<String>, key: impl Into<String>, value: Vec<u8>) -> Self {
        Self {
            namespace: namespace.into(),
            key: key.into(),
            value: Some(value),
            ttl_seconds: None,
            cas_version: None,
            prefix: None,
        }
    }

    /// Creates a `Delete` payload.
    pub fn delete(namespace: impl Into<String>, key: impl Into<String>) -> Self {
        Self {
            namespace: namespace.into(),
            key: key.into(),
            value: None,
            ttl_seconds: None,
            cas_version: None,
            prefix: None,
        }
    }

    /// Creates a `List` payload with a prefix filter.
    pub fn list(namespace: impl Into<String>, prefix: impl Into<String>) -> Self {
        Self {
            namespace: namespace.into(),
            key: String::new(),
            value: None,
            ttl_seconds: None,
            cas_version: None,
            prefix: Some(prefix.into()),
        }
    }

    /// Sets a TTL on this payload.
    pub fn with_ttl(mut self, seconds: u32) -> Self {
        self.ttl_seconds = Some(seconds);
        self
    }

    /// Sets a CAS version on this payload.
    pub fn with_cas_version(mut self, version: u64) -> Self {
        self.cas_version = Some(version);
        self
    }
}

/// Result of a state operation.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct StateOpResult {
    /// Whether the key was found (for `Get`) or operation succeeded.
    pub found: bool,
    /// Returned value bytes (for `Get`).
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub value: Option<Vec<u8>>,
    /// Current version / etag if the backend supports it.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub version: Option<u64>,
    /// Matched keys for `List` operations.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub keys: Option<Vec<String>>,
    /// Error message if the operation failed.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub error: Option<String>,
}

impl StateOpResult {
    /// A successful result with a value.
    pub fn found(value: Vec<u8>) -> Self {
        Self {
            found: true,
            value: Some(value),
            version: None,
            keys: None,
            error: None,
        }
    }

    /// A not-found result.
    pub fn not_found() -> Self {
        Self {
            found: false,
            value: None,
            version: None,
            keys: None,
            error: None,
        }
    }

    /// A successful write/delete acknowledgement.
    pub fn ok() -> Self {
        Self {
            found: true,
            value: None,
            version: None,
            keys: None,
            error: None,
        }
    }

    /// A list result.
    pub fn list(keys: Vec<String>) -> Self {
        Self {
            found: true,
            value: None,
            version: None,
            keys: Some(keys),
            error: None,
        }
    }

    /// An error result.
    pub fn err(message: impl Into<String>) -> Self {
        Self {
            found: false,
            value: None,
            version: None,
            keys: None,
            error: Some(message.into()),
        }
    }

    /// Sets the version on this result.
    pub fn with_version(mut self, version: u64) -> Self {
        self.version = Some(version);
        self
    }
}

/// State backend kind resolved from capability offers.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(tag = "backend", rename_all = "snake_case"))]
pub enum StateBackendKind {
    /// In-memory ephemeral store (dev/test).
    Memory {
        /// Maximum number of entries (0 = unlimited).
        #[cfg_attr(feature = "serde", serde(default))]
        max_entries: u32,
        /// Default TTL in seconds (0 = no expiry).
        #[cfg_attr(feature = "serde", serde(default))]
        default_ttl_seconds: u32,
    },
    /// Redis-backed persistent store.
    Redis {
        /// Redis connection URL.
        redis_url: String,
        /// Key prefix for namespacing.
        #[cfg_attr(feature = "serde", serde(default = "default_key_prefix"))]
        key_prefix: String,
        /// Default TTL in seconds (0 = no expiry).
        #[cfg_attr(feature = "serde", serde(default))]
        default_ttl_seconds: u32,
        /// Connection pool size.
        #[cfg_attr(feature = "serde", serde(default = "default_pool_size"))]
        pool_size: u32,
        /// Whether TLS is enabled.
        #[cfg_attr(feature = "serde", serde(default))]
        tls_enabled: bool,
    },
}

#[cfg(feature = "serde")]
fn default_key_prefix() -> String {
    String::from("greentic")
}

#[cfg(feature = "serde")]
const fn default_pool_size() -> u32 {
    5
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn state_op_display() {
        assert_eq!(StateOp::Get.as_str(), "state.get");
        assert_eq!(StateOp::Put.as_str(), "state.put");
        assert_eq!(StateOp::Delete.as_str(), "state.delete");
        assert_eq!(StateOp::List.as_str(), "state.list");
        assert_eq!(StateOp::Cas.as_str(), "state.cas");
    }

    #[test]
    fn payload_builders() {
        let p = StateOpPayload::get("dev::t1::team", "session:abc");
        assert_eq!(p.namespace, "dev::t1::team");
        assert_eq!(p.key, "session:abc");
        assert!(p.value.is_none());

        let p = StateOpPayload::put("dev::t1::team", "k", vec![1, 2, 3]).with_ttl(60);
        assert_eq!(p.value, Some(vec![1, 2, 3]));
        assert_eq!(p.ttl_seconds, Some(60));

        let p = StateOpPayload::delete("ns", "k");
        assert!(p.value.is_none());

        let p = StateOpPayload::list("ns", "session:");
        assert_eq!(p.prefix, Some("session:".to_string()));
    }

    #[test]
    fn result_builders() {
        let r = StateOpResult::found(vec![42]);
        assert!(r.found);
        assert_eq!(r.value, Some(vec![42]));

        let r = StateOpResult::not_found();
        assert!(!r.found);

        let r = StateOpResult::ok().with_version(7);
        assert!(r.found);
        assert_eq!(r.version, Some(7));

        let r = StateOpResult::list(vec!["a".into(), "b".into()]);
        assert_eq!(r.keys, Some(vec!["a".to_string(), "b".to_string()]));

        let r = StateOpResult::err("boom");
        assert!(!r.found);
        assert_eq!(r.error, Some("boom".to_string()));
    }

    #[test]
    fn state_op_payload_json_roundtrip() {
        let original = StateOpPayload::put("ns", "key", b"hello".to_vec()).with_ttl(300);
        let json = match serde_json::to_string(&original) {
            Ok(value) => value,
            Err(err) => panic!("serialize failed: {err}"),
        };
        let decoded: StateOpPayload = match serde_json::from_str(&json) {
            Ok(value) => value,
            Err(err) => panic!("deserialize failed: {err}"),
        };
        assert_eq!(decoded.namespace, "ns");
        assert_eq!(decoded.key, "key");
        assert_eq!(decoded.value, Some(b"hello".to_vec()));
        assert_eq!(decoded.ttl_seconds, Some(300));
    }

    #[test]
    fn state_op_result_json_roundtrip() {
        let original = StateOpResult::found(b"world".to_vec()).with_version(42);
        let json = match serde_json::to_string(&original) {
            Ok(value) => value,
            Err(err) => panic!("serialize failed: {err}"),
        };
        let decoded: StateOpResult = match serde_json::from_str(&json) {
            Ok(value) => value,
            Err(err) => panic!("deserialize failed: {err}"),
        };
        assert!(decoded.found);
        assert_eq!(decoded.value, Some(b"world".to_vec()));
        assert_eq!(decoded.version, Some(42));
    }

    #[test]
    fn state_backend_kind_json_roundtrip() {
        let memory = StateBackendKind::Memory {
            max_entries: 10000,
            default_ttl_seconds: 0,
        };
        let json = match serde_json::to_string(&memory) {
            Ok(value) => value,
            Err(err) => panic!("serialize failed: {err}"),
        };
        assert!(json.contains("\"backend\":\"memory\""));
        let decoded: StateBackendKind = match serde_json::from_str(&json) {
            Ok(value) => value,
            Err(err) => panic!("deserialize failed: {err}"),
        };
        assert_eq!(decoded, memory);

        let redis = StateBackendKind::Redis {
            redis_url: "redis://localhost:6379/0".into(),
            key_prefix: "greentic".into(),
            default_ttl_seconds: 3600,
            pool_size: 10,
            tls_enabled: false,
        };
        let json = match serde_json::to_string(&redis) {
            Ok(value) => value,
            Err(err) => panic!("serialize failed: {err}"),
        };
        assert!(json.contains("\"backend\":\"redis\""));
        let decoded: StateBackendKind = match serde_json::from_str(&json) {
            Ok(value) => value,
            Err(err) => panic!("deserialize failed: {err}"),
        };
        assert_eq!(decoded, redis);
    }
}
