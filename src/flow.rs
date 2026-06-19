//! Unified flow model used by packs and runtimes.

use alloc::collections::{BTreeMap, BTreeSet};
use alloc::string::String;
use core::hash::BuildHasherDefault;

use fnv::FnvHasher;
use indexmap::IndexMap;
use serde_json::Value;

use crate::{ComponentId, FlowId, NodeId};

/// Build hasher used for flow node maps (Fnv for `no_std` friendliness).
pub type FlowHasher = BuildHasherDefault<FnvHasher>;

#[cfg(feature = "schemars")]
use schemars::JsonSchema;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Supported flow kinds across Greentic packs.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub enum FlowKind {
    /// Inbound messaging flows (Telegram, Teams, HTTP chat).
    Messaging,
    /// Event-driven flows (webhooks, NATS, cron, etc.).
    Event,
    /// Flows that configure components/providers/infrastructure.
    ComponentConfig,
    /// Batch/background jobs.
    Job,
    /// HTTP-style request/response flows.
    Http,
}

/// Canonical flow representation embedded in packs.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(
    feature = "schemars",
    derive(JsonSchema),
    schemars(
        title = "Greentic Flow v1",
        description = "Canonical flow model with components, routing and telemetry.",
        rename = "greentic.flow.v1"
    )
)]
pub struct Flow {
    /// Schema version for the flow document.
    pub schema_version: String,
    /// Flow identifier inside the pack.
    pub id: FlowId,
    /// Flow execution kind.
    pub kind: FlowKind,
    /// Entrypoints for this flow keyed by name (for example `default`, `telegram`, `http:/path`).
    #[cfg_attr(feature = "serde", serde(default))]
    #[cfg_attr(
        feature = "schemars",
        schemars(with = "alloc::collections::BTreeMap<String, Value>")
    )]
    pub entrypoints: BTreeMap<String, Value>,
    /// Ordered node map describing the flow graph.
    #[cfg_attr(feature = "serde", serde(default))]
    #[cfg_attr(
        feature = "schemars",
        schemars(with = "alloc::collections::BTreeMap<NodeId, Node>")
    )]
    pub nodes: IndexMap<NodeId, Node, FlowHasher>,
    /// Optional metadata for authoring/UX.
    #[cfg_attr(feature = "serde", serde(default))]
    pub metadata: FlowMetadata,
}

impl Flow {
    /// Returns `true` when no nodes are defined.
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    /// Returns the implicit ingress node (first user-declared entry).
    pub fn ingress(&self) -> Option<(&NodeId, &Node)> {
        self.nodes.iter().next()
    }
}

/// Flow node representation.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub struct Node {
    /// Node identifier.
    pub id: NodeId,
    /// Component binding referenced by the node.
    pub component: ComponentRef,
    /// Component input mapping configuration.
    #[cfg_attr(feature = "serde", serde(alias = "in_map"))]
    pub input: InputMapping,
    /// Component output mapping configuration.
    #[cfg_attr(feature = "serde", serde(alias = "out_map"))]
    pub output: OutputMapping,
    /// Optional error mapping configuration.
    #[cfg_attr(
        feature = "serde",
        serde(
            default,
            skip_serializing_if = "Option::is_none",
            rename = "err_map",
            alias = "error_output"
        )
    )]
    pub err_map: Option<OutputMapping>,
    /// Routing behaviour after this node.
    pub routing: Routing,
    /// Optional telemetry hints for this node.
    #[cfg_attr(feature = "serde", serde(default))]
    pub telemetry: TelemetryHints,
}

impl Node {
    /// Returns the canonical input mapping surface.
    pub fn in_map(&self) -> &InputMapping {
        &self.input
    }

    /// Returns the canonical output mapping surface.
    pub fn out_map(&self) -> &OutputMapping {
        &self.output
    }

    /// Returns the canonical error mapping surface when configured.
    pub fn err_map(&self) -> Option<&OutputMapping> {
        self.err_map.as_ref()
    }

    /// Whether this node is a runner builtin (engine-handled, resolving to no
    /// pack component): `dw.agent[.<id>]`, `dw.agent_graph[.<id>]`,
    /// `sorla.call`, `operala.call`, `agentic.call`, `session.wait`,
    /// `flow.call`, `provider.invoke`, `emit.*`.
    ///
    /// A `dw.agent.<id>` (and other dispatch) node compiles to the generic
    /// `component.exec` placeholder with the real kind in `operation`, so the
    /// operation is authoritative when the component id is empty/`component.exec`.
    pub fn is_builtin(&self) -> bool {
        let id = self.component.id.as_str();
        let effective = if id.is_empty() || id == "component.exec" {
            self.component.operation.as_deref().unwrap_or(id)
        } else {
            id
        };
        is_builtin_component_id(effective)
    }
}

