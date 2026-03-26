//! Component QA schema (v0.6.0).
use alloc::{boxed::Box, collections::BTreeMap, string::String, vec::Vec};
use core::{fmt, str::FromStr};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use ciborium::value::Value;

use crate::i18n_text::I18nText;

// ---------------------------------------------------------------------------
// Skip expression types
// ---------------------------------------------------------------------------

/// Skip condition expression — supports AND/OR with nesting.
///
/// Used for conditional questions that should be skipped based on previous answers.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
pub enum SkipExpression {
    /// Single condition: field equals/not_equals/is_empty/is_not_empty.
    Condition(SkipCondition),
    /// All conditions must be true (logical AND).
    And(Vec<SkipExpression>),
    /// At least one condition must be true (logical OR).
    Or(Vec<SkipExpression>),
    /// Negate the inner expression (logical NOT).
    Not(Box<SkipExpression>),
}

/// Single skip condition for field comparison.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub struct SkipCondition {
    /// The field name to check in the answers.
    pub field: String,
    /// Skip if field equals this value.
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub equals: Option<Value>,
    /// Skip if field does not equal this value.
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub not_equals: Option<Value>,
    /// Skip if field is empty (null, missing, or empty string).
    #[cfg_attr(feature = "serde", serde(default))]
    pub is_empty: bool,
    /// Skip if field is not empty.
    #[cfg_attr(feature = "serde", serde(default))]
    pub is_not_empty: bool,
}

/// QA mode.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
pub enum QaMode {
    /// Default mode.
    Default,
    /// Setup mode.
    Setup,
    /// Update mode.
    #[cfg_attr(feature = "serde", serde(alias = "upgrade"))]
    Update,
    /// Remove mode.
    Remove,
}

impl QaMode {
    /// Canonical string form for this mode.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Default => "default",
            Self::Setup => "setup",
            Self::Update => "update",
            Self::Remove => "remove",
        }
    }
}

impl fmt::Display for QaMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for QaMode {
    type Err = &'static str;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "default" => Ok(Self::Default),
            "setup" => Ok(Self::Setup),
            "update" | "upgrade" => Ok(Self::Update),
            "remove" => Ok(Self::Remove),
            _ => Err("invalid QA mode"),
        }
    }
}

/// QA spec for a component.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub struct ComponentQaSpec {
    /// QA mode.
    pub mode: QaMode,
    /// Human-readable title.
    pub title: I18nText,
    /// Optional description.
    pub description: Option<I18nText>,
    /// Questions to present.
    pub questions: Vec<Question>,
    /// Default values (canonical order).
    pub defaults: BTreeMap<String, Value>,
}

impl ComponentQaSpec {
    /// Collect all i18n keys referenced by this spec.
    pub fn i18n_keys(&self) -> alloc::collections::BTreeSet<String> {
        let mut keys = alloc::collections::BTreeSet::new();
        keys.insert(self.title.key.clone());
        if let Some(desc) = &self.description {
            keys.insert(desc.key.clone());
        }
        for question in &self.questions {
            question.collect_i18n_keys(&mut keys);
        }
        keys
    }
}

/// Question entry.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub struct Question {
    /// Question identifier.
    pub id: String,
    /// Label shown to the user.
    pub label: I18nText,
    /// Optional help text.
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub help: Option<I18nText>,
    /// Optional error message (validation feedback).
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub error: Option<I18nText>,
    /// Kind of question.
    pub kind: QuestionKind,
    /// Whether the question is required.
    pub required: bool,
    /// Optional default value.
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub default: Option<Value>,
    /// Condition to skip this question based on previous answers.
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub skip_if: Option<SkipExpression>,
}

impl Question {
    fn collect_i18n_keys(&self, keys: &mut alloc::collections::BTreeSet<String>) {
        keys.insert(self.label.key.clone());
        if let Some(help) = &self.help {
            keys.insert(help.key.clone());
        }
        if let Some(error) = &self.error {
            keys.insert(error.key.clone());
        }
        if let QuestionKind::Choice { options } = &self.kind {
            for option in options {
                keys.insert(option.label.key.clone());
            }
        }
    }
}

