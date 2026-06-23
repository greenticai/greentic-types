//! Contract tests for the uniform node I/O types (spec §4.2–§4.4).
//! These pin the wire shapes the runner/flow/designer will standardize on.
#![cfg(feature = "serde")]

use greentic_types::node_io::{ErrorKind, InputValue, NodeError, NodeOutput, StructuredInput};
use serde_json::json;

#[test]
fn node_output_success_serializes_as_data() {
    let out = NodeOutput::ok(json!({ "ticket_id": "T-1" }));
    assert!(out.is_ok());
    assert_eq!(
        serde_json::to_value(&out).unwrap(),
        json!({ "data": { "ticket_id": "T-1" } })
    );
    // round-trip
    let back: NodeOutput = serde_json::from_value(json!({ "data": { "x": 1 } })).unwrap();
    assert_eq!(back.data(), Some(&json!({ "x": 1 })));
    assert!(back.errors().is_empty());
}

#[test]
fn node_output_failure_serializes_as_errors_and_is_mutually_exclusive() {
    let err = NodeError {
        code: "PROVIDER_TIMEOUT".into(),
        message: "upstream timed out".into(),
        kind: ErrorKind::Timeout,
        retryable: true,
        source: None,
        details: json!({}),
    };
    let out = NodeOutput::failed(vec![err]);
    assert!(!out.is_ok());
    assert!(out.data().is_none());
    let v = serde_json::to_value(&out).unwrap();
    assert_eq!(v["errors"][0]["code"], json!("PROVIDER_TIMEOUT"));
    assert_eq!(v["errors"][0]["kind"], json!("timeout"));
    assert_eq!(v["errors"][0]["retryable"], json!(true));
    // a success output must NOT carry an `errors` key, and vice-versa
    assert!(
        serde_json::to_value(NodeOutput::ok(json!({})))
            .unwrap()
            .get("errors")
            .is_none()
    );
    assert!(v.get("data").is_none());
}

#[test]
fn error_kind_round_trips_snake_case() {
    for (k, s) in [
        (ErrorKind::Validation, "validation"),
        (ErrorKind::Timeout, "timeout"),
        (ErrorKind::Auth, "auth"),
        (ErrorKind::NotFound, "not_found"),
        (ErrorKind::RateLimited, "rate_limited"),
        (ErrorKind::Internal, "internal"),
        (ErrorKind::Cancelled, "cancelled"),
    ] {
        assert_eq!(serde_json::to_value(k).unwrap(), json!(s));
        let back: ErrorKind = serde_json::from_value(json!(s)).unwrap();
        assert_eq!(back, k);
    }
}

#[test]
fn input_value_is_structurally_tagged_literal_ref_template() {
    // Each variant serializes to its own single-key object — no delimiter sniffing.
    assert_eq!(
        serde_json::to_value(InputValue::Literal(json!(60))).unwrap(),
        json!({ "literal": 60 })
    );
    assert_eq!(
        serde_json::to_value(InputValue::Ref("classify.data.ticket_id".into())).unwrap(),
        json!({ "ref": "classify.data.ticket_id" })
    );
    assert_eq!(
        serde_json::to_value(InputValue::Template("Hi {{ enrich.data.name }}".into())).unwrap(),
        json!({ "template": "Hi {{ enrich.data.name }}" })
    );
    let si: StructuredInput =
        serde_json::from_value(json!({ "name": "threshold", "value": { "literal": 60 } })).unwrap();
    assert_eq!(si.name, "threshold");
    assert_eq!(si.value, InputValue::Literal(json!(60)));
}
