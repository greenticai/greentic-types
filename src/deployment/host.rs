//! Versioned host requirements. These are requests, never grants of host access.
//!
//! A deployer must validate these requirements against operator-owned policy and
//! freshly probed host facts before launching a workload. Packs cannot supply
//! that policy. This module deliberately performs no filesystem or network I/O.

use alloc::{collections::BTreeSet, string::String, vec::Vec};

#[cfg(feature = "schemars")]
use schemars::JsonSchema;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use super::DeploymentPlan;

/// Typed host deployment envelope, preserving the original plan's wire format.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(deny_unknown_fields)
)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub struct HostDeploymentPlan {
    /// Versioned requirements for the target host.
    pub host: HostRequirements,
    /// Existing provider-agnostic deployment plan.
    pub deployment: DeploymentPlan,
}

/// Supported wire version. Unknown versions fail deserialization.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub enum HostRequirementsVersion {
    /// Initial host requirements contract.
    #[cfg_attr(feature = "serde", serde(rename = "1.0.0"))]
    V1,
}

/// Supported operating systems; adding one requires deployer support.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(rename_all = "snake_case")
)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub enum HostOperatingSystem {
    /// Linux container host.
    Linux,
}

/// Native workload architecture.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(rename_all = "snake_case")
)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub enum HostArchitecture {
    /// 64-bit x86.
    X86_64,
    /// 64-bit ARM.
    Aarch64,
}

/// Explicit network isolation requirement.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(rename_all = "snake_case")
)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub enum HostNetworkMode {
    /// No network interface beyond loopback.
    None,
    /// Isolated container network with explicit published ports.
    Bridge,
    /// Host namespace; requires explicit operator permission.
    Host,
}

/// Maximum access required for a mapped character device.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(rename_all = "snake_case")
)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub enum HostDeviceAccess {
    /// Read-only device access.
    Read,
    /// Read/write access; never grants device-node creation.
    ReadWrite,
}

/// One bounded device mapping; wildcards and whole-device-directory maps fail.
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(deny_unknown_fields)
)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub struct HostDeviceMapping {
    /// Exact host path, resolved and verified by the deployer.
    pub source: String,
    /// Exact workload path.
    pub target: String,
    /// Required access mask.
    pub access: HostDeviceAccess,
}

/// One Unix socket mapping, requiring operator policy and native socket checks.
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(deny_unknown_fields)
)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub struct HostSocketMapping {
    /// Exact socket path on the host.
    pub source: String,
    /// Exact socket path inside the workload.
    pub target: String,
}

/// Persistent storage with an explicit expected host owner.
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(deny_unknown_fields)
)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub struct HostPersistentMount {
    /// Existing host directory; the deployer verifies ownership and path safety.
    pub source: String,
    /// Workload directory.
    pub target: String,
    /// Whether the workload may write the mounted directory.
    pub read_only: bool,
    /// Required numerical host owner.
    pub owner_uid: u32,
    /// Required numerical host group.
    pub owner_gid: u32,
}

/// Supported port transports.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(rename_all = "snake_case")
)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub enum HostPortProtocol {
    /// TCP listener.
    Tcp,
    /// UDP listener.
    Udp,
}

/// Explicit host binding scope.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(rename_all = "snake_case")
)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub enum HostPortBind {
    /// Host loopback only.
    Loopback,
    /// All host interfaces; requires explicit operator permission.
    AllInterfaces,
}

/// One published port. Host-network mode uses equal host/workload ports.
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(deny_unknown_fields)
)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub struct HostPort {
    /// Host-side listening port.
    pub host_port: u16,
    /// Workload-side listening port.
    pub workload_port: u16,
    /// Network transport.
    pub protocol: HostPortProtocol,
    /// Required binding scope.
    pub bind: HostPortBind,
}

/// Bounded local HTTP readiness check; no arbitrary health command execution.
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(deny_unknown_fields)
)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub struct HostHealthRequirement {
    /// Workload HTTP port, queried over loopback in its network namespace.
    pub port: u16,
    /// Absolute HTTP path without query, fragment or authority.
    pub path: String,
    /// Maximum startup wait, in seconds.
    pub startup_timeout_seconds: u16,
}

