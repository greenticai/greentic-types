//! Pack QA schema (v0.6.0).
use alloc::{collections::BTreeMap, string::String, vec::Vec};
use core::{fmt, str::FromStr};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use ciborium::value::Value;

use crate::i18n_text::I18nText;

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

/// QA spec for a pack.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub struct PackQaSpec {
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

impl PackQaSpec {
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
    pub help: Option<I18nText>,
    /// Optional error message (validation feedback).
    pub error: Option<I18nText>,
    /// Kind of question.
    pub kind: QuestionKind,
    /// Whether the question is required.
    pub required: bool,
    /// Optional default value.
    pub default: Option<Value>,
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
        match &self.kind {
            QuestionKind::Choice { options } => {
                for option in options {
                    keys.insert(option.label.key.clone());
                }
            }
            QuestionKind::Table {
                columns,
                add_label,
                empty_label,
                ..
            } => {
                if let Some(t) = add_label {
                    keys.insert(t.key.clone());
                }
                if let Some(t) = empty_label {
                    keys.insert(t.key.clone());
                }
                for column in columns {
                    keys.insert(column.label.key.clone());
                    if let Some(t) = &column.help {
                        keys.insert(t.key.clone());
                    }
                    if let Some(t) = &column.placeholder {
                        keys.insert(t.key.clone());
                    }
                    if let ColumnKind::Choice { options } = &column.kind {
                        for option in options {
                            keys.insert(option.label.key.clone());
                        }
                    }
                }
            }
            QuestionKind::Text | QuestionKind::Number | QuestionKind::Bool => {}
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
    /// Repeating-row input — answer is a JSON array of objects, one per row,
    /// keyed by the column `key` field. Used for setup answers like
    /// `nav_links` whose runtime shape is a list of structured items but
    /// today are entered as a JSON-string-as-array workaround.
    ///
    /// See `docs/rfcs/2026-05-02-questionkind-table.md` for the design.
    Table {
        /// Column definitions, in display order.
        columns: Vec<TableColumn>,
        /// Minimum required row count (default 0).
        #[cfg_attr(
            feature = "serde",
            serde(default, skip_serializing_if = "Option::is_none")
        )]
        min_rows: Option<u16>,
        /// Maximum row count (default unbounded).
        #[cfg_attr(
            feature = "serde",
            serde(default, skip_serializing_if = "Option::is_none")
        )]
        max_rows: Option<u16>,
        /// Optional i18n key for the "add row" button label.
        #[cfg_attr(
            feature = "serde",
            serde(default, skip_serializing_if = "Option::is_none")
        )]
        add_label: Option<I18nText>,
        /// Optional i18n key for the empty-state placeholder.
        #[cfg_attr(
            feature = "serde",
            serde(default, skip_serializing_if = "Option::is_none")
        )]
        empty_label: Option<I18nText>,
    },
}

/// One column in a `QuestionKind::Table`. Each row's answer is a JSON
/// object whose keys match the columns' `key` field.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub struct TableColumn {
    /// JSON object key the column's value is stored under (e.g. `"label"`,
    /// `"url"`). Stable identifier — do not rename without a migration.
    pub key: String,
    /// Header label shown above the column / next to each row's input.
    pub label: I18nText,
    /// Column data kind. Intentionally narrower than `QuestionKind` —
    /// nested tables are not supported (keeps the schema and renderers
    /// simple).
    pub kind: ColumnKind,
    /// Whether the column must be filled for a row to count as non-empty.
    /// A row whose required columns are all empty is dropped silently
    /// (lets operators leave a "blank" row mid-table).
    pub required: bool,
    /// Optional inline help (rendered as a tooltip / description).
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub help: Option<I18nText>,
    /// Optional placeholder shown when the cell is empty.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub placeholder: Option<I18nText>,
    /// Optional per-row default value (applied to new rows only).
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub default: Option<Value>,
}

/// Column data kind — a deliberately narrowed subset of `QuestionKind`.
/// No nesting; a column cannot itself be a table.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case", tag = "type"))]
pub enum ColumnKind {
    /// Free-form text input.
    Text,
    /// Choice / dropdown.
    Choice {
        /// Choice options presented to the user.
        options: Vec<ChoiceOption>,
    },
    /// Numeric input.
    Number,
    /// Boolean toggle.
    Bool,
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
    use super::QaMode;
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
    fn serde_table_kind_round_trips_json() -> Result<(), Box<dyn std::error::Error>> {
        use super::{ChoiceOption, ColumnKind, QuestionKind, TableColumn};
        use crate::i18n_text::I18nText;
        let kind = QuestionKind::Table {
            columns: alloc::vec![
                TableColumn {
                    key: "label".to_string(),
                    label: I18nText::new("nav.col.label".to_string(), None),
                    kind: ColumnKind::Text,
                    required: true,
                    help: None,
                    placeholder: Some(I18nText::new("nav.col.label.placeholder".to_string(), None)),
                    default: None,
                },
                TableColumn {
                    key: "external".to_string(),
                    label: I18nText::new("nav.col.external".to_string(), None),
                    kind: ColumnKind::Bool,
                    required: false,
                    help: None,
                    placeholder: None,
                    default: None,
                },
                TableColumn {
                    key: "method".to_string(),
                    label: I18nText::new("nav.col.method".to_string(), None),
                    kind: ColumnKind::Choice {
                        options: alloc::vec![ChoiceOption {
                            value: "GET".to_string(),
                            label: I18nText::new("nav.col.method.get".to_string(), None),
                        }],
                    },
                    required: false,
                    help: None,
                    placeholder: None,
                    default: None,
                },
            ],
            min_rows: None,
            max_rows: Some(8),
            add_label: Some(I18nText::new("nav.add".to_string(), None)),
            empty_label: None,
        };
        let json = serde_json::to_string(&kind)?;
        let back: QuestionKind = serde_json::from_str(&json)?;
        assert_eq!(back, kind, "table kind must round-trip via JSON");
        // The legacy primitive variants must keep their existing tagged form
        // (back-compat with packs already on the registry).
        let text_json = serde_json::to_string(&QuestionKind::Text)?;
        assert_eq!(text_json, r#"{"type":"text"}"#);
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
}
