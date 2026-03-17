# Alignment Matrix

Legend:
- `A` aligned
- `P` partial/mixed
- `C` conflicting or missing
- `N/A` not assessable in workspace

Capabilities:
- `QA` setup/update/remove lifecycle
- `I18N` localization readiness
- `TENANT` tenant/team-first model
- `SIGN` signatures/attestation posture
- `CBOR` CBOR/WASM-first runtime IO
- `POLICY` policy + auditability hooks
- `EXT` extension model alignment
- `VAL` validator model alignment
- `META` pack/component metadata consistency
- `RUNTIME` runtime contract self-description consistency

| Repo | QA | I18N | TENANT | SIGN | CBOR | POLICY | EXT | VAL | META | RUNTIME |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| greentic-types | P | P | A | P | P | P | A | P | A | P |
| greentic-interfaces | P | P | A | P | P | P | P | P | P | P |
| greentic-component | P | A | P | P | A | P | P | P | P | A |
| greentic-flow | P | P | P | C | P | C | P | A | P | P |
| greentic-pack | P | C | P | A | A | P | P | A | P | P |
| greentic-runner | P | C | A | A | P | A | P | P | P | P |
| greentic-distributor-client | C | C | P | C | C | P | P | C | P | P |
| greentic-operator | P | C | P | P | C | P | C | P | P | P |
| greentic-secrets | C | C | A | P | C | A | P | P | P | P |
| greentic-messaging-providers | P | A | P | P | P | P | P | P | C | P |
| greentic-events-providers | P | C | P | P | C | P | P | P | P | P |
| greentic-qa | P | C | C | C | P | P | P | P | P | P |
| greentic-telemetry | C | C | P | C | C | P | C | C | C | P |
| greentic-mcp | C | C | P | C | C | P | P | C | P | P |
| greentic-oauth | P | C | A | A | C | A | P | P | P | P |
| greentic-deployer | C | C | P | P | P | P | P | P | P | P |
| greentic-session | C | C | A | C | C | P | C | C | P | P |
| greentic-state | C | C | A | C | C | P | C | C | P | P |
| greentic-config | N/A | N/A | N/A | N/A | N/A | N/A | N/A | N/A | N/A | N/A |

## Matrix highlights

Strongest areas:

- Multi-tenancy foundations: `greentic-types`, `greentic-runner`, `greentic-oauth`, `greentic-session`, `greentic-state`
- Signing in pack/runtime path: `greentic-pack`, `greentic-runner`
- Component CBOR/i18n direction: `greentic-component`

Weakest areas:

- QA lifecycle normalization across non-QA repos
- Unified CBOR-first posture (JSON remains dominant in several repos)
- Extension metadata consistency (`greentic.ext.provider` drift)
- Signature verification consistency outside pack/runner core

Most critical cross-cutting blockers:

- Operator not yet orchestrator-only (still provider-aware)
- Duplicate manifest/type ownership across pack toolchain
- Legacy protocol surfaces still active in runner/interfaces
