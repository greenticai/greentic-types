# Architecture context

> [!WARNING]
> This document primarily describes compatibility adapters for legacy schemas.
> For canonical v0.6 design targets, see [`vision/canonical-v0.6.md`](vision/canonical-v0.6.md).

## Legacy adapters
Legacy migration adapters live under `src/adapters/` and map a supported legacy schema module
(e.g., `src/schemas/component/v0_5_0/`) into the canonical v0.6.0 CBOR schemas.

Adapter pattern:
- Define the legacy schema in `src/schemas/<domain>/vX_Y_Z/` with doc comments explaining it is a
  supported migration contract (not necessarily historical truth).
- Implement an adapter in `src/adapters/<domain>_vX_Y_Z_to_v0_6_0.rs` that:
  - Parses legacy input (use `serde_json` only inside adapters if needed).
  - Converts legacy values to v0.6.0 types.
  - Generates deterministic i18n keys and preserves legacy strings as fallbacks.
  - Emits canonical CBOR via `canonical::to_canonical_cbor`.
- Add fixtures in `fixtures/legacy/` and tests that parse legacy input, adapt to v0.6.0,
  decode CBOR, and assert i18n + canonical roundtrips.
