# GT-PR-01 — Add generic validation model in greentic-types

REPO: greenticai/greentic-types

GOAL
Provide canonical `Diagnostic` + `ValidationReport` types used by:
- greentic-pack doctor JSON output
- validator WASM execution mapping
- domain tooling dashboards

DELIVERABLES
- module `validate` (or repo convention) exposing:
  - enum Severity { Info, Warn, Error }
  - struct Diagnostic { severity, code, message, path: Option<String>, hint: Option<String> }
  - struct ValidationReport { diagnostics: Vec<Diagnostic> } + helpers (has_errors, counts)
- serde support with stable field names
- tests for serde roundtrip + helper behavior

ACCEPTANCE
- downstream crates can depend on `greentic_types::validate::*` without duplicating schemas.

