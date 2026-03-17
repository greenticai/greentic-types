# Deep Dive: greentic-types Findings

## Executive summary
`greentic-types` is close to being the canonical data-contract repository, but it still carries transitional baggage that causes cross-repo drift:

- mixed legacy/new tenant identity fields in the same struct
- both CBOR-first and JSON-first envelope families in core exports
- overlapping ownership with `greentic-interfaces` around invocation semantics and context naming
- runtime envelope variants (`InvocationEnvelope`, worker JSON envelopes, flow resolve JSON) that are not yet cleanly stratified by purpose

## Evidence-backed findings

## 1) Tenant context still carries duplicated legacy/new fields
Evidence:

- `greentic-types/src/lib.rs:1266-1280` (`TenantCtx`) has both `tenant` and `tenant_id`, `team` and `team_id`, `user` and `user_id`.
- `greentic-types/src/lib.rs:1297` has `i18n_id: Option<String>`.

Why this conflicts with v0.6:

- North star expects one canonical tenant identity surface with mandatory i18n context.
- Optional `i18n_id` and duplicated aliases increase mapper complexity and allow partial context propagation.

Target state:

- `TenantCtxV0_6` (or next-version replacement) with single canonical identity fields and required `i18n_id`.
- Compatibility mapper retained behind explicit legacy feature gate.

## 2) Invocation semantics drift from interface call-spec model
Evidence:

- `greentic-types/src/lib.rs:1438-1451` defines `InvocationEnvelope { ctx, flow_id, node_id, op, payload, metadata }`.
- `greentic-interfaces/contracts/0.6.0/RENAMES.md` and `CANONICAL_MAP.md` model flow persistence as `CallSpec` with host-owned envelope wrapping.

Why this conflicts:

- Multiple envelope idioms lead to duplicated conversion logic and ambiguous persistence boundaries.

Target state:

- In `greentic-types`, expose one canonical runtime invocation struct aligned to interfaces terminology (`CallSpec` + host context wrapper fields).
- Keep legacy alias structs only in `compat` module.

## 3) JSON worker envelopes are in foundational shared types
Evidence:

- `greentic-types/src/worker.rs:41-55` uses `payload_json: String` in both request and message.

Why this conflicts:

- Core shared-types crate should privilege canonical CBOR payload model; JSON strings here become de facto runtime contract.

Target state:

- Keep worker transport DTOs in a dedicated compatibility/transport module or separate crate.
- Add CBOR payload variant for canonical worker runtime path.

## 4) Good CBOR foundation exists but is diluted by mixed contract surfaces
Evidence:

- `greentic-types/src/envelope.rs` canonical CBOR envelope APIs.
- `greentic-types/src/qa.rs` supports canonical validation/canonicalization of CBOR answers.
- `greentic-types/src/cbor/mod.rs` includes dedicated pack manifest encode/decode.

Why this matters:

- The right primitives already exist; migration risk is mostly ownership/compat cleanup, not greenfield.

Target state:

- Promote these modules as the only canonical runtime data path.
- Mark JSON compatibility surfaces explicitly as transitional.

## 5) Docs mix canonical intent with compatibility behavior
Evidence:

- `greentic-types/README.md` states CBOR/WIT evolution and canonical contracts.
- `greentic-types/docs/worker.md` still defines JSON payload fields as baseline.

Why this conflicts:

- Users cannot easily distinguish canonical runtime contracts from compatibility transport artifacts.

Target state:

- Split docs into:
  - `canonical/` (must-use v0.6 path)
  - `compat/` (legacy adapters, sunset date)

## Ownership model proposal (types vs interfaces)

## What stays in greentic-types

- Rust-native canonical data structures (tenant context, ids, pack/flow/component manifests, policy DTOs).
- Canonical CBOR envelopes and schema IDs.
- Signature and manifest verification DTO structures.
- Shared validation primitives for canonical model.

## What stays in greentic-interfaces

- WIT package definitions and versioned ABI surfaces.
- Generated guest/host bindings and minimal mappers.
- Compatibility matrix and ABI deprecation policy.

## What should move out to extension packs

- provider-specific runtime metadata that is not cross-domain canonical
- domain-specific setup payload conventions currently represented as ad-hoc JSON blobs

## Delete / deprecate / rewrite in greentic-types

Delete (after migration):

- duplicate legacy identity fields in `TenantCtx`.

Deprecate:

- JSON-string worker payload contract as canonical shared type.
- any envelope forms not aligned to call-spec + host-wrapper model.

Rewrite:

- `TenantCtx` into a strict v0.6 canonical shape.
- runtime invocation struct naming/fielding to match interface contract vocabulary.

## Migration notes for downstream repos

- `greentic-runner`, `greentic-operator`, `greentic-session`, `greentic-state`, `greentic-oauth` are highest-impact consumers.
- Introduce a compatibility shim crate/module first, then remove legacy fields in one major-version step.
- Add compile-time lints/deny rules for legacy fields once replacement API is shipped.
