# TypesValPR-02 — Add generic validation model + (optional) host-side mapping helpers

**Repo:** `greenticai/greentic-types`

## Goal
Provide a single, reusable validation model used by:
- `greentic-pack doctor` output (human + JSON)
- validator WASM execution results
- domain-specific validators (messaging/events/secrets) when compiled as Rust crates or emitted by WASM validators

This PR assumes `greentic-types` is the canonical home for the validation schema.

## Non-goals
- No domain rules here.
- No greentic-pack changes in this PR.

## Deliverables

### A) Validation types
Add module `src/validate/mod.rs` (or repo convention):

- `Severity` enum: `Info | Warn | Error`
- `Diagnostic` struct:
  - `severity: Severity`
  - `code: String`
  - `message: String`
  - `path: Option<String>`
  - `hint: Option<String>`
- `ValidationReport`:
  - `diagnostics: Vec<Diagnostic>`
  - `fn has_errors(&self) -> bool`
  - helpers for counting severities

### B) (Optional) Conversions for WIT validator output
If this repo already generates bindings for WIT types or has a pattern:
- Add small helper to map WIT `Diagnostic` fields (severity string) into `Severity` enum.
- Keep this optional and feature-gated if it introduces dependencies.

### C) Stable JSON output
Ensure `Diagnostic` and `ValidationReport` are serde-serializable with stable field names.
Add a short doc comment indicating stability expectations (these go into doctor JSON).

## Tests
- Serde round-trip for `Diagnostic` and `ValidationReport`.
- `has_errors` behavior tests.

## Acceptance criteria
- Downstream crates can depend on `greentic-types::validate::*` to represent validator results.
- No domain coupling introduced.