/// Question type.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case", tag = "type"))]
pub enum QuestionKind {
    /// Free-form text input.
    Text,
    /// Choice list.
    Choice {
        /// Choice options presented to the user.
        options: Vec<ChoiceOption>,
    },
    /// Numeric input.
    Number,
    /// Boolean input.
    Bool,
    /// Inline JSON input with optional JSON Schema validation.
    InlineJson {
        /// Optional JSON Schema for validation (Draft 2020-12).
        #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
        schema: Option<Value>,
    },
    /// Asset file/directory path reference with optional existence check.
    AssetRef {
        /// Allowed file extensions (e.g., `["json", "yaml"]`).
        #[cfg_attr(feature = "serde", serde(default))]
        file_types: Vec<String>,
        /// Base path for resolving relative paths (e.g., `"assets/"`).
        #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
        base_path: Option<String>,
        /// Whether to check file existence (default: `true`).
        #[cfg_attr(feature = "serde", serde(default = "default_true"))]
        check_exists: bool,
        /// Whether remote references may be resolved by the host/orchestrator before use.
        #[cfg_attr(feature = "serde", serde(default, skip_serializing_if = "is_false"))]
        allow_remote: bool,
    },
}

fn default_true() -> bool {
    true
}

fn is_false(value: &bool) -> bool {
    !value
}

/// Choice option.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub struct ChoiceOption {
    /// Value returned for this option.
    pub value: String,
    /// Label shown to the user.
    pub label: I18nText,
}

#[cfg(test)]
mod tests {
    use super::{QaMode, QuestionKind};
    use alloc::string::ToString;
    use core::str::FromStr;

    #[test]
    fn from_str_accepts_upgrade_and_update() {
        assert_eq!(QaMode::from_str("upgrade"), Ok(QaMode::Update));
        assert_eq!(QaMode::from_str("update"), Ok(QaMode::Update));
    }

    #[test]
    fn display_emits_update() {
        assert_eq!(QaMode::Update.to_string(), "update");
    }

    #[cfg(feature = "serde")]
    #[test]
    fn serde_accepts_upgrade_and_emits_update_json() -> Result<(), Box<dyn std::error::Error>> {
        let legacy: QaMode = serde_json::from_str("\"upgrade\"")?;
        assert_eq!(legacy, QaMode::Update);

        let canonical = serde_json::to_string(&QaMode::Update)?;
        assert_eq!(canonical, "\"update\"");
        Ok(())
    }

    #[cfg(feature = "serde")]
    #[test]
    fn serde_accepts_upgrade_and_emits_update_cbor() -> Result<(), Box<dyn std::error::Error>> {
        let mut legacy_bytes = Vec::new();
        ciborium::ser::into_writer(
            &ciborium::value::Value::Text("upgrade".into()),
            &mut legacy_bytes,
        )?;
        let legacy = crate::CborBytes::new(legacy_bytes).decode::<QaMode>()?;
        assert_eq!(legacy, QaMode::Update);

        let canonical = crate::cbor::canonical::to_canonical_cbor(&QaMode::Update)?;
        let value: ciborium::value::Value = ciborium::de::from_reader(canonical.as_slice())?;
        assert_eq!(value, ciborium::value::Value::Text("update".into()));
        Ok(())
    }

    #[cfg(feature = "serde")]
    #[test]
    fn asset_ref_allow_remote_defaults_to_false_when_omitted()
    -> Result<(), Box<dyn std::error::Error>> {
        let kind: QuestionKind = serde_json::from_str(
            r#"{
                "type": "asset_ref",
                "file_types": ["json"],
                "base_path": "assets/",
                "check_exists": true
            }"#,
        )?;

        assert_eq!(
            kind,
            QuestionKind::AssetRef {
                file_types: vec!["json".to_string()],
                base_path: Some("assets/".to_string()),
                check_exists: true,
                allow_remote: false,
            }
        );

        let value = serde_json::to_value(&kind)?;
        let object = value
            .as_object()
            .ok_or_else(|| std::io::Error::other("asset_ref object"))?;
        assert!(!object.contains_key("allow_remote"));
        Ok(())
    }

    #[cfg(feature = "serde")]
    #[test]
    fn asset_ref_allow_remote_serializes_when_true() -> Result<(), Box<dyn std::error::Error>> {
        let kind = QuestionKind::AssetRef {
            file_types: vec!["json".to_string()],
            base_path: None,
            check_exists: true,
            allow_remote: true,
        };

        let value = serde_json::to_value(&kind)?;
        let object = value
            .as_object()
            .ok_or_else(|| std::io::Error::other("asset_ref object"))?;
        assert_eq!(
            object.get("allow_remote"),
            Some(&serde_json::Value::Bool(true))
        );
        Ok(())
    }
}
