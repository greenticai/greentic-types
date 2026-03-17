//! Adapter from legacy v0.5.0 QA to v0.6.0 Component QA.
use alloc::{collections::BTreeMap, string::String, vec::Vec};

use ciborium::value::Value;

use crate::cbor::canonical;
use crate::cbor_bytes::CborBytes;
use crate::i18n_text::I18nText;
use crate::schemas::component::v0_5_0::LegacyComponentQaSpec;
use crate::schemas::component::v0_6_0::{
    ChoiceOption, ComponentQaSpec, QaMode, Question, QuestionKind,
};

#[cfg(feature = "serde")]
/// Parse JSON and adapt legacy v0.5.0 QA into canonical v0.6.0 CBOR.
pub fn adapt_component_qa_spec_json(
    mode: QaMode,
    legacy_json: &str,
) -> canonical::Result<CborBytes> {
    let legacy: LegacyComponentQaSpec = serde_json::from_str(legacy_json)
        .map_err(|err| canonical::CanonicalError::Decode(err.to_string()))?;
    adapt_component_qa_spec(mode, &legacy)
}

/// Adapt a legacy v0.5.0 QA spec into canonical v0.6.0 CBOR.
pub fn adapt_component_qa_spec(
    mode: QaMode,
    legacy: &LegacyComponentQaSpec,
) -> canonical::Result<CborBytes> {
    let spec = map_component_qa_spec(mode, legacy);
    let bytes = canonical::to_canonical_cbor(&spec)?;
    Ok(CborBytes::new(bytes))
}

fn map_component_qa_spec(mode: QaMode, legacy: &LegacyComponentQaSpec) -> ComponentQaSpec {
    let title_key = "legacy.component.v0_5_0.title".to_string();
    let description_key = "legacy.component.v0_5_0.description".to_string();

    let questions = legacy
        .questions
        .iter()
        .map(map_question)
        .collect::<Vec<_>>();

    ComponentQaSpec {
        mode,
        title: I18nText::new(title_key, Some(legacy.title.clone())),
        description: legacy
            .description
            .as_ref()
            .map(|value| I18nText::new(description_key.clone(), Some(value.clone()))),
        questions,
        defaults: collect_defaults(legacy),
    }
}

fn collect_defaults(legacy: &LegacyComponentQaSpec) -> BTreeMap<String, Value> {
    let mut defaults = BTreeMap::new();
    for question in &legacy.questions {
        if let Some(value) = &question.default {
            defaults.insert(question.id.clone(), value.clone());
        }
    }
    defaults
}

fn map_question(question: &crate::schemas::component::v0_5_0::qa::LegacyQuestion) -> Question {
    let label_key = format!("legacy.component.v0_5_0.{}.label", question.id);
    let help_key = format!("legacy.component.v0_5_0.{}.help", question.id);

    Question {
        id: question.id.clone(),
        label: I18nText::new(label_key, Some(question.label.clone())),
        help: question
            .help
            .as_ref()
            .map(|value| I18nText::new(help_key.clone(), Some(value.clone()))),
        error: None,
        kind: map_kind(question),
        required: question.required,
        default: question.default.clone(),
        skip_if: None,
    }
}

fn map_kind(question: &crate::schemas::component::v0_5_0::qa::LegacyQuestion) -> QuestionKind {
    use crate::schemas::component::v0_5_0::qa::LegacyQuestionKind as LegacyKind;

    match &question.kind {
        LegacyKind::Text => QuestionKind::Text,
        LegacyKind::Number => QuestionKind::Number,
        LegacyKind::Bool => QuestionKind::Bool,
        LegacyKind::Choice => {
            let options = question
                .choices
                .as_ref()
                .map(|choices| {
                    choices
                        .iter()
                        .map(|choice| ChoiceOption {
                            value: choice.value.clone(),
                            label: I18nText::new(
                                format!(
                                    "legacy.component.v0_5_0.{}.option.{}",
                                    question.id, choice.value
                                ),
                                Some(choice.label.clone()),
                            ),
                        })
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();
            QuestionKind::Choice { options }
        }
    }
}
