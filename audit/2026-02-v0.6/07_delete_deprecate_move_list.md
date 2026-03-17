# Delete / Deprecate / Move List

## Format
- `Action`: delete, deprecate, or move
- `Item`
- `Rationale`
- `Blast radius`

| Action | Item | Rationale | Blast radius |
| --- | --- | --- | --- |
| Deprecate | `greentic-types::TenantCtx` duplicate identity fields (`tenant`/`tenant_id`, `team`/`team_id`, `user`/`user_id`) | Removes ambiguous canonical identity shape | High: runner/operator/interfaces/oauth/session/state |
| Deprecate | Optional `i18n_id` in `TenantCtx` | v0.6 requires first-class i18n context | High: all context constructors/mappers |
| Move | JSON worker envelopes (`greentic-types/src/worker.rs`) to compat transport module/crate | Keep core shared contracts CBOR-first | Medium: worker consumers |
| Deprecate | Non-canonical invocation envelope variants in `greentic-types` | Align to single call-spec + host wrapper model | High: runner/interfaces/component |
| Delete (post-migration) | Duplicate pack manifest model in `greentic-pack/src/builder.rs` | Single source of truth in `greentic-types` | High: pack build pipeline |
| Delete (post-migration) | Duplicate pack manifest model in `greentic-pack/crates/packc/src/manifest.rs` | Same as above | High: packc CLI/tests |
| Deprecate | `greentic-pack` dual decode fallback (`reader.rs:884-890`) | Legacy decode path should be isolated and sunset | Medium-high: pack ingestion |
| Deprecate | Legacy extension key `greentic.ext.provider` in provider repos | Canonical extension key consistency | High: messaging/deployer/provider tooling |
| Move | Provider metadata generation bypassing canonical types (`greentic-messaging-providers/tools/generate_pack_metadata.py`) | Metadata ownership should be types-driven | Medium: provider build scripts |
| Delete | Operator interactive setup prompt path (`greentic-operator/src/setup_input.rs`) from core orchestration path | QA must own lifecycle UX | High: operator setup UX |
| Deprecate | Hardcoded `setup_default` orchestration in operator (`greentic-operator/src/providers.rs`) | Core operator should not encode provider setup semantics | High: operator runtime behavior |
| Deprecate | Implicit default tenant scaffold in operator (`greentic-operator/src/project/layout.rs`) | Avoid hidden global defaults | Medium: demo bootstrap flows |
| Deprecate | Active legacy invoke path docs/behavior in runner (`component@0.5/@0.4`) | Canonical v0.6 runtime should be default | High: runtime compatibility |
| Deprecate | `greentic-distributor-client` deprecated CLI wrapper | Reduce command-surface confusion | Low-medium: operator/dev scripts |
| Move | Signature verification logic into shared policy package used by distributor-client + mcp | Uniform supply-chain guarantees | Medium: distributor/mcp |
| Deprecate | mcp legacy exec shims and signature stubs as default path | Must converge to canonical verified execution | Medium-high: mcp users |
| Deprecate | OAuth legacy start endpoint (`/:env/:tenant/:provider/start`) | Reduce legacy protocol surface | Medium: old OAuth clients |
| Move | Secrets `wizard/apply/init` lifecycle to QA-driven lifecycle contracts | Canonical setup/update/remove model | High: secrets operator flows |
| Move | Events-provider setup lifecycle conventions to QA contracts | Same lifecycle normalization | Medium: events setup tooling |
| Deprecate | JSON-first session persistence as only canonical path | Introduce CBOR canonical persistence | Medium-high: session stores |
| Deprecate | JSON-first state API as only canonical path | Introduce CBOR canonical persistence | Medium-high: state consumers |
| Move | Telemetry env-only setup assumptions into extension policy/config model | Core should be explicit and policy-governed | Medium: telemetry bootstrap |

## Candidate immediate deletions (low risk)

- `greentic-distributor-client` deprecation wrapper binary after migration completion.
- stale docs that describe legacy as current default without compatibility warning.

## Candidate delayed deletions (require migration window)

- legacy component interface runtime paths
- duplicate pack manifest structs
- legacy extension keys
- duplicated tenant identity fields in `TenantCtx`
