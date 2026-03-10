#!/usr/bin/env bash
set -euo pipefail

# Candidate schema base URLs. During org migration, one of these may 404 temporarily.
SCHEMA_BASES_DEFAULT=(
  "https://greenticai.github.io/greentic-types/schemas/v1"
  "https://greenticai-org.github.io/greentic-types/schemas/v1"
)

SCHEMA_BASES=()
if [[ -n "${GITHUB_REPOSITORY_OWNER:-}" ]]; then
  SCHEMA_BASES+=("https://${GITHUB_REPOSITORY_OWNER}.github.io/greentic-types/schemas/v1")
fi
SCHEMA_BASES+=("${SCHEMA_BASES_DEFAULT[@]}")

SCHEMA_FILES=(
  "pack-id.schema.json"
  "component-id.schema.json"
  "flow-id.schema.json"
  "node-id.schema.json"
  "tenant-context.schema.json"
  "hash-digest.schema.json"
  "semver-req.schema.json"
  "redaction-path.schema.json"
  "capabilities.schema.json"
  "limits.schema.json"
  "telemetry-spec.schema.json"
  "node-summary.schema.json"
  "node-failure.schema.json"
  "node-status.schema.json"
  "run-status.schema.json"
  "transcript-offset.schema.json"
  "tools-caps.schema.json"
  "secrets-caps.schema.json"
  "otlp-keys.schema.json"
  "run-result.schema.json"
)

for schema_file in "${SCHEMA_FILES[@]}"; do
  downloaded_url=""
  tmp=$(mktemp)

  for base in "${SCHEMA_BASES[@]}"; do
    url="$base/$schema_file"
    echo "Checking schema $url"
    status=$(curl -sS -w "%{http_code}" -o "$tmp" "$url") || {
      echo "Failed to download $url" >&2
      continue
    }
    if [[ "$status" == "200" ]]; then
      downloaded_url="$url"
      break
    fi
    echo "Schema $url responded with HTTP $status"
  done

  if [[ -z "$downloaded_url" ]]; then
    echo "Schema $schema_file was not available on any configured host" >&2
    rm -f "$tmp"
    exit 1
  fi

  schema_id=$(python3 - "$tmp" <<'PY'
import json
import sys
from pathlib import Path

path = Path(sys.argv[1])
try:
    data = json.loads(path.read_text())
except json.JSONDecodeError as exc:
    print(f"Failed to parse {path}: {exc}", file=sys.stderr)
    sys.exit(1)

print(data.get("$id", ""))
PY
  )
  rm -f "$tmp"

  valid_id=false
  for base in "${SCHEMA_BASES[@]}"; do
    if [[ "$schema_id" == "$base/$schema_file" ]]; then
      valid_id=true
      break
    fi
  done

  if [[ "$valid_id" != true ]]; then
    echo "Schema $downloaded_url has mismatched \$id ('$schema_id')" >&2
    exit 1
  fi
  echo "✔ $downloaded_url"
done

