# Host deployment requirements (HOME-03)

`deployment::host::HostRequirements` describes Linux x86_64 or aarch64 host
requirements without granting access. `HostDeploymentPlan` wraps the existing
`DeploymentPlan`; existing Rust struct literals and serialized deployment plans
remain unchanged. Consumers opt into the wrapper. Its requirements use explicit
wire version `1.0.0`; unknown versions, fields, privileges and enum values are
rejected. JSON Schema export includes both new types.

The contract covers network mode, exact character-device and Unix-socket
mappings, persistent mounts with expected owner UID/GID, published ports and a
bounded local HTTP readiness probe. It has no privileged-container, arbitrary
health-command, device-node-creation or unbounded capability field. Each mapping
list is limited to 32 entries by `validate()`.

Consumers must call `validate()` on constructed and deserialized requirements.
It rejects root/relative/traversing paths, wildcard/control/shell-expansion
characters, overlapping workload targets, mounts of sensitive system trees,
contradictory network/port requirements and unbounded readiness waits. Exact
paths intentionally use a restricted ASCII alphabet; no shell interpolation is
needed. A path passing this check is not proof that it exists or is safe to use.

The deployer must additionally:

1. Check the same typed requirements at preview, plan and apply boundaries.
2. Compare requests against separate operator-owned policy. A pack must never
   provide its own access grants. Host networking, public listeners, devices,
   sockets and mount roots all require explicit grants.
3. Probe the actual OS/architecture and target image architecture. Check device
   kind, resolved path and permissions; check sockets and mount ownership.
   Refuse symlink races and missing prerequisites before workload launch.
4. Render only the validated bounded mappings. Run containers without privileged
   mode, device creation or implicit extra capabilities. A mounted D-Bus socket
   still requires an independently bounded D-Bus service policy.
5. Enforce readiness and rollback failed application without deleting persistent
   application data.

This type-only change does not make any existing deployer support ARM64 or host
access. It supplies no native probe, systemd unit, deployment engine or Pi-specific
code. Deployer integration and real Linux/ARM64 qualification remain separate
HOME-03 work. Downstream code must consume a published compatible release before
claiming completion; no local path dependency is a released interface.
