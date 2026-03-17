# PR-02: Add migration notes for upgrade -> update

## Goals
- Ensure any cross-crate feature flags / semver notes are prepared for downstream repos (interfaces/flow/pack/component/providers).
- Provide a migration note that downstream should treat `upgrade` as deprecated alias.

## Implementation Steps
1) Add a MIGRATION.md (or add section to README/CHANGELOG) describing:
   - New name `update`
   - Alias handling
   - Expected downstream changes (interfaces + tooling)
2) If you maintain versioned modules (e.g., `v0_6_0`), update references accordingly.
3) If there is a `schema_hash` mechanism relying on text, verify unchanged where intended.

## Acceptance Criteria
- Migration note exists in repo.
- No breaking change to canonical CBOR bytes unless explicitly intended and documented.


