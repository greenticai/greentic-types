# Greentic v0.6.0 Deep Audit: Scope and Method

## Scope scanned
Target repos requested:

- greentic-types
- greentic-interfaces
- greentic-component
- greentic-flow
- greentic-pack
- greentic-runner
- greentic-distributor-client
- greentic-operator
- greentic-secrets
- greentic-messaging-providers
- greentic-events-providers
- greentic-qa
- greentic-telemetry
- greentic-mcp
- greentic-oauth
- greentic-deployer
- greentic-session
- greentic-state
- greentic-config (not present in local workspace)

## What was scanned
For each present repo:

- docs truthfulness (`README*`, `docs/**`, migration files)
- type and interface definitions (Rust structs, WIT packages, schemas)
- lifecycle surfaces (`setup/update/remove`, interactive setup, bespoke CLIs)
- tenant/policy boundaries (`tenant/team/default/global`, policy hooks)
- extension/validator usage and provider wiring
- security/signing/secrets handling
- serialization/envelope patterns (JSON/CBOR/WASM contracts)

## Commands and heuristics
Primary commands used:

- `rg --files` for doc/file inventory
- `rg -n -i` with category keywords (`tenant`, `policy`, `signature`, `cbor`, `json`, `setup_default`, `extension`, `validator`, `legacy`, `deprecated`, `i18n`, `qa`)
- `nl -ba ... | sed -n ...` to capture line-anchored evidence snippets

Heuristics used:

- Treat explicit `legacy/deprecated/migration` markers as non-canonical unless guarded as temporary compatibility
- Score higher when code paths are CBOR-first, self-describing, tenant-scoped, and policy-enforced
- Score lower when repos embed provider-specific setup logic in core orchestration paths
- Score lower when signatures are optional/stubbed or only documented without verification
- Score lower when JSON-first contracts remain in runtime-critical surfaces

## Directories scanned
Workspace base: `/projects/ai/greentic-ng`

Examples of directly scanned paths:

- `/projects/ai/greentic-ng/greentic-types/src`
- `/projects/ai/greentic-ng/greentic-interfaces/crates/greentic-interfaces/wit`
- `/projects/ai/greentic-ng/greentic-pack/crates`
- `/projects/ai/greentic-ng/greentic-runner/crates`
- `/projects/ai/greentic-ng/greentic-operator/src`
- `/projects/ai/greentic-ng/greentic-secrets/crates`
- `/projects/ai/greentic-ng/greentic-messaging-providers/packs`

Excluded/de-emphasized:

- `target/`, `vendor/`, generated lock/cache artifacts except when they were evidence of contract shape

## Limitations

- `greentic-config` repo is missing locally: `MISSING:/projects/ai/greentic-ng/greentic-config`
- This is static analysis only; no end-to-end multi-repo runtime execution was performed
- Existing per-repo audit docs were used as evidence where they describe current code, but not treated as truth by default
- Some findings depend on current docs declaring behavior (flagged where runtime confirmation would be needed)

## Evidence style used in this audit

- Concrete paths
- Symbols/types/functions where applicable
- Short excerpts only
- Conflict against v0.6 north-star called out explicitly
- Target-state proposal attached per major finding
