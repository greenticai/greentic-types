# Repo Scorecards (v0.6 rubric)

Scoring: `0` absent/conflicting, `3` partial/mixed, `5` strongly aligned.

Categories:
- `A` v0.6 component/pack alignment
- `B` QA lifecycle alignment
- `C` Multi-tenancy alignment
- `D` Security posture
- `E` CBOR/WASM-first alignment
- `F` i18n readiness
- `G` Operator-orchestrator model compatibility
- `H` Duplication/coupling
- `I` Docs correctness

## greentic-types
Scores: `A4 B2 C4 D3 E3 F3 G2 H2 I3`

✅ Aligned (keep and build on)
- Canonical pack/type structures and CBOR helpers (`src/pack_manifest.rs`, `src/cbor/mod.rs`, `src/envelope.rs`).
- Shared tenant/session primitives.

🟡 Needs work (top 5)
- Make `i18n_id` mandatory in `TenantCtx` (`src/lib.rs:1297`).
- Remove dual legacy/new identity fields (`tenant` + `tenant_id`, etc.) once migration lands (`src/lib.rs:1266-1280`).
- Split JSON worker envelopes from core runtime types (`src/worker.rs:41-55`).
- Consolidate invocation envelope semantics with interfaces call-spec model.
- Tighten ownership boundaries with `greentic-interfaces`.

❌ Goes against vision
- JSON-first worker payload fields in foundational types (`payload_json`).

## greentic-interfaces
Scores: `A3 B2 C4 D3 E2 F3 G2 H1 I3`

✅ Aligned
- Canonical 0.6 contract docs and naming map (`contracts/0.6.0/*`).
- Explicit CBOR call-spec framing in README.

🟡 Needs work (top 5)
- Remove active legacy surfaces from default path (`component@0.4/0.5`, old pack-export).
- Eliminate repeated `tenant-ctx` definitions across WIT trees.
- Reduce JSON-string payload worlds still exported.
- Clarify what is canonical vs compatibility-only in published interfaces.
- Add stronger contract linting for legacy package usage.

❌ Goes against vision
- Mixed canonical + legacy interfaces presented as co-equal (`README.md:27`).

## greentic-component
Scores: `A4 B3 C2 D3 E4 F4 G3 H3 I3`

✅ Aligned
- 0.6 component authoring flow and CBOR runtime intent.
- i18n scaffolding (`docs/component_wizard.md:18-19`).

🟡 Needs work (top 5)
- Ensure generated components default to strict 0.6-only surfaces.
- Improve tenant-aware assumptions for generated/testing paths.
- Harden signing/verification defaults in CLI workflows.
- Explicitly route setup/update/remove through QA contracts.
- Deprecate legacy output paths earlier.

❌ Goes against vision
- Legacy output/documented compatibility paths still central in UX.

## greentic-flow
Scores: `A4 B3 C2 D2 E3 F3 G3 H3 I3`

✅ Aligned
- Strong flow tooling + doctor discipline.
- Wizard lifecycle terminology mostly aligned (`default/setup/update/remove`).

🟡 Needs work (top 5)
- Remove `upgrade` alias and residual legacy mode handling.
- Strengthen tenant metadata propagation in flow artifacts.
- Move more JSON flow metadata toward canonical CBOR call-spec tooling.
- Add policy/audit hooks for flow mutation operations.
- Improve validator coverage for extension/validator metadata.

❌ Goes against vision
- Remaining legacy mode aliases (`upgrade`) keep old lifecycle semantics alive.

## greentic-pack
Scores: `A4 B2 C3 D4 E4 F2 G2 H1 I4`

✅ Aligned
- Strong signing and verify paths (`crates/greentic-pack/src/reader.rs`).
- Pack kinds and extension concepts are present.

🟡 Needs work (top 5)
- Remove duplicate `PackManifest` models (`builder.rs` vs `packc/src/manifest.rs`).
- Remove dual decode fallback model (`reader.rs:884-890`).
- Route lifecycle metadata explicitly through QA contracts.
- Normalize extension key policy with greentic-types constants.
- Tighten validator model as first-class pack metadata.

❌ Goes against vision
- Multiple pack manifest definitions create contract drift.

## greentic-runner
Scores: `A3 B2 C4 D4 E3 F2 G2 H2 I3`

