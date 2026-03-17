# Deprecation Signals and Replacements

Top legacy/compat surfaces currently flagged in `greentic-types`.

| Legacy surface | Status | Replacement |
| --- | --- | --- |
| `WaitScope` alias | Deprecated | `ReplyScope` |
| `LegacyComponentQaSpec` public re-export | Deprecated | `schemas::component::v0_6_0::ComponentQaSpec` |
| QA mode `upgrade` (decode alias) | Compatibility-only | `update` |
| Worker `payload_json` transport fields | Compatibility-only | Canonical CBOR runtime payload/envelope path |
| `*.resolve.json` flow sidecar | Tooling-only compatibility artifact | Canonical flow + pack/component refs |
| Legacy adapter modules under `src/adapters/` | Compatibility-only | Native v0.6 schemas/types |
| Legacy component schema module `v0_5_0` | Compatibility-only | `schemas::component::v0_6_0` |
| Mixed legacy/new tenant identity documentation | Legacy-context docs only | Canonical v0.6 contract docs |
| Migration-only docs in `MIGRATION*.md` | Compatibility guidance | `docs/vision/canonical-v0.6.md` |
| Legacy examples in docs where marked | Compatibility examples | Canonical examples in README + vision docs |

Notes:

- This file tracks signaling; it does not remove behavior.
- Behavior-breaking removals require dedicated migration PRs.