/// Builtin/dispatch flow-node kinds handled by the runner engine itself, with
/// no pack component to resolve. MUST mirror the runner's `NodeKind` dispatch.
/// Agentic/dispatch kinds appear as `dw.agent.<id>` etc., so callers match both
/// the bare kind and the dotted `<kind>.<suffix>` form (see [`Node::is_builtin`]).
pub const BUILTIN_NODE_KINDS: &[&str] = &[
    "session.wait",
    "flow.call",
    "provider.invoke",
    "dw.agent",
    "dw.agent_graph",
    "sorla.call",
    "operala.call",
    "agentic.call",
];

/// Whether a component-id string names a runner builtin (bare or dotted form;
/// `emit.*` is always builtin).
pub fn is_builtin_component_id(id: &str) -> bool {
    id.starts_with("emit.")
        || BUILTIN_NODE_KINDS.iter().any(|kind| {
            id == *kind
                || id
                    .strip_prefix(kind)
                    .is_some_and(|rest| rest.starts_with('.'))
        })
}

/// Component reference within a flow.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub struct ComponentRef {
    /// Component identifier.
    pub id: ComponentId,
    /// Dependency pack alias when referencing external components.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub pack_alias: Option<String>,
    /// Optional operation name within the component.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub operation: Option<String>,
}

/// Opaque component input mapping configuration.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub struct InputMapping {
    /// Mapping configuration (templates, expressions, etc.).
    #[cfg_attr(feature = "serde", serde(default))]
    pub mapping: Value,
}

/// Opaque component output mapping configuration.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub struct OutputMapping {
    /// Mapping configuration (templates, expressions, etc.).
    #[cfg_attr(feature = "serde", serde(default))]
    pub mapping: Value,
}

/// Optional authoring metadata for flows.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub struct FlowMetadata {
    /// Optional human-friendly title.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub title: Option<String>,
    /// Optional human-friendly description.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub description: Option<String>,
    /// Optional tags.
    #[cfg_attr(feature = "serde", serde(default))]
    pub tags: BTreeSet<String>,
    /// Free-form metadata.
    #[cfg_attr(feature = "serde", serde(default))]
    pub extra: Value,
}

impl Default for FlowMetadata {
    fn default() -> Self {
        Self {
            title: None,
            description: None,
            tags: BTreeSet::new(),
            extra: Value::Null,
        }
    }
}

/// Routing behaviour for a node.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub enum Routing {
    /// Continue to the specified node.
    Next {
        /// Destination node identifier.
        node_id: NodeId,
    },
    /// Branch based on status string -> node id.
    Branch {
        /// Mapping of status value to destination node.
        #[cfg_attr(feature = "serde", serde(default))]
        #[cfg_attr(
            feature = "schemars",
            schemars(with = "alloc::collections::BTreeMap<String, NodeId>")
        )]
        on_status: BTreeMap<String, NodeId>,
        /// Default node when no status matches.
        #[cfg_attr(
            feature = "serde",
            serde(default, skip_serializing_if = "Option::is_none")
        )]
        default: Option<NodeId>,
    },
    /// Flow terminates successfully.
    End,
    /// Reply to origin (Messaging/Http flows).
    Reply,
    /// Component- or runtime-specific routing.
    Custom(Value),
}

/// Optional telemetry hints for a node.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub struct TelemetryHints {
    /// Optional span name.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub span_name: Option<String>,
    /// Attributes to attach to spans/logs.
    #[cfg_attr(feature = "serde", serde(default))]
    pub attributes: BTreeMap<String, String>,
    /// Sampling hint (`high`, `normal`, `low`).
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub sampling: Option<String>,
}

#[cfg(test)]
mod builtin_tests {
    use super::is_builtin_component_id;

    #[test]
    fn builtin_component_ids_cover_runner_node_kinds() {
        for id in [
            "session.wait",
            "flow.call",
            "provider.invoke",
            "emit.response",
            "dw.agent",
            "dw.agent.support",
            "dw.agent_graph",
            "dw.agent_graph.triage",
            "sorla.call",
            "operala.call",
            "agentic.call",
        ] {
            assert!(is_builtin_component_id(id), "{id} should be builtin");
        }
        for id in [
            "qa.process",
            "templating.handlebars",
            "dw.agentish",
            "agentic",
        ] {
            assert!(!is_builtin_component_id(id), "{id} must NOT be builtin");
        }
    }
}
