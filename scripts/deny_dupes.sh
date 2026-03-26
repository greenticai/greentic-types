#!/usr/bin/env bash
set -euo pipefail

# Patterns to guard against: if another crate/files re-define these shared structs/enums,
# fail with a helpful diagnostic. Comma-separated allow lists point to the canonical files.
declare -a PATTERNS=(
  "struct[[:space:]]+RunResult([^[:alnum:]_]|$)::src/run.rs"
  "struct[[:space:]]+NodeSummary([^[:alnum:]_]|$)::src/run.rs"
  "struct[[:space:]]+NodeFailure([^[:alnum:]_]|$)::src/run.rs"
  "enum[[:space:]]+RunStatus([^[:alnum:]_]|$)::src/run.rs"
  "enum[[:space:]]+NodeStatus([^[:alnum:]_]|$)::src/run.rs"
  "struct[[:space:]]+Capabilities([^[:alnum:]_]|$)::src/capabilities.rs"
  "struct[[:space:]]+Limits([^[:alnum:]_]|$)::src/capabilities.rs"
  "struct[[:space:]]+TelemetrySpec([^[:alnum:]_]|$)::src/capabilities.rs"
)

STATUS=0

for entry in "${PATTERNS[@]}"; do
  pattern=${entry%%::*}
  allow=${entry#*::}
  IFS=',' read -r -a allowed_files <<<"$allow"

  matches=$(rg --files-with-matches -g'*.rs' "$pattern" --glob '!target/**' --glob '!dist/**' --glob '!greentic-types-macros/**' || true)
  for file in $matches; do
    skip=false
    for allowed_file in "${allowed_files[@]}"; do
      if [[ "$file" == "$allowed_file" ]]; then
        skip=true
        break
      fi
    done

    if [[ "$skip" == false ]]; then
      echo "Found duplicate definition ('$pattern') in $file. Use greentic-types instead of redefining shared structs." >&2
      STATUS=1
    fi
  done
 done

if [[ $STATUS -ne 0 ]]; then
  echo "Duplicate struct definitions detected. Greentic repos should depend on greentic-types instead." >&2
fi

exit $STATUS