✅ Aligned
- Multi-tenant runtime architecture and policy hooks are substantial.
- Pack signature checks in resolver path exist.

🟡 Needs work (top 5)
- Remove legacy component invoke path (`component@0.5/@0.4`) from runtime docs/behavior.
- Enforce strict signature policy by default where possible.
- Reduce JSON envelope dependence in runtime validation and state helpers.
- Formalize operator-runner CBOR contract (currently documented/planned, not universal).
- Strengthen i18n propagation guarantees.

❌ Goes against vision
- Legacy component protocol still documented as active execution path (`docs/runner_current_behaviour.md:22`).

## greentic-distributor-client
Scores: `A2 B1 C3 D1 E1 F1 G2 H3 I3`

✅ Aligned
- Useful as a thin distribution abstraction and cache helper.

🟡 Needs work (top 5)
- Implement real signature/provenance verification.
- Move off JSON-centric DTO assumptions for runtime-facing contracts.
- Align extension model with canonical types keyspace.
- Clarify long-term role vs `greentic-dist` successor.
- Add stronger tenant/policy guards for client operations.

❌ Goes against vision
- Explicitly no signature verification today (`docs/oci_components_audit.md:11,27`).

## greentic-operator
Scores: `A2 B2 C3 D3 E2 F2 G1 H2 I3`

✅ Aligned
- Central orchestration intent and tenant/team project model.
- Secrets canonical URI handling is improving.

🟡 Needs work (top 5)
- Remove provider-specific setup orchestration from core (`src/providers.rs`).
- Replace bespoke interactive setup with QA canonical lifecycle (`src/setup_input.rs`).
- Stop creating implicit default tenant/team scaffolding (`src/project/layout.rs:23-35`).
- Move domain/provider knowledge to extension packs.
- Make operator primarily policy + wiring engine.

❌ Goes against vision
- Operator currently embeds provider setup behavior and interactive setup logic.

## greentic-secrets
Scores: `A3 B1 C4 D4 E2 F1 G2 H3 I4`

✅ Aligned
- Strong tenant/team secret scoping and policy docs.
- Clear extension-oriented packaging in many areas.

🟡 Needs work (top 5)
- Replace secrets-specific setup wizard/init lifecycle with QA-driven lifecycle.
- Increase CBOR-first contract usage (many JSON-centric flows remain).
- Ensure operator integration is extension-capability, not bespoke coupling.
- Improve i18n readiness for user-facing setup UX.
- Consolidate provider-extension ID consistency with ecosystem.

❌ Goes against vision
- Dedicated bespoke lifecycle (`wizard/apply/init`) bypasses canonical QA path.

## greentic-messaging-providers
Scores: `A3 B2 C3 D3 E3 F4 G2 H1 I3`

✅ Aligned
- Strong extension-pack orientation and good i18n investment.
- Rich provider flow coverage.

🟡 Needs work (top 5)
- Migrate legacy extension key `greentic.ext.provider` to canonical key.
- Remove Python metadata path that bypasses greentic-types guardrails.
- Normalize setup lifecycle to QA canonical model end-to-end.
- Tighten signature/provenance expectations for provider packs.
- Reduce contract drift between pack YAML/manifest/CBOR outputs.

❌ Goes against vision
- Legacy provider extension key is entrenched (`docs/provider_extension_key_mismatch.md`).

## greentic-events-providers
Scores: `A3 B2 C3 D2 E2 F1 G2 H2 I3`

✅ Aligned
- Pack-based provider decomposition is directionally good.

🟡 Needs work (top 5)
- Move fully to v0.6 provider-core/canonical worlds (docs still mention older events world).
- Normalize setup lifecycle via QA conventions.
- Strengthen signing/attestation expectations.
- Improve i18n support for operator/user-facing interactions.
- Reduce JSON-centric config assumptions.

❌ Goes against vision
- Legacy interface framing still present in docs (`docs/overview.md:5`).

## greentic-qa
Scores: `A2 B4 C1 D2 E2 F1 G2 H3 I3`

✅ Aligned
- Clear QA tooling assets and reusable question/spec model.
- Good foundation for canonical lifecycle engine.

