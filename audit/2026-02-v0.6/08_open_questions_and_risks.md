# Open Questions and Risks

## Open questions

1. `greentic-config` ownership and defaults
- Repo is missing locally, so default tenant/environment precedence and policy boundaries could not be validated.

2. Runtime truth vs docs for legacy invokes
- `greentic-runner/docs/runner_current_behaviour.md` states legacy invoke path usage.
- Need confirmation from current runtime code paths/tests whether this remains active by default.

3. Canonical extension ID migration timeline
- How quickly can provider repos migrate from `greentic.ext.provider` without breaking existing packs?

4. QA contract format
- Should QA lifecycle be expressed entirely in CBOR/WIT, or as hybrid (CBOR runtime + JSON authoring tooling)?

5. Session/state canonical storage policy
- Should core persistence switch to CBOR hard-default in v0.6.x or staged in v0.7?

6. Operator scope boundary
- Which pieces remain in operator core vs extracted into capability packs during first extraction wave?

## Primary migration risks

1. Contract fragmentation during transition
- Running old/new envelopes and manifest models simultaneously can create silent mismatch bugs.

2. Pack compatibility breakage
- Migrating extension keys and manifest ownership can invalidate existing built `.gtpack` artifacts.

3. Tenant identity regressions
- Removing duplicate identity fields without strong mappers can break downstream assumptions.

4. Lifecycle UX disruption
- Replacing bespoke setup prompts with QA contracts may regress operator workflows if not phased.

5. Signature policy hardening fallout
- Enabling strict signature checks in currently permissive clients may block existing dev pipelines.

## Compatibility traps

- Docs that declare legacy compatibility without explicit default/priority can be misread as canonical.
- Feature flags that silently enable legacy interfaces can hide drift.
- JSON compatibility paths may be used accidentally as runtime contracts.

## Risk mitigations

- Add conformance CI checks before large refactors.
- Ship compatibility adapters with explicit sunset dates.
- Add migration-report artifacts for pack/interface conversions.
- Roll out strict signature policy in warn -> enforce phases.
- Add tenant/i18n contract assertions in integration tests.

## Suggested decision gates before refactor wave

1. Approve canonical `TenantCtx` schema and deprecation map.
2. Approve canonical invocation envelope/call-spec contract.
3. Approve extension key migration and CI enforcement date.
4. Approve QA lifecycle contract format and operator integration model.
5. Approve signature policy baseline (strict defaults by environment).
