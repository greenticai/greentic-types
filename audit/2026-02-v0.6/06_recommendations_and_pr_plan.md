# Recommendations and PR Plan

## Prioritization strategy

1. Stop the confusion (docs + explicit deprecations)
2. Single source of truth for types/contracts
3. QA lifecycle normalization
4. Extensions extraction and operator simplification

## Stop-the-bleeding recommendations (immediate)

- Mark canonical vs compatibility contracts unambiguously in docs.
- Freeze new legacy surface additions (`component@0.4/0.5`, old pack-export, legacy extension keys).
- Add CI checks for canonical extension IDs and disallow duplicate pack manifest definitions in new code.

## Proposed PR series

## PR-01: Canonical vs Legacy Docs Split
Goal
- Remove ambiguity in all core repos.

Scope
- Add `canonical/` and `compat/` docs sections.
- Rewrite top-level READMEs to show canonical path first.

Acceptance criteria
- Every audited repo has explicit canonical/compat note.
- Legacy surfaces include removal target release.

Files/modules likely touched
- `README.md`, `docs/**`, `MIGRATION*.md` across repos.

Migration notes
- No code behavior change.

Risk & rollback
- Low risk; rollback is doc revert.

## PR-02: TenantCtx Canonicalization in greentic-types
Goal
- Introduce strict v0.6 tenant context shape.

Scope
- Add new canonical `TenantCtx` version with required `i18n_id`.
- Add compatibility mapper from legacy fields.

Acceptance criteria
- Core crates compile using canonical context.
- Legacy access only via compatibility API.

Files/modules likely touched
- `greentic-types/src/lib.rs`
- downstream mappers in `greentic-interfaces` and runtime repos.

Migration notes
- Transitional dual support behind feature flag.

Risk & rollback
- Medium; rollback by keeping dual field struct behind default feature.

## PR-03: Invocation Contract Unification (CallSpec + Envelope)
Goal
- Align `greentic-types` and `greentic-interfaces` invocation semantics.

Scope
- Define one canonical runtime invocation model.
- Deprecate overlapping envelope variants.

Acceptance criteria
- No new code path serializes ambiguous envelope formats.
- Interfaces and types docs match exactly.

Files/modules likely touched
- `greentic-types/src/lib.rs`, `greentic-types/src/envelope.rs`
- `greentic-interfaces/contracts/0.6.0/*`

Migration notes
- Provide conversion helpers for old envelope shape.

Risk & rollback
- Medium-high; rollback by retaining old struct with explicit `legacy` namespace.

## PR-04: Pack Manifest Single Source of Truth
Goal
- Eliminate manifest model duplication.

Scope
- Refactor `greentic-pack` + `packc` to use canonical `greentic-types::PackManifest`.
- Remove dual decode fallback path after migration.

Acceptance criteria
- One manifest model in pack pipeline.
- Build/doctor/sign/verify pass with canonical manifest only.

Files/modules likely touched
- `greentic-pack/crates/greentic-pack/src/builder.rs`
- `greentic-pack/crates/packc/src/manifest.rs`
- `greentic-pack/crates/greentic-pack/src/reader.rs`

Migration notes
- Keep read-only compatibility adapter for previously built packs until cutoff.

Risk & rollback
- High; rollback by keeping compatibility decode in isolated adapter module.

## PR-05: Canonical Extension Key Enforcement
Goal
- End extension key drift.

Scope
- Enforce canonical provider extension key in pack generators and validators.
- Fail CI on `greentic.ext.provider` for new/updated artifacts.

Acceptance criteria
- Messaging/deployer/secrets provider packs all emit canonical extension ID.
- Existing test fixtures updated.

Files/modules likely touched
- `greentic-messaging-providers/tools/*`
- `greentic-pack` validators
- provider repo manifests/tests

Migration notes
- One-time migration script for old manifests.

Risk & rollback
- Medium; rollback by temporary dual-key reader (write canonical only).

## PR-06: QA Lifecycle Contract v0.6
Goal
- Make `greentic-qa` the canonical setup/update/remove contract owner.

Scope
- Define lifecycle schema and runtime protocol in `greentic-qa`.
- Publish reusable contract crate/interfaces.

Acceptance criteria
- Operator can invoke setup/update/remove through QA contract for at least one domain end-to-end.

