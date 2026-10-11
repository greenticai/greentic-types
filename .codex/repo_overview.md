# Repository Overview

## Active HOME-03 inspection (2026-10-11)

Baseline `bfd273c55c405c88a28c485e8afb727ab6934d51` uses package version
1.2.0. `src/deployment.rs` has a generic DeploymentPlan and opaque runner
capabilities, but no typed host architecture, network mode, device/socket
mapping or persistent-mount requirements. HOME-03 adds a separate versioned
HostDeploymentPlan wrapper and validated host requirements while preserving the
existing DeploymentPlan Rust fields and wire format. Enforcement, native host
probing and apply/rollback remain deployer responsibilities. The referenced
`.codex/repo_overview_task.md` is absent; this refresh uses the actual source.
The draft now implements typed Linux/x86_64/aarch64 requirements, bounded exact
device/socket/mount mappings, port/network consistency, readiness bounds and
JSON Schema exports. Six targeted tests pass, including legacy DeploymentPlan
round-trip compatibility. The full strict owner check passed formatting, Clippy,
build, tests, schema export, duplicate guards, online schema checks and both
Rust 1.95 MSRV configurations. Use `GITHUB_REPOSITORY_OWNER=greenticai`, matching
hosted CI: the legacy organisation schema URLs return 404 when the owner is
omitted locally. No native host enforcement or Pi qualification is claimed by
this type-only change.

## 1. High-Level Purpose
- Shared Rust crate supplying Greentic’s multi-tenant primitives: tenant/session identity, flow/component/pack manifests, execution envelopes, supply-chain and store models, and telemetry helpers used across the greentic-ng stack (runner, deployer, connectors, packs, state/session services).
- Provides serde/schemars-friendly types, schema export tooling, and a proc-macro for auto-initialising telemetry, keeping downstream repos aligned on identifiers, manifests, and envelopes.

## 2. Main Components and Functionality
- **Path:** `src/lib.rs`  
  **Role:** Crate root re-exporting modules, constants, and helpers.  
  **Key functionality:** Defines identifier newtypes with validation, schema IDs, invocation envelopes, `NodeError`/`NodeResult`, idempotency key generation, `write_all_schemas` for schema export (feature `schema` + `std`), and exposes module exports.  
  **Key dependencies / integration points:** Uses `semver`, optional `schemars`/`serde`/`time`; schema export invoked by `bin/export-schemas.rs`.
- **Path:** `src/tenant.rs`, `src/context.rs`, `src/session.rs`  
  **Role:** Tenant/session identity and context helpers.  
  **Key functionality:** `TenantCtx` builder for tenant/team/user/session/flow/node metadata with impersonation and deadlines; `TenantIdentity`/`TenantContext` derivations; `DeploymentCtx` describing cloud/platform/runtime; session keys, canonical session key builder, and session cursor/data models.
- **Path:** `src/tenant_config.rs`  
  **Role:** Tenant-facing UI configuration documents.  
  **Key functionality:** Skin/theme/layout/auth/DID-aligned JSON shapes for tenant branding, login/console visuals, and related metadata served to UI clients.
- **Path:** `src/capabilities.rs`, `src/policy.rs`  
  **Role:** Capability and policy declarations for packs/runtimes.  
  **Key functionality:** Capability toggles for HTTP, secrets, KV, FS, networking, tools plus `Limits` and `TelemetrySpec`; network `AllowList`/`NetworkPolicy` and `PolicyDecision` with legacy compatibility fields.
- **Path:** `src/component.rs`  
  **Role:** Component manifest and capability requirements.  
  **Key functionality:** `ComponentManifest` with flow support checks, profile selection/fallback, configurator references; Wasi/host capability structs (secrets, state, messaging/events/http, telemetry scope, IaC permissions); profile error types.
- **Path:** `src/component_source.rs`  
  **Role:** Canonical component source references.  
  **Key functionality:** `ComponentSourceRef` parsing/validation for oci/repo/store/file references, normalization helpers, and error types.
- **Path:** `src/flow.rs`  
  **Role:** Flow graph representation used in packs.  
  **Key functionality:** `Flow` with ordered nodes (Fnv hasher), ingress helper, structure/component validation against manifests, `FlowKind` variants (messaging/events), and node metadata (kind/profile/component/config/routing).
- **Path:** `src/flow_resolve.rs`, `src/flow_resolve_summary.rs`  
  **Role:** Flow resolve sidecars and summary payloads.  
  **Key functionality:** Versioned JSON shapes for component resolution metadata, resolve modes, and pinned digest summaries for each flow node.
- **Path:** `src/pack_manifest.rs`, `src/pack.rs`  
  **Role:** Pack manifests and references.  
  **Key functionality:** `PackManifest` (.gtpack) with flows/components, optional profiles/connectors/component_sources and `PackKind`; `PackRef`/`Signature` models for OCI-hosted packs; extension helpers under `src/pack/extensions/` for component source and per-component manifest indexes.