/// Explicit, bounded workload host requirements, independent of any provider.
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(deny_unknown_fields)
)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub struct HostRequirements {
    /// Wire version.
    pub schema_version: HostRequirementsVersion,
    /// Target operating system.
    pub os: HostOperatingSystem,
    /// Target native architecture.
    pub architecture: HostArchitecture,
    /// Required network isolation.
    pub network: HostNetworkMode,
    /// Exact device requests; no privileges are implied by this list.
    pub devices: Vec<HostDeviceMapping>,
    /// Exact socket requests, for example an operator-approved D-Bus socket.
    pub sockets: Vec<HostSocketMapping>,
    /// Persistent mount requests.
    pub mounts: Vec<HostPersistentMount>,
    /// Declared listening ports.
    pub ports: Vec<HostPort>,
    /// Required readiness probe.
    pub health: HostHealthRequirement,
}

/// Stable validation categories; deployers may attach field-specific diagnostics.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HostRequirementError {
    /// More than 32 mappings of one kind were requested.
    TooManyMappings,
    /// A path is not exact, absolute, normalized or bounded.
    InvalidPath,
    /// Device mapping did not name individual device paths.
    InvalidDevice,
    /// Multiple mappings target the same workload path.
    DuplicateTarget,
    /// Invalid, duplicate or contradictory listening ports.
    InvalidPort,
    /// Readiness path, port or time budget is invalid.
    InvalidHealth,
}

impl HostRequirements {
    /// Check portable structural constraints; never grants access to a host.
    pub fn validate(&self) -> Result<(), HostRequirementError> {
        if [
            self.devices.len(),
            self.sockets.len(),
            self.mounts.len(),
            self.ports.len(),
        ]
        .into_iter()
        .any(|len| len > 32)
        {
            return Err(HostRequirementError::TooManyMappings);
        }
        let mut targets = BTreeSet::new();
        for (source, target) in self
            .devices
            .iter()
            .map(|m| (&m.source, &m.target))
            .chain(self.sockets.iter().map(|m| (&m.source, &m.target)))
            .chain(self.mounts.iter().map(|m| (&m.source, &m.target)))
        {
            if !safe_absolute_path(source) || !safe_absolute_path(target) {
                return Err(HostRequirementError::InvalidPath);
            }
            if !targets.insert(target) {
                return Err(HostRequirementError::DuplicateTarget);
            }
        }
        for device in &self.devices {
            if !device.source.starts_with("/dev/") || !device.target.starts_with("/dev/") {
                return Err(HostRequirementError::InvalidDevice);
            }
        }
        for mount in &self.mounts {
            for path in [&mount.source, &mount.target] {
                if ["/dev", "/proc", "/sys", "/etc", "/run"]
                    .into_iter()
                    .any(|prefix| {
                        path == prefix
                            || path
                                .strip_prefix(prefix)
                                .is_some_and(|rest| rest.starts_with('/'))
                    })
                {
                    return Err(HostRequirementError::InvalidPath);
                }
            }
        }
        for parent in &targets {
            if targets.iter().any(|child| {
                child
                    .strip_prefix(parent.as_str())
                    .is_some_and(|rest| rest.starts_with('/'))
            }) {
                return Err(HostRequirementError::DuplicateTarget);
            }
        }
        let mut ports = BTreeSet::new();
        for port in &self.ports {
            if port.host_port == 0
                || port.workload_port == 0
                || self.network == HostNetworkMode::None
                || (self.network == HostNetworkMode::Host && port.host_port != port.workload_port)
                || !ports.insert((port.protocol, port.host_port))
            {
                return Err(HostRequirementError::InvalidPort);
            }
        }
        if self.health.port == 0
            || !(self.health.path == "/" || safe_absolute_path(&self.health.path))
            || self.health.startup_timeout_seconds == 0
            || self.health.startup_timeout_seconds > 300
        {
            return Err(HostRequirementError::InvalidHealth);
        }
        Ok(())
    }
}

fn safe_absolute_path(path: &str) -> bool {
    path.len() > 1
        && path.len() <= 4096
        && path.starts_with('/')
        && path
            .bytes()
            .all(|c| c.is_ascii_alphanumeric() || b"/._-".contains(&c))
        && path[1..]
            .split('/')
            .all(|part| !part.is_empty() && part != "." && part != "..")
}