Files/modules likely touched
- `greentic-qa/crates/*`
- `greentic-interfaces` (if WIT additions needed)

Migration notes
- Existing wizard features become one UX adapter over canonical lifecycle.

Risk & rollback
- Medium; rollback by running QA and legacy paths in parallel behind feature flag.

## PR-07: Operator QA-only Lifecycle Execution
Goal
- Remove bespoke provider setup orchestration from operator core.

Scope
- Replace `setup_input` interactive/provider-specific flows with QA contract invocations.
- Keep operator as orchestration + policy + capability wiring.

Acceptance criteria
- `demo/domain setup/update/remove` paths use QA contracts.
- Provider-specific setup code removed from operator core.

Files/modules likely touched
- `greentic-operator/src/providers.rs`
- `greentic-operator/src/setup_input.rs`
- `greentic-operator/src/component_qa_ops.rs`

Migration notes
- Temporary adapter to map old setup.yaml answers into QA payload.

Risk & rollback
- High; rollback by feature-gated legacy provider setup path.

## PR-08: Legacy Interface Runtime Gate
Goal
- Contain legacy component protocols.

Scope
- Make legacy component invokes opt-in only (`LEGACY_APPROVED`/equivalent).
- Default runtime path is canonical 0.6.

Acceptance criteria
- Runner defaults to v0.6 invocation path.
- Legacy paths emit explicit warning/telemetry and are deny-by-default.

Files/modules likely touched
- `greentic-runner/docs/*`
- runtime invocation modules in `greentic-runner` and `greentic-interfaces`.

Migration notes
- Document per-tenant override and sunset timeline.

Risk & rollback
- Medium-high; rollback by temporarily defaulting policy to permissive while keeping gate.

## PR-09: Signature Policy Convergence
Goal
- Uniform verification policy across runtime/distribution/mcp.

Scope
- Implement signature verification in distributor-client and mcp.
- Harmonize strict/dev behavior with runner/pack.

Acceptance criteria
- Distributor-client rejects unsigned/invalid artifacts in strict mode.
- MCP exec verifies digest/signature according to policy.

Files/modules likely touched
- `greentic-distributor-client/src/*`
- `greentic-mcp/crates/mcp-exec/*`

Migration notes
- Start with strict optional flag, then promote to default in next release.

Risk & rollback
- Medium; rollback to warn-only mode with explicit insecure marker.

## PR-10: CBOR-first Storage Envelope for Session/State
Goal
- Remove JSON-only persistence assumptions from core session/state.

Scope
- Add canonical CBOR storage formats with migration adapters.
- Preserve JSON debug tooling as compatibility layer.

Acceptance criteria
- New writes use canonical CBOR by default.
- Old JSON entries still readable during migration window.

Files/modules likely touched
- `greentic-session`
- `greentic-state`
- `greentic-runner` integration points

Migration notes
- Background migration or lazy on-read rewrite strategy.

Risk & rollback
- Medium-high; rollback by keeping dual read/write toggle.

## PR-11: Capability Extraction Roadmap (extensions)
Goal
- Move non-core domain logic out of core repos.

Scope
- Define extraction boundaries for telemetry, state, secrets, oauth, deployer capability packs.
- Add operator capability registry resolution flow.

Acceptance criteria
- At least one domain (e.g., secrets) runs as pure extension capability via operator wiring.

Files/modules likely touched
- `greentic-operator`, `greentic-pack`, domain repos.

Migration notes
- Phased extraction by domain; keep compatibility pack bundles.

Risk & rollback
- High; rollback by dual registration (embedded + extension).

## PR-12: Release Guardrails and Conformance Checks
Goal
- Prevent regression into legacy drift.

Scope
- Add conformance checks for: extension keys, legacy interface usage, mandatory i18n_id, QA lifecycle contract usage, signature enforcement status.

Acceptance criteria
- CI fails on non-canonical additions in core repos.
- Versioned conformance report generated per release.

Files/modules likely touched
- CI scripts across repos.

Migration notes
- Start in warn mode for one release, then enforce.

Risk & rollback
- Low-medium; rollback by downgrading blockers to warnings temporarily.

## Suggested execution order

1. PR-01
2. PR-02
3. PR-03
4. PR-04
5. PR-05
6. PR-06
7. PR-07
8. PR-08
9. PR-09
10. PR-10
11. PR-11
12. PR-12