- **Path:** `src/cbor.rs`  
  **Role:** Canonical CBOR encoding for pack manifests.  
  **Key functionality:** Encode/decode `PackManifest` into CBOR with symbol tables and validation errors.
- **Path:** `src/validate.rs`  
  **Role:** Pack validation diagnostics.  
  **Key functionality:** `ValidationReport`/`Diagnostic` types with severity counts and extension-aware pack validation metadata.
- **Path:** `src/deployment.rs`  
  **Role:** Provider-agnostic deployment planning shapes.  
  **Key functionality:** `DeploymentPlan` capturing pack/version, tenant/env, runner sizing, messaging subjects, channels, secrets, OAuth clients, telemetry hints, and extensible `extra` metadata.
- **Path:** `src/run.rs`, `src/outcome.rs`, `src/error.rs`, `src/state.rs`  
  **Role:** Execution outcome, error, and state primitives.  
  **Key functionality:** Run/node status enums, summaries, failures, `RunResult` with duration helper; generic `Outcome<T>` with convenience helpers; `GreenticError`/`ErrorCode` with conversions; state keys and JSON pointer helpers via `StatePath`.
- **Path:** `src/secrets.rs`  
  **Role:** Canonical secret identifiers and scope.  
  **Key functionality:** `SecretKey` validation, `SecretScope`, and requirement types shared across bindings and manifests.
- **Path:** `src/bindings.rs`  
  **Role:** Resource binding hints for runtimes.  
  **Key functionality:** Network allowlists, secrets/env passthrough, and MCP server hints consumed by host runtimes and tooling.
- **Path:** `src/messaging.rs`, `src/events.rs`, `src/events_provider.rs`, `src/worker.rs`, `src/distributor.rs`  
  **Role:** Messaging, event, and worker envelopes.  
  **Key functionality:** Channel message envelopes with attachments/metadata; event envelopes with validated `EventId`, timestamps, payload and metadata; event provider descriptors (kind/transport/reliability/ordering/tags); worker request/response/message payload shapes; distributor API DTOs (env IDs, digests, statuses, artifact locations, signature/cache info, resolve request/response) aligned with `greentic:distributor-api@1.0.0`.
- **Path:** `src/provider.rs`, `src/provider_install.rs`  
  **Role:** Provider declarations and installation records.  
  **Key functionality:** Provider manifests/decls/runtime refs plus shared install records for provisioning outputs (config/secret refs, webhook/subscription state, metadata).
- **Path:** `src/store.rs`  
  **Role:** Storefront, catalog, subscription, and desired state models.  
  **Key functionality:** Themes/layout sections, storefronts/collections/product overrides; products/plans/subscriptions with pricing/versioning; bundle/spec/export descriptors for desired state (BundleId/BundleSpec now described as ids for distribution-bundle `.gtpack` outputs); uses BTreeMap metadata and pack/component references.
- **Path:** `src/supply_chain.rs`  
  **Role:** Supply-chain build/scan/signing/verification models.  
  **Key functionality:** Build plans/status with outputs/log refs; scan requests/results; attestation/signing/verification records; repo/store contexts; uses IndexMap with FNV hasher for no_std friendliness.
- **Path:** `src/telemetry`, `greentic-types-macros/src/lib.rs`  
  **Role:** Telemetry context helpers and proc-macro.  
  **Key functionality:** `SpanContext` for OTLP-aligned spans; optional `OtlpKeys` constants; telemetry auto-init helpers (`install_telemetry`, `set_current_tenant_ctx`) behind `telemetry-autoinit`; `#[greentic_types::telemetry::main]` proc-macro wraps async main with tokio entry and telemetry install while enforcing no-arg async signature.
- **Path:** `bin/export-schemas.rs`, `src/schema.rs` (feature gated)  
  **Role:** Schema generation utilities.  
  **Key functionality:** `export-schemas` binary calls `write_all_schemas` to emit JSON Schemas into `dist/schemas/v1/` and creates `.nojekyll`; schema module supplies entry list when `schema` feature is enabled.
- **Path:** `tests/`  
  **Role:** Integration tests validating serde round-trips and compatibility.  
  **Key functionality:** Round-trip coverage for events, messaging, worker envelopes, pack/flow/manifests, store models, supply-chain types, UI documents, and property tests on identifiers/semver requirements.

## 3. Work In Progress, TODOs, and Stubs
- No TODO/FIXME/XXX/unimplemented markers found across source or tests (`rg` search), and no stubbed functions detected.

## 4. Broken, Failing, or Conflicting Areas
- Test status unknown (not run as part of this refresh).

## 5. Notes for Future Work
- Re-run `cargo run --bin export-schemas --all-features` whenever models change to keep `dist/schemas/v1/` in sync.
- `ci/local_check.sh` now checks for `python3` before schema ID verification and `rg` before `scripts/deny_dupes.sh` so missing prerequisites are reported explicitly.
