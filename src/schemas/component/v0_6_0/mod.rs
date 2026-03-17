//! Component schemas (v0.6.0).
pub mod describe;
pub mod qa;

pub use crate::i18n_text::I18nText;
#[cfg(all(feature = "std", feature = "serde"))]
pub use describe::schema_hash;
pub use describe::{
    ComponentDescribe, ComponentInfo, ComponentOperation, ComponentRunInput, ComponentRunOutput,
    RedactionKind, RedactionRule,
};
pub use qa::{
    ChoiceOption, ComponentQaSpec, QaMode, Question, QuestionKind, SkipCondition, SkipExpression,
};
