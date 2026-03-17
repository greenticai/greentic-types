# Legacy and Compatibility Surfaces

This page tracks compatibility-only surfaces. They are supported for migration, not as primary design targets.

## Legacy/compat list

- `LegacyComponentQaSpec` (`src/schemas/component/v0_5_0/*`): v0.5 QA schema adapter input.
- `WaitScope` (`src/session.rs`): alias for `ReplyScope`.
- `upgrade` QA mode decode compatibility (`MIGRATION.md`): accepted for decode, canonical emit is `update`.
- Worker JSON payload fields (`payload_json` in `src/worker.rs`): compatibility transport model.
- Flow resolve JSON sidecar (`*.resolve.json` via `src/flow_resolve.rs`): tooling artifact, not runtime canonical envelope.

## WIT deprecation note

`greentic-types` does not define WIT packages directly; WIT versioning/deprecation is handled in `greentic-interfaces`. No WIT `@deprecated` edits are applicable in this repository.

## Usage policy

- Do not introduce new dependencies on these surfaces.
- Prefer canonical v0.6 contracts documented in [`canonical-v0.6.md`](canonical-v0.6.md).
- Keep compatibility adapters isolated and explicitly labeled.
