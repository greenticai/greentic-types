//! Canonical flow resolve sidecar types and helpers.
//!
//! # JSON shape (v1)
//! ```json
//! {
//!   "schema_version": 1,
//!   "flow": "main.ygtc",
//!   "nodes": {
//!     "fetch": {
//!       "source": {
//!         "kind": "oci",
//!         "ref": "ghcr.io/greentic/demo/component:1.2.3",
//!         "digest": "sha256:deadbeef"
//!       },
//!       "mode": "pinned"
//!     }
//!   }
//! }
//! ```

use alloc::collections::BTreeMap;
use alloc::format;
use alloc::string::String;

#[cfg(feature = "schemars")]
use schemars::JsonSchema;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::{ErrorCode, GResult, GreenticError};

/// Current schema version for flow resolve sidecars.
pub const FLOW_RESOLVE_SCHEMA_VERSION: u32 = 1;

/// Flow resolve sidecar (v1).
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(
    feature = "schemars",
    derive(JsonSchema),
    schemars(
        title = "Greentic Flow Resolve v1",
        description = "Component source references for flow nodes.",
        rename = "greentic.flow.resolve.v1"
    )
)]
pub struct FlowResolveV1 {
    /// Schema version (must be 1).
    pub schema_version: u32,
    /// Flow file basename (for example `main.ygtc`).
    pub flow: String,
    /// Resolve metadata keyed by node name.
    pub nodes: BTreeMap<String, NodeResolveV1>,
}

/// Resolve metadata for a flow node.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub struct NodeResolveV1 {
    /// Component source reference.
    pub source: ComponentSourceRefV1,
    /// Optional resolve mode hint.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub mode: Option<ResolveModeV1>,
}

/// Resolve mode hints for sidecars.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub enum ResolveModeV1 {
    /// Follows moving references.
    Tracked,
    /// Uses pinned digest only.
    Pinned,
}

/// Component source references for flow resolve sidecars.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(tag = "kind", rename_all = "snake_case"))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub enum ComponentSourceRefV1 {
    /// Local wasm path relative to the flow file.
    Local {
        /// Relative path to the wasm artifact.
        path: String,
        /// Optional pinned digest.
        #[cfg_attr(
            feature = "serde",
            serde(default, skip_serializing_if = "Option::is_none")
        )]
        digest: Option<String>,
    },
    /// OCI component reference.
    Oci {
        /// OCI reference.
        r#ref: String,
        /// Optional pinned digest.
        #[cfg_attr(
            feature = "serde",
            serde(default, skip_serializing_if = "Option::is_none")
        )]
        digest: Option<String>,
    },
    /// Repository component reference.
    Repo {
        /// Repository reference.
        r#ref: String,
        /// Optional pinned digest.
        #[cfg_attr(
            feature = "serde",
            serde(default, skip_serializing_if = "Option::is_none")
        )]
        digest: Option<String>,
    },
    /// Store component reference.
    Store {
        /// Store reference.
        r#ref: String,
        /// Optional pinned digest.
        #[cfg_attr(
            feature = "serde",
            serde(default, skip_serializing_if = "Option::is_none")
        )]
        digest: Option<String>,
        /// Optional license hint for store resolution.
        #[cfg_attr(
            feature = "serde",
            serde(default, skip_serializing_if = "Option::is_none")
        )]
        license_hint: Option<String>,
        /// Optional meter flag for usage tracking.
        #[cfg_attr(
            feature = "serde",
            serde(default, skip_serializing_if = "Option::is_none")
        )]
        meter: Option<bool>,
    },
    /// Extension-sourced component reference: the runtime component embedded in
    /// an extension's `.gtxpack` (resolved by packc to a `Local` embed at build).
    /// Ref form: `ext://<extension-id>#component`.
    Ext {
        /// Extension component reference (`ext://<id>#component`).
        r#ref: String,
        /// Optional pinned digest of the embedded component wasm.
        #[cfg_attr(
            feature = "serde",
            serde(default, skip_serializing_if = "Option::is_none")
        )]
        digest: Option<String>,
    },
}

