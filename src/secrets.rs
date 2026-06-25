//! Canonical secret requirement primitives shared across Greentic crates.
//! All repos must use these helpers; local re-implementation is forbidden.

use crate::{ErrorCode, GResult, GreenticError};
use alloc::{format, string::String, vec::Vec};
use core::ops::Deref;
#[cfg(feature = "schemars")]
use schemars::JsonSchema;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Canonical secret identifier used across manifests and bindings.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[cfg_attr(feature = "serde", serde(transparent))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub struct SecretKey(String);

impl SecretKey {
    /// Constructs a secret key and validates the identifier format.
    pub fn new(key: impl Into<String>) -> GResult<Self> {
        let key = key.into();
        Self::parse(&key).map_err(|err| {
            GreenticError::new(
                ErrorCode::InvalidInput,
                format!("invalid secret key: {err}"),
            )
        })
    }

    /// Returns the key as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Parses and validates a secret key string.
    ///
    /// Validation rules:
    /// - must be non-empty
    /// - allowed characters: ASCII `a-zA-Z0-9._-/`
    /// - must not start with `/`
    /// - must not contain a `..` path segment
    pub fn parse(value: &str) -> Result<Self, SecretKeyError> {
        if value.is_empty() {
            return Err(SecretKeyError::Empty);
        }
        if value.starts_with('/') {
            return Err(SecretKeyError::LeadingSlash);
        }
        for c in value.chars() {
            if !(c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-' | '/')) {
                return Err(SecretKeyError::InvalidChar { c });
            }
        }
        if value.split('/').any(|segment| segment == "..") {
            return Err(SecretKeyError::DotDotSegment);
        }
        Ok(Self(value.to_owned()))
    }

    /// Parses and validates a secret key in **canonical** form.
    ///
    /// Canonical keys must satisfy all rules of [`Self::parse`] plus:
    /// - no ASCII uppercase letters (`A-Z`) — use [`Self::normalize`] to lowercase first
    /// - must not end with `/`
    ///
    /// A single-segment lowercase key (e.g. `"flat_lower"`) is valid; a `/` separator is
    /// not required.
    pub fn parse_canonical(value: &str) -> Result<Self, SecretKeyError> {
        // Run base validation first (handles empty, leading slash, invalid chars, `..`).
        Self::parse(value)?;

        // Reject any uppercase ASCII letter.
        if value.chars().any(|c| c.is_ascii_uppercase()) {
            return Err(SecretKeyError::Uppercase);
        }

        // Reject trailing `/`.
        if value.ends_with('/') {
            return Err(SecretKeyError::TrailingSlash);
        }

        Ok(Self(value.to_owned()))
    }

    /// Lowercases the input (ASCII only, lossless apart from case) and then validates it
    /// with [`Self::parse_canonical`].
    ///
    /// This is the preferred entry-point when ingesting externally-sourced keys that may
    /// use `UPPER_SNAKE_CASE` or `Mixed/Case` conventions.
    pub fn normalize(value: impl Into<String>) -> Result<Self, SecretKeyError> {
        let lowered = value.into().to_ascii_lowercase();
        Self::parse_canonical(&lowered)
    }
}

/// Validation errors produced by [`SecretKey::parse`] and [`SecretKey::parse_canonical`].
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum SecretKeyError {
    /// Input was empty.
    #[error("secret key must not be empty")]
    Empty,
    /// Input started with `/`.
    #[error("secret key must not start with '/'")]
    LeadingSlash,
    /// Input contained a `..` path segment.
    #[error("secret key must not contain '..' segments")]
    DotDotSegment,
    /// Input contained a disallowed character.
    #[error("secret key contains invalid character '{c}'")]
    InvalidChar {
        /// The offending character.
        c: char,
    },
    /// Canonical key contained an ASCII uppercase letter.
    ///
    /// Use [`SecretKey::normalize`] to lowercase the input before parsing canonically.
    #[error("canonical secret key must not contain uppercase letters")]
    Uppercase,
    /// Canonical key ended with `/`.
    #[error("canonical secret key must not end with '/'")]
    TrailingSlash,
}

impl Deref for SecretKey {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<String> for SecretKey {
    fn from(key: String) -> Self {
        Self(key)
    }
}

impl From<&str> for SecretKey {
    fn from(key: &str) -> Self {
        Self(key.to_owned())
    }
}

impl From<SecretKey> for String {
    fn from(key: SecretKey) -> Self {
        key.0
    }
}

#[cfg(feature = "serde")]
impl<'de> Deserialize<'de> for SecretKey {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        SecretKey::parse(&value).map_err(serde::de::Error::custom)
    }
}

/// Canonical secret scope (environment, tenant, team).
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub struct SecretScope {
    /// Environment identifier (e.g., `dev`, `prod`).
    pub env: String,
    /// Tenant identifier within the environment.
    pub tenant: String,
    /// Optional team for finer-grained isolation.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub team: Option<String>,
}

/// Supported secret content formats.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "lowercase"))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub enum SecretFormat {
    /// Arbitrary bytes.
    Bytes,
    /// UTF-8 text.
    Text,
    /// JSON document.
    Json,
}

/// Structured secret requirement used in capabilities, bindings, and deployment plans.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub struct SecretRequirement {
    /// Logical key the runtime should resolve.
    pub key: SecretKey,
    /// Whether the secret is mandatory for execution.
    #[cfg_attr(
        feature = "serde",
        serde(default = "SecretRequirement::default_required")
    )]
    pub required: bool,
    /// Optional description for operator-facing tooling.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub description: Option<String>,
    /// Expected scope for resolution (environment/tenant/team).
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub scope: Option<SecretScope>,
    /// Preferred secret format when known.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub format: Option<SecretFormat>,
    /// Optional JSON Schema fragment describing the value shape.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub schema: Option<serde_json::Value>,
    /// Example payloads for documentation.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Vec::is_empty")
    )]
    pub examples: Vec<String>,
}

impl Default for SecretRequirement {
    fn default() -> Self {
        Self {
            key: SecretKey::default(),
            required: true,
            description: None,
            scope: None,
            format: None,
            schema: None,
            examples: Vec::new(),
        }
    }
}

impl SecretRequirement {
    const fn default_required() -> bool {
        true
    }
}
