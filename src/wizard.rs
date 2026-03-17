//! Wizard planning and delegation primitives.

use alloc::{collections::BTreeMap, string::String, vec::Vec};

#[cfg(feature = "schemars")]
use schemars::JsonSchema;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Stable wizard identifier.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[cfg_attr(feature = "serde", serde(transparent))]
pub struct WizardId(pub String);

impl WizardId {
    /// Returns the identifier as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<String> for WizardId {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl From<&str> for WizardId {
    fn from(value: &str) -> Self {
        Self(value.to_owned())
    }
}

impl From<WizardId> for String {
    fn from(value: WizardId) -> Self {
        value.0
    }
}

/// Target system that will execute wizard actions.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
pub enum WizardTarget {
    /// Component scope.
    Component,
    /// Flow scope.
    Flow,
    /// Pack scope.
    Pack,
    /// Operator scope.
    Operator,
    /// Development tooling scope.
    Dev,
    /// Bundle scope.
    Bundle,
}

/// Canonical wizard mode.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
pub enum WizardMode {
    /// Default mode.
    Default,
    /// Setup mode.
    Setup,
    /// Update mode.
    Update,
    /// Remove mode.
    Remove,
    /// Scaffold mode.
    Scaffold,
    /// Build mode.
    Build,
    /// New mode.
    New,
}

/// Wizard plan metadata.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub struct WizardPlanMeta {
    /// Wizard identifier.
    pub id: WizardId,
    /// Wizard target scope.
    pub target: WizardTarget,
    /// Requested execution mode.
    pub mode: WizardMode,
}

/// Deterministic wizard plan.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub struct WizardPlan {
    /// Plan metadata and routing identity.
    pub meta: WizardPlanMeta,
    /// Ordered steps to execute.
    pub steps: Vec<WizardStep>,
}

/// A single wizard plan step.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case", tag = "type"))]
pub enum WizardStep {
    /// Ensure directories exist.
    EnsureDir {
        /// Directories to ensure.
        paths: Vec<String>,
    },
    /// Write files as path -> UTF-8 content.
    WriteFiles {
        /// Map of relative path to content.
        files: BTreeMap<String, String>,
    },
    /// Legacy bridge to invoke CLI commands.
    RunCli {
        /// Executable command.
        command: String,
        /// Command arguments.
        #[cfg_attr(
            feature = "serde",
            serde(default, skip_serializing_if = "Vec::is_empty")
        )]
        args: Vec<String>,
    },
    /// Delegate part of the wizard to another target/id/mode.
    Delegate {
        /// Target scope for delegation.
        target: WizardTarget,
        /// Delegate wizard identifier.
        id: WizardId,
        /// Delegate mode.
        mode: WizardMode,
        /// Optional prefilled answers for deterministic replay.
        #[cfg_attr(
            feature = "serde",
            serde(default, skip_serializing_if = "BTreeMap::is_empty")
        )]
        prefilled_answers: BTreeMap<String, Value>,
        /// Optional output key remapping.
        #[cfg_attr(
            feature = "serde",
            serde(default, skip_serializing_if = "BTreeMap::is_empty")
        )]
        output_map: BTreeMap<String, String>,
    },
}
