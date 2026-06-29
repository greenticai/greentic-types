//! Stable flow resolve summary types and helpers.
//!
//! # JSON shape (v1)
//! ```json
//! {
//!   "schema_version": 1,
//!   "flow": "main.ygtc",
//!   "nodes": {
//!     "fetch": {
//!       "component_id": "greentic.demo.component",
//!       "source": {
//!         "kind": "oci",
//!         "ref": "ghcr.io/greentic/demo/component:1.2.3"
//!       },
//!       "digest": "sha256:deadbeef",
//!       "manifest": {
//!         "world": "greentic:component/world",
//!         "version": "1.2.3"
//!       }
//!     }
//!   }
//! }
//! ```

use alloc::collections::BTreeMap;
use alloc::format;
use alloc::string::String;

use semver::Version;

#[cfg(feature = "schemars")]
use schemars::JsonSchema;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::{ComponentId, ErrorCode, GResult, GreenticError};

/// Current schema version for flow resolve summaries.
pub const FLOW_RESOLVE_SUMMARY_SCHEMA_VERSION: u32 = 1;

/// Flow resolve summary (v1).
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(
    feature = "schemars",
    derive(JsonSchema),
    schemars(
        title = "Greentic Flow Resolve Summary v1",
        description = "Stable component resolution summary for flow nodes.",
        rename = "greentic.flow.resolve-summary.v1"
    )
)]
pub struct FlowResolveSummaryV1 {
    /// Schema version (must be 1).
    pub schema_version: u32,
    /// Flow file basename (for example `main.ygtc`).
    pub flow: String,
    /// Resolve summary keyed by node name.
    pub nodes: BTreeMap<String, NodeResolveSummaryV1>,
}

/// Resolve summary for a flow node.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub struct NodeResolveSummaryV1 {
    /// Component identifier referenced by this node.
    pub component_id: ComponentId,
    /// Component source reference.
    pub source: FlowResolveSummarySourceRefV1,
    /// Pinned digest for the resolved artifact.
    pub digest: String,
    /// Optional manifest metadata captured at resolve time.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub manifest: Option<FlowResolveSummaryManifestV1>,
}

/// Minimal manifest metadata included in the summary.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub struct FlowResolveSummaryManifestV1 {
    /// Referenced WIT world binding.
    pub world: String,
    /// Semantic component version.
    #[cfg_attr(
        feature = "schemars",
        schemars(with = "String", description = "SemVer version")
    )]
    pub version: Version,
}

/// Component source references for flow resolve summaries.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(tag = "kind", rename_all = "snake_case"))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub enum FlowResolveSummarySourceRefV1 {
    /// Local wasm path relative to the flow file.
    Local {
        /// Relative path to the wasm artifact.
        path: String,
    },
    /// OCI component reference.
    Oci {
        /// OCI reference.
        r#ref: String,
    },
    /// Repository component reference.
    Repo {
        /// Repository reference.
        r#ref: String,
    },
    /// Store component reference.
    Store {
        /// Store reference.
        r#ref: String,
    },
    /// Extension-sourced component reference (`ext://<id>#component`).
    Ext {
        /// Extension component reference.
        r#ref: String,
    },
}

#[cfg(feature = "std")]
use std::ffi::OsString;
#[cfg(feature = "std")]
use std::fs;
#[cfg(feature = "std")]
use std::path::{Path, PathBuf};

/// Returns the expected summary sidecar path for a flow file.
#[cfg(feature = "std")]
pub fn resolve_summary_path_for_flow(flow_path: &Path) -> PathBuf {
    let file_name = flow_path
        .file_name()
        .map(OsString::from)
        .unwrap_or_else(|| OsString::from("flow.ygtc"));
    let mut sidecar_name = file_name;
    sidecar_name.push(".resolve.summary.json");
    flow_path.with_file_name(sidecar_name)
}

/// Reads a flow resolve summary from disk and validates it.
#[cfg(all(feature = "std", feature = "serde"))]
pub fn read_flow_resolve_summary(path: &Path) -> GResult<FlowResolveSummaryV1> {
    let raw = fs::read_to_string(path).map_err(|err| io_error("read flow resolve summary", err))?;
    let doc: FlowResolveSummaryV1 =
        serde_json::from_str(&raw).map_err(|err| json_error("parse flow resolve summary", err))?;
    validate_flow_resolve_summary(&doc)?;
    Ok(doc)
}

/// Writes a flow resolve summary to disk after validation.
#[cfg(all(feature = "std", feature = "serde"))]
pub fn write_flow_resolve_summary(path: &Path, doc: &FlowResolveSummaryV1) -> GResult<()> {
    validate_flow_resolve_summary(doc)?;
    let raw = serde_json::to_string_pretty(doc)
        .map_err(|err| json_error("serialize flow resolve summary", err))?;
    fs::write(path, raw).map_err(|err| io_error("write flow resolve summary", err))?;
    Ok(())
}

/// Validates a flow resolve summary document.
#[cfg(feature = "std")]
pub fn validate_flow_resolve_summary(doc: &FlowResolveSummaryV1) -> GResult<()> {
    if doc.schema_version != FLOW_RESOLVE_SUMMARY_SCHEMA_VERSION {
        return Err(GreenticError::new(
            ErrorCode::InvalidInput,
            format!(
                "flow resolve summary schema_version must be {}",
                FLOW_RESOLVE_SUMMARY_SCHEMA_VERSION
            ),
        ));
    }

    for (node_name, node) in &doc.nodes {
        if let FlowResolveSummarySourceRefV1::Local { path } = &node.source
            && Path::new(path).is_absolute()
        {
            return Err(GreenticError::new(
                ErrorCode::InvalidInput,
                format!(
                    "local component path for node '{}' must be relative",
                    node_name
                ),
            ));
        }
        validate_digest(&node.digest)?;
        if let Some(metadata) = &node.manifest
            && metadata.world.trim().is_empty()
        {
            return Err(GreenticError::new(
                ErrorCode::InvalidInput,
                format!("manifest world for node '{}' must not be empty", node_name),
            ));
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