#[cfg(feature = "std")]
use std::ffi::OsString;
#[cfg(feature = "std")]
use std::fs;
#[cfg(feature = "std")]
use std::path::{Path, PathBuf};

/// Returns the expected sidecar path for a flow file.
#[cfg(feature = "std")]
pub fn sidecar_path_for_flow(flow_path: &Path) -> PathBuf {
    let file_name = flow_path
        .file_name()
        .map(OsString::from)
        .unwrap_or_else(|| OsString::from("flow.ygtc"));
    let mut sidecar_name = file_name;
    sidecar_name.push(".resolve.json");
    flow_path.with_file_name(sidecar_name)
}

/// Reads a flow resolve sidecar from disk and validates it.
#[cfg(all(feature = "std", feature = "serde"))]
pub fn read_flow_resolve(path: &Path) -> GResult<FlowResolveV1> {
    let raw = fs::read_to_string(path).map_err(|err| io_error("read flow resolve", err))?;
    let doc: FlowResolveV1 =
        serde_json::from_str(&raw).map_err(|err| json_error("parse flow resolve", err))?;
    validate_flow_resolve(&doc)?;
    Ok(doc)
}

/// Writes a flow resolve sidecar to disk after validation.
#[cfg(all(feature = "std", feature = "serde"))]
pub fn write_flow_resolve(path: &Path, doc: &FlowResolveV1) -> GResult<()> {
    validate_flow_resolve(doc)?;
    let raw = serde_json::to_string_pretty(doc)
        .map_err(|err| json_error("serialize flow resolve", err))?;
    fs::write(path, raw).map_err(|err| io_error("write flow resolve", err))?;
    Ok(())
}

/// Validates a flow resolve document.
#[cfg(feature = "std")]
pub fn validate_flow_resolve(doc: &FlowResolveV1) -> GResult<()> {
    if doc.schema_version != FLOW_RESOLVE_SCHEMA_VERSION {
        return Err(GreenticError::new(
            ErrorCode::InvalidInput,
            format!(
                "flow resolve schema_version must be {}",
                FLOW_RESOLVE_SCHEMA_VERSION
            ),
        ));
    }

    for (node_name, node) in &doc.nodes {
        match &node.source {
            ComponentSourceRefV1::Local { path, digest } => {
                if Path::new(path).is_absolute() {
                    return Err(GreenticError::new(
                        ErrorCode::InvalidInput,
                        format!(
                            "local component path for node '{}' must be relative",
                            node_name
                        ),
                    ));
                }
                if let Some(value) = digest {
                    validate_digest(value)?;
                }
            }
            ComponentSourceRefV1::Oci { digest, .. }
            | ComponentSourceRefV1::Repo { digest, .. }
            | ComponentSourceRefV1::Store { digest, .. }
            | ComponentSourceRefV1::Ext { digest, .. } => {
                if let Some(value) = digest {
                    validate_digest(value)?;
                }
            }
        }
    }

    Ok(())
}

#[cfg(feature = "std")]
fn validate_digest(digest: &str) -> GResult<()> {
    let hex = digest.strip_prefix("sha256:").ok_or_else(|| {
        GreenticError::new(ErrorCode::InvalidInput, "digest must match sha256:<hex>")
    })?;
    if hex.is_empty() || !hex.chars().all(|ch| ch.is_ascii_hexdigit()) {
        return Err(GreenticError::new(
            ErrorCode::InvalidInput,
            "digest must match sha256:<hex>",
        ));
    }
    Ok(())
}

#[cfg(all(feature = "std", feature = "serde"))]
fn json_error(context: &str, err: serde_json::Error) -> GreenticError {
    GreenticError::new(ErrorCode::InvalidInput, format!("{context}: {err}")).with_source(err)
}

#[cfg(feature = "std")]
fn io_error(context: &str, err: std::io::Error) -> GreenticError {
    let code = match err.kind() {
        std::io::ErrorKind::NotFound => ErrorCode::NotFound,
        std::io::ErrorKind::PermissionDenied => ErrorCode::PermissionDenied,
        _ => ErrorCode::Unavailable,
    };
    GreenticError::new(code, format!("{context}: {err}")).with_source(err)
}
