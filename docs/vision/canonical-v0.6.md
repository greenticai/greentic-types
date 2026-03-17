# Canonical Contract (v0.6)

This is the primary contract for `greentic-types`.

## Scope of canonical types

- Shared IDs and tenant/session/state primitives.
- Canonical pack/component/flow structures.
- Canonical CBOR helpers and schema IDs.
- Policy/security/signing DTOs used across runtime and tooling.

## Canonical expectations

1. Tenant context is always present and propagated at runtime boundaries.
2. Runtime payload contracts are CBOR-first.
3. Pack and component metadata are self-describing and extension-friendly.
4. Security-sensitive structures (signatures/secrets/policy outcomes) stay in shared types.
5. `greentic-types` remains the single source of truth for shared Rust data contracts.

## Non-goals for canonical path

- Domain-specific setup logic.
- Provider-specific orchestration behavior.
- Re-introducing duplicated shared structs in downstream repos.

## Related docs

- Root README canonical usage: [`../../README.md`](../../README.md)
- Models guidance: [`../../MODELS.md`](../../MODELS.md)
- Published schema URLs: [`../../SCHEMAS.md`](../../SCHEMAS.md)
