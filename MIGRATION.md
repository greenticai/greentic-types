# Migration Guide

> [!WARNING]
> This file is compatibility guidance.
> Canonical v0.6 contract docs are in [`docs/vision/canonical-v0.6.md`](docs/vision/canonical-v0.6.md).

## QA Mode Rename: `upgrade` -> `update` (v0.6 schemas)

`greentic-types` now uses `update` as the canonical QA lifecycle mode name in v0.6 pack/component QA schemas.

- Canonical emit/encode: `update`
- Backward-compatible decode/parse: accepts both `update` and `upgrade`
- Compatibility behavior in this crate is intentionally silent (no warning emission)

This change is scoped to QA lifecycle mode naming only. Unrelated fields such as pack bootstrap `upgrade_flow` are unchanged.

## Downstream Impact

Downstream repos should move emitted payloads and generated artifacts to `update`:

- `greentic-interfaces`
- `greentic-flow`
- pack/component tooling and providers (CLI/operator/host layers)

During migration, downstream readers can continue accepting `upgrade` as a deprecated alias while standardizing outputs to `update`.

## Schema / Hash Notes

- This change does not alter the component describe `schema_hash` algorithm or inputs.
- Existing payloads containing `upgrade` remain decodable.
- Newly emitted QA mode values from this crate are `update`.
