//! Uniform node I/O contract types (spec §4.2–§4.4).
//!
//! - [`NodeOutput`] — a node's output, **mutually exclusive** `{data}` (success)
//!   or `{errors}` (failure), per JSON:API / JSON-RPC 2.0.
//! - [`NodeError`] — a Temporal-shaped error object (`code`/`message`/`kind`/
//!   `retryable`/`source`/`details`).
//! - [`InputValue`] / [`StructuredInput`] — explicit literal-vs-ref-vs-template
//!   inputs (Step Functions lesson: encode it structurally, never sniff delimiters).
//! - [`FlowState`] — the per-node output namespace, keyed by node id (n8n model).

use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;

use serde_json::Value;

#[cfg(feature = "schemars")]
use schemars::JsonSchema;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Coarse, routing/retry-oriented classification of a [`NodeError`]. This is the
/// branch/retry key (Step Functions `ErrorEquals`, Temporal failure `type`);
/// `code` stays the canonical open-ended string.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub enum ErrorKind {
    /// Input failed validation.
    Validation,
    /// Operation timed out.
    Timeout,
    /// Authentication / authorization failure.
    Auth,
    /// Required entity not found.
    NotFound,
    /// Throttled by rate limits.
    RateLimited,
    /// Unclassified internal failure.
    Internal,
    /// Operation cancelled (e.g. a user cancelled a form/QA step).
    Cancelled,
}

/// Where an error originated — the node and (optionally) the input that failed.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub struct NodeErrorSource {
    /// The node id that produced the error.
    pub node: String,
    /// The input name that triggered it, when applicable.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub input: Option<String>,
}

/// A single error in a node's `{errors}` envelope.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub struct NodeError {
    /// Stable, author/UI-facing error code (open-ended string, per JSON:API).
    pub code: String,
    /// Required human-readable message (one concise sentence).
    pub message: String,
    /// Coarse routing/retry classification.
    pub kind: ErrorKind,
    /// Explicit retryability — never inferred from `kind` (Temporal's `non_retryable`).
    pub retryable: bool,
    /// Optional provenance.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub source: Option<NodeErrorSource>,
    /// Free-form structured detail; omitted from the wire when null.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Value::is_null")
    )]
    pub details: Value,
}

/// A node's output: success carries `{data}`, failure carries `{errors}`. The two
/// are mutually exclusive (enforced by the type); engine metadata lives in a
/// sibling `meta`, never inside `data`.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(untagged))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub enum NodeOutput {
    /// Success: arbitrary node-defined fields under `data`.
    Data {
        /// The success payload.
        data: Value,
    },
    /// Failure: one or more errors under `errors`.
    Errors {
        /// The error list (a node may report several validation failures at once).
        errors: Vec<NodeError>,
    },
}

impl NodeOutput {
    /// Build a success output `{ data }`.
    pub fn ok(data: Value) -> Self {
        Self::Data { data }
    }

    /// Build a failure output `{ errors }`.
    pub fn failed(errors: Vec<NodeError>) -> Self {
        Self::Errors { errors }
    }

    /// `true` when this is a success (`{data}`) output.
    pub fn is_ok(&self) -> bool {
        matches!(self, Self::Data { .. })
    }

    /// The success payload, or `None` on a failure output.
    pub fn data(&self) -> Option<&Value> {
        match self {
            Self::Data { data } => Some(data),
            Self::Errors { .. } => None,
        }
    }

    /// The error list, or an empty slice on a success output.
    pub fn errors(&self) -> &[NodeError] {
        match self {
            Self::Errors { errors } => errors,
            Self::Data { .. } => &[],
        }
    }
}

/// An explicit node input value — resolved structurally, never by sniffing
/// delimiters. Serializes to a single-key object: `{literal|ref|template}`.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub enum InputValue {
    /// Used as-is.
    Literal(Value),
    /// A single addressable path (`node.<id>.data.<field>`), resolved to its typed value.
    Ref(String),
    /// String interpolation; always yields a string.
    Template(String),
}

/// A named node input (`{ name, value }`).
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub struct StructuredInput {
    /// The input parameter name.
    pub name: String,
    /// Its value: literal, reference, or template.
    pub value: InputValue,
}

/// The per-node output namespace: each node's [`NodeOutput`] kept immutable,
/// keyed by its stable node id. Downstream addresses `<node_id>.data.<path>`.
pub type FlowState = BTreeMap<String, NodeOutput>;
