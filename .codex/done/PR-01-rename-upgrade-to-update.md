# PR-01: Rename QA lifecycle mode upgrade -> update (types layer)

## Goals
- Replace the lifecycle mode name **`upgrade`** with **`update`** in all 0.6-facing types/CBOR contracts.
- Provide a temporary compatibility alias so older callers using `upgrade` still work during a migration window.
- Keep canonical CBOR encoding stable (no accidental changes to canonicalization).

## Implementation Steps
1) Locate the authoritative enum / type(s) that represent QA/lifecycle mode for 0.6:
   - Search for `QaMode`, `WizardMode`, `upgrade`, `Upgrade`.
   - Identify serialization (serde/minicbor) and any string conversions.

2) Rename:
   - Update enum variant `Upgrade` -> `Update` (or if variant names must remain, ensure external string is `update`).
   - Update all mappings: `as_str()`, `Display`, `FromStr`, clap value enums (if any live here), schema IDs, etc.

3) Backward compatibility:
   - Implement `FromStr` (or equivalent) accepting both `"update"` and `"upgrade"`, but:
     - Prefer `"update"` for Display/output.
     - Emit a deprecation warning mechanism if this crate supports it (feature-gated, or return a structured “deprecated” flag). If not, just accept alias silently and document in CHANGELOG/README.
   - If CBOR encodes enum as integer, keep existing discriminants stable and only change textual/JSON forms.
   - If CBOR encodes enum as text, accept both on decode and always encode `"update"` going forward.

4) Update any public structs that contain the mode (requests, events, envelopes):
   - Ensure docs/comments call it `update`.

5) Tests:
   - Add unit tests that:
     - parse `"upgrade"` -> Update
     - parse `"update"` -> Update
     - serialization roundtrip encodes Update consistently
     - discriminants/canonical CBOR remain stable (golden bytes if you have a pattern)

6) Update docs/changelog if present.

## Acceptance Criteria
- `rg -n "\bupgrade\b" src tests` finds **no** remaining references in 0.6 APIs except:
  - compatibility decoding paths (explicitly documented).
- Tests pass: `cargo test`.
- If this crate provides JSON/CBOR schema output, it reflects `update`.


