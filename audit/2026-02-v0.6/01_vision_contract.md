# v0.6.0 Vision Contract (North Star)

## North Star architecture

Core repos remain minimal and generic:

- `greentic-component`
- `greentic-flow`
- `greentic-pack`
- `greentic-runner`
- `greentic-operator`
- `greentic-mcp`
- `greentic-session`
- `greentic-distributor-client`

All other domains become capability extensions distributed as `.gtpack`:

- secrets
- oauth
- messaging
- events
- deployer
- telemetry
- state
- provider-specific capabilities

Operator is a dynamic orchestrator:

- resolves packs/components
- enforces policy
- runs QA lifecycle (`setup/update/remove`) through canonical QA contracts
- wires capabilities dynamically
- does not embed provider-specific setup logic

## Non-negotiables

1. QA lifecycle is canonical
- Every capability pack must expose setup/update/remove through QA contracts.
- `greentic-qa` is the single standard for setup/update/remove UX and payload semantics.
- No bespoke interactive setup in core repos.

2. Multi-tenant by default
- Every runtime/storage operation is scoped by `env + tenant + team (+ user where applicable)`.
- No hidden globals/defaults without explicit policy gate.
- Cross-tenant reads/writes are denied by default and auditable.

3. CBOR/WASM first
- Runtime envelopes and op payloads are canonical CBOR.
- JSON allowed only for human-facing tooling, docs, or explicit compatibility adapters.
- Pack/runtime contracts must be deterministic and hash-stable.

4. Signatures and supply chain are foundational
- Packs/components must have verifiable signatures and digest chains.
- Verification must be enforceable in strict policy mode with clear failure semantics.
- Attestations and policy checks are first-class, not optional add-ons.

5. Self-description and extension model
- Components and packs are self-describing (ops, schemas, capabilities, lifecycle hooks).
- Extensions and validators are structured, versioned, and schema-validated.
- Provider-specific logic belongs in extension packs, not core orchestrators.

6. i18n is first-class
- `i18n_id` and localization keys are mandatory in invocation context.
- User-facing prompts/messages/cards must be localizable by default.

7. Policy and auditability
- Policy hooks exist at orchestration and runtime boundaries.
- Security-sensitive operations (secret access, token minting, external calls, provider ops) are auditable.

## Canonical interface contract (future)

- `greentic-types`: canonical shared data structures, envelope formats, shared schema IDs, manifest structures, signing/policy DTOs.
- `greentic-interfaces`: canonical WIT contracts only; no duplicate or drifting type ownership.
- `greentic-pack`: pack assembly/verification on canonical types only.
- `greentic-runner`: executes canonical CBOR envelopes and policy-checked capability imports.
- `greentic-operator`: orchestrates packs and QA lifecycle via stable contracts, no provider hardcoding.

## Explicit anti-goals

- JSON-first runtime contracts in core execution paths
- Multiple co-equal lifecycle frameworks
- Duplicated pack/type/envelope models across repos
- “Operator knows everything” provider-specific code
- tenant/team defaults that silently broaden scope

## Compatibility posture

- Compatibility shims are temporary and isolated.
- Legacy surfaces must be explicitly marked, test-gated, and tracked with removal date/PR.
- New features ship only on canonical v0.6 line.
