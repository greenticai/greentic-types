//! Simple i18n text wrapper used by CBOR schemas.
use alloc::string::String;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// I18n-aware text value with a stable key and optional fallback string.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct I18nText {
    /// Stable i18n key.
    pub key: String,
    /// Optional fallback string (usually legacy text).
    pub fallback: Option<String>,
}

impl I18nText {
    /// Create a new i18n text value.
    pub fn new(key: impl Into<String>, fallback: Option<String>) -> Self {
        Self {
            key: key.into(),
            fallback,
        }
    }

    /// Iterate over the key.
    pub fn keys(&self) -> impl Iterator<Item = &str> {
        core::iter::once(self.key.as_str())
    }
}
