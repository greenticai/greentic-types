//! Wire contract for dispatching work from a flow to a separate runtime
//! (sorla / operala / agentic) over a pub/sub transport. Shared by the runner
//! and every runtime-side bridge so both ends agree on the message payload.

use alloc::{format, string::String, vec::Vec};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Whether the flow blocks for a response (`Await`) or continues immediately
/// after publishing the request (`FireAndForget`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
pub enum DispatchMode {
    /// Block the flow until the runtime responds (durable pause/resume).
    Await,
    /// Publish the request and continue the flow immediately.
    FireAndForget,
}

/// Payload of a `greentic.<runtime>.request.v1` message.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct RuntimeDispatchRequest {
    /// Logical target inside the runtime (e.g. a sorx deployment id).
    pub target: String,
    /// Operation/endpoint within the runtime.
    pub operation: String,
    /// Await vs fire-and-forget dispatch semantics.
    pub mode: DispatchMode,
    /// Opaque runtime input.
    pub input: Value,
    /// Await-mode timeout budget in milliseconds.
    pub deadline_ms: Option<u64>,
}

/// Payload of a `greentic.<runtime>.response.v1` message.
/// The transport correlation id MUST echo the request's.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct RuntimeDispatchResponse {
    /// Whether the runtime handled the request successfully.
    pub ok: bool,
    /// Runtime output payload (meaningful when `ok`).
    pub output: Value,
    /// Optional runtime-emitted events.
    #[cfg_attr(feature = "serde", serde(default))]
    pub events: Vec<Value>,
    /// Error details when `ok` is false.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub error: Option<DispatchError>,
}

/// Structured error returned by a runtime when a dispatch fails.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct DispatchError {
    /// Stable machine-readable error code.
    pub code: String,
    /// Human-readable error message.
    pub message: String,
}

/// Request topic/subject for a runtime, e.g. `greentic.sorla.request.v1`.
pub fn request_topic(runtime: &str) -> String {
    format!("greentic.{runtime}.request.v1")
}

/// Response topic/subject for a runtime, e.g. `greentic.sorla.response.v1`.
pub fn response_topic(runtime: &str) -> String {
    format!("greentic.{runtime}.response.v1")
}

#[cfg(all(test, feature = "serde"))]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn request_round_trips_through_json() {
        let req = RuntimeDispatchRequest {
            target: "dep-123".into(),
            operation: "create_invoice".into(),
            mode: DispatchMode::Await,
            input: json!({ "amount": 10 }),
            deadline_ms: Some(30_000),
        };
        let encoded = serde_json::to_value(&req).expect("serialize dispatch request");
        let decoded: RuntimeDispatchRequest =
            serde_json::from_value(encoded).expect("deserialize dispatch request");
        assert_eq!(decoded, req);
    }

    #[test]
    fn topic_helpers_use_runtime_name() {
        assert_eq!(request_topic("sorla"), "greentic.sorla.request.v1");
        assert_eq!(response_topic("sorla"), "greentic.sorla.response.v1");
    }
}