🟡 Needs work (top 5)
- Expand from form wizard tooling to canonical setup/update/remove runtime contract.
- Add tenant-aware lifecycle context by default.
- Increase CBOR-first contract support for runtime integration.
- Add i18n-first authoring/runtime guarantees.
- Align pack dependencies away from legacy external IDs.

❌ Goes against vision
- Current focus is mostly wizard/form generation, not full lifecycle orchestration standard.

## greentic-telemetry
Scores: `A1 B0 C2 D2 E0 F1 G1 H3 I3`

✅ Aligned
- Useful tenant-scoped telemetry context model.

🟡 Needs work (top 5)
- Define CBOR-first telemetry envelope for runtime links.
- Add explicit policy/audit hooks for sensitive telemetry paths.
- Strengthen i18n-aware telemetry label strategy.
- Clarify extension packaging and runtime capability boundaries.
- Reduce env-driven implicit behavior in core paths.

❌ Goes against vision
- No CBOR/WASM-first contract posture in current public surface.

## greentic-mcp
Scores: `A2 B1 C2 D2 E1 F0 G2 H2 I3`

✅ Aligned
- Potentially useful as dynamic capability execution layer.

🟡 Needs work (top 5)
- Replace legacy exec shims with canonical capability model.
- Implement actual signature verification (currently stubs).
- Align payload contracts with CBOR-first runtime boundaries.
- Add explicit tenant and policy enforcement defaults.
- Improve i18n/readable contract support where user-facing outputs exist.

❌ Goes against vision
- Signature enforcement is stubbed (`crates/mcp-exec/README.md:12`).

## greentic-oauth
Scores: `A3 B2 C4 D4 E0 F1 G2 H3 I3`

✅ Aligned
- Strong multi-tenant and security posture overall.
- Rich audit events and broker model.

🟡 Needs work (top 5)
- Remove/retire legacy endpoint compatibility paths.
- Add CBOR-first contract track for broker-runtime calls.
- Improve i18n in user-facing auth prompts/messages.
- Further decouple from core orchestration assumptions.
- Consolidate extension capability packaging story.

❌ Goes against vision
- Legacy start endpoints remain active (`README.md:256-258`).

## greentic-deployer
Scores: `A2 B1 C3 D3 E2 F1 G2 H2 I3`

✅ Aligned
- Good direction toward deployment packs and provider mapping.

🟡 Needs work (top 5)
- Remove legacy Rust backend fallback model.
- Make deployment capability fully extension-pack driven.
- Align provider extension keys to canonical IDs.
- Increase QA lifecycle normalization for install/update/remove flows.
- Improve CBOR/policy/audit contract cohesion with operator/runner.

❌ Goes against vision
- Legacy backend fallback still first-class (`README.md:11`, `README.md:63`).

## greentic-session
Scores: `A2 B1 C4 D2 E0 F0 G2 H3 I3`

✅ Aligned
- Strong tenant context enforcement and scoping.

🟡 Needs work (top 5)
- Add canonical CBOR storage envelope option (currently JSON blob focus).
- Add explicit policy/audit hooks around session reads/writes.
- Improve lifecycle integration with QA setup/update/remove context.
- Reduce compatibility/deprecated API surface.
- Clarify extension packaging boundary if extracted from core.

❌ Goes against vision
- JSON-centric storage contract (`README.md:99-101`).

## greentic-state
Scores: `A2 B1 C4 D2 E0 F0 G2 H3 I3`

✅ Aligned
- Strict tenant-aware keying and robust state primitives.

🟡 Needs work (top 5)
- Introduce CBOR-native storage and pointer model for canonical runtime.
- Add policy/audit boundary hooks on state operations.
- Improve lifecycle integration semantics for setup/update/remove artifacts.
- Clarify whether state remains core or becomes extension capability.
- Add i18n-aware metadata strategy for user-visible state diagnostics.

❌ Goes against vision
- Explicit JSON-first API remains core (`README.md:8`).

## greentic-config
Scores: `N/A` (repo missing in workspace)

✅ Aligned
- Not assessable locally.

🟡 Needs work (top 5)
- Add repo to workspace for complete v0.6 audit.
- Validate tenant defaults and policy boundary semantics.
- Validate config ownership boundary vs `greentic-types`.
- Validate CBOR/signing/i18n alignment.
- Validate QA lifecycle integration points.

❌ Goes against vision
- Missing repository blocks full cross-repo contract validation.
