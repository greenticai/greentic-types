# Cross-Repo Smells and Duplicates

## 1) Duplicate pack manifest models across pack toolchain
Evidence:

- `greentic-pack/crates/greentic-pack/src/builder.rs:245` (`PackManifest`)
- `greentic-pack/crates/packc/src/manifest.rs:133` (`PackManifest`)
- `greentic-pack/crates/greentic-pack/src/reader.rs:884-890` dual decode path (`serde_cbor::from_slice::<PackManifest>` then `decode_pack_manifest`)

Conflict:

- Multiple authoritative manifest models break single-source-of-truth and increase conversion bugs.

Target state:

- Canonical manifest type from `greentic-types` only; local models become thin internal build DTOs or removed.

## 2) Extension key drift (`greentic.ext.provider` vs canonical key)
Evidence:

- `greentic-messaging-providers/docs/provider_extension_key_mismatch.md:3-7`
- Pack generation path bypasses canonical constants (`tools/generate_pack_metadata.py` in same repo, documented in audit note)

Conflict:

- Extension discovery/validation is inconsistent across repos.

Target state:

- single canonical extension keyspace from `greentic-types` constants
- CI checks in provider repos reject non-canonical extension IDs

## 3) Lifecycle model fragmentation (QA vs bespoke setup)
Evidence:

- `greentic-operator/src/providers.rs:103-113` hardcoded `setup_default` and local answer handling
- `greentic-operator/src/setup_input.rs:138-145` interactive prompt path
- `greentic-secrets/README.md:85-88` bespoke `wizard/apply` setup flow
- `greentic-events-providers/docs/webhook.md:15` own setup flow lifecycle

Conflict:

- v0.6 expects QA-driven setup/update/remove to be canonical across capabilities.

Target state:

- Operator invokes QA lifecycle contracts uniformly.
- Capability repos expose lifecycle through QA-compatible descriptors.

## 4) JSON/CBOR duality remains runtime-visible
Evidence:

- CBOR-first intent: `greentic-interfaces/README.md:20-21`
- JSON legacy still active in interfaces and other packages (`greentic-interfaces/README.md:27`, many JSON payload types under `crates/greentic-interfaces/wit/...`)
- `greentic-types/src/worker.rs:41-55` JSON worker payload fields
- `greentic-state/README.md:8` explicit JSON-first API
- `greentic-session/README.md:99-101` Redis JSON blobs

Conflict:

- Canonical runtime data path is not singular.

Target state:

- CBOR-first canonical runtime path; JSON retained as compatibility/view layer only.

## 5) Operator still contains provider-aware orchestration logic
Evidence:

- `greentic-operator/src/providers.rs` loops provider packs, handles setup flows, QA apply, status files.
- `greentic-operator/src/project/layout.rs:23-35` bakes in default tenant scaffolding.

Conflict:

- Operator should orchestrate capabilities dynamically, not encode provider lifecycle details.

Target state:

- Provider orchestration metadata lives in pack extensions.
- Operator reads generic capability descriptors + policy.

## 6) Legacy protocol surfaces remain active in core runtime/docs
Evidence:

- `greentic-runner/docs/runner_current_behaviour.md:22` `component@0.5.0` / `@0.4.0` invoke path
- `greentic-interfaces/contracts/0.6.0/COMPAT_MATRIX.md:5-7` legacy versions still listed as active compatibility line

Conflict:

- Legacy is still operationally central rather than isolated compatibility path.

Target state:

- Hard runtime default to v0.6 canonical interfaces; legacy opt-in only with explicit policy gate and sunset.

## 7) Signature posture inconsistent across repos
Evidence:

- Strong path in pack/runner (`greentic-pack/src/reader.rs`, `greentic-runner/crates/runner-core/src/packs/mod.rs:237-245`)
- Missing in distributor client (`greentic-distributor-client/docs/oci_components_audit.md:11,27`)
- Stubbed in mcp (`greentic-mcp/crates/mcp-exec/README.md:12`)

Conflict:

- Supply-chain guarantees are uneven across fetch/execute surfaces.

Target state:

- shared verification policy contract consumed by runner, distributor-client, mcp, deployer.

## 8) Deprecated CLIs and replacement ambiguity
Evidence:

- `greentic-distributor-client/src/bin/greentic-distributor-client.rs:7` deprecation warning

Conflict:

- User-facing command surface is fragmented, migration path unclear in some repos.

Target state:

- one command per domain, explicit migration timeline, compatibility wrapper removed after transition.

## 9) i18n expectations are uneven
Evidence:

- `greentic-types` tenant context has optional `i18n_id` (`src/lib.rs:1297`)
- `greentic-component` scaffolding includes i18n assets (`docs/component_wizard.md:18-19`)
- several capability repos have no strong i18n contract in lifecycle flows.

Conflict:

- first-class i18n requires consistent mandatory context + key distribution.

Target state:

- mandatory `i18n_id` and i18n key contracts across setup/update/remove and runtime responses.

## 10) Missing `greentic-config` repo in audit workspace
Evidence:

- Local path missing: `/projects/ai/greentic-ng/greentic-config`

Conflict:

- Cannot fully verify config ownership, default tenant policy, and global-vs-tenant precedence.

Target state:

- include repo in workspace; run same rubric and boundary checks.
