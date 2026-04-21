# Audit: greentic-types Alignment for Embedded Manifest Metadata

## Scope

This audit reviews whether `greentic-types` is the right home for shared types related to:

- external `component.manifest.json` authoring input
- deterministic CBOR embedded into built Wasm
- `describe()` runtime self-description
- shared metadata consumed by pack/dev/runner/tooling flows

This is an audit only. It does not propose moving manifest parsing, validation, or canonicalization into this crate.

## Executive Recommendation

`greentic-types` should not own the full authoring manifest model for the planned embedded-manifest architecture.

Recommended option: **Option B, partial manifest projection types**, with an MVP nuance:

- **For MVP, no broad type move is required.**
- `greentic-component` should continue to own manifest file loading, schema validation, normalization, canonicalization, deterministic CBOR embedding, and build-time mismatch checks.
- `greentic-types` should remain focused on shared contract/projection types.
- If multiple repos need to read embedded metadata immediately after MVP, add **versioned embedded descriptor envelope/projection types** here rather than expanding `ComponentManifest` into a second canonical manifest owner.

In short:

- do **not** move full manifest ownership into `greentic-types`
- do **not** duplicate canonical manifest logic across `greentic-component` and `greentic-types`
- do allow `greentic-types` to carry stable, reduced, versioned metadata projections that multiple repos consume

## Current State

The crate already contains three distinct surfaces:

| Surface | Current owner in code | Current shape |
| --- | --- | --- |
| Component manifest-ish shared model | `src/component.rs` | `ComponentManifest` plus capabilities/profiles/configurators/resources/dev flows |
| Runtime `describe()` contract model | `src/schemas/component/v0_6_0/describe.rs` | `ComponentDescribe`, `ComponentInfo`, versioned `SchemaIr`-based operations |
| Generic CBOR wrappers/helpers | `src/envelope.rs`, `src/cbor/*`, `src/cbor_bytes.rs`, `src/schema_id.rs` | generic canonical CBOR encode/decode, `Envelope`, `CborBytes`, schema IDs |

The crate also already carries pack-oriented component metadata:

- component manifest index extension types in [`src/pack/extensions/component_manifests.rs`](/projects/ai/greentic-ng/greentic-types/src/pack/extensions/component_manifests.rs)
- component sources/resolution metadata in [`src/pack/extensions/component_sources.rs`](/projects/ai/greentic-ng/greentic-types/src/pack/extensions/component_sources.rs)
- capability-offer metadata in [`src/pack/extensions/capabilities.rs`](/projects/ai/greentic-ng/greentic-types/src/pack/extensions/capabilities.rs)

That means `greentic-types` already understands some component metadata, but today it does so in three different layers:

1. a manifest-shaped model
2. a runtime `describe()` model
3. pack/indexing metadata extensions

The architectural risk is not absence of manifest understanding. The risk is **drift between those layers if this crate starts owning full canonical manifest semantics.**

## Inventory

| Module | Type(s) | Purpose | Used by | Keep / Extend / Avoid |
| --- | --- | --- | --- | --- |
| `src/component.rs` | `ComponentManifest`, `ComponentProfiles`, `ComponentConfigurators`, `ComponentOperation`, `ResourceHints`, `ComponentCapabilities` and nested capability types | Shared component metadata structure used in pack/component schemas; includes authoring-only `dev_flows` | pack manifests, validation, schema export, tooling | **Keep with caution**. Do not promote to canonical authoring owner. Avoid growing it into full normalized manifest model. |
| `src/schemas/component/v0_6_0/describe.rs` | `ComponentDescribe`, `ComponentInfo`, `ComponentOperation`, `ComponentRunInput`, `ComponentRunOutput`, `RedactionRule`, `RedactionKind`, `schema_hash` | Versioned runtime self-description payload for component contracts | `describe()` contract, fixtures, schema/hash checks | **Keep**. This is the strongest current shared runtime contract surface. |
| `src/contracts/wit_map.rs` | `WitReturnSchema`, `WIT_RETURNS` | Static map from WIT exports to schema IDs/versions | contract validation/tests/tooling | **Keep**. Useful bridge, but not the payload model itself. |
| `src/schemas/common/schema_ir.rs` | `SchemaIr`, `AdditionalProperties` | Typed schema representation used by versioned describe/QA contracts | component/pack describe schemas | **Keep**. Good shared contract substrate. |
| `src/op_descriptor.rs` | `IoSchema`, `OpDescriptor`, `OpExample` | Older/generic self-describing operation helper using `SchemaSource` + example CBOR blobs | generic tooling/docs | **Keep but do not treat as canonical describe model**. |
| `src/envelope.rs` | `Envelope` | Generic canonical CBOR envelope with `kind`, `schema`, `version`, `body` | fixtures/tests/generic typed payload transport | **Keep**. Too generic to be the embedded component section contract by itself. |
| `src/cbor/canonical.rs` | canonical encode/decode/hash helpers | Deterministic canonical CBOR primitives | pack/component fixtures and helpers | **Keep**. Reusable foundation. |
| `src/cbor_bytes.rs` | `CborBytes`, `Blob` | Canonical CBOR byte wrapper and opaque blob helper | envelopes, QA/examples | **Keep**. Useful substrate, not enough as embedded metadata model alone. |
| `src/schema_id.rs` | `SchemaId`, `SchemaSource` | Stable schema identity/reference model over canonical CBOR | op descriptors, QA, schema refs | **Keep**. Relevant for embedded metadata that points at schemas. |
| `src/pack_manifest.rs` | `PackManifest`, `ExtensionRef`, `ExtensionInline`, helper methods for component sources/capabilities/provider extensions | Pack packaging and extension container model | pack authoring/validation/indexing | **Keep**. Packaging/indexing oriented, not runtime embedded-manifest contract. |
| `src/pack/extensions/component_manifests.rs` | `ComponentManifestIndexV1`, `ComponentManifestIndexEntryV1`, `ManifestEncoding` | Pack-level manifest file index metadata | pack tooling/indexing | **Keep**. Strong evidence that shared manifest-adjacent metadata belongs here when versioned and reduced. |
| `src/pack/extensions/component_sources.rs` | `ComponentSourcesV1`, `ComponentSourceEntryV1`, `ResolvedComponentV1`, `ArtifactLocationV1` | Pack-level component source/resolution/artifact metadata | pack resolution, validation, indexing | **Keep**. Good example of reduced cross-repo metadata, not canonical manifest ownership. |
| `src/pack/extensions/capabilities.rs` | `CapabilitiesExtensionV1` and nested types | Pack-level capability offer/setup metadata | pack/runtime/tooling coordination | **Keep**. Another good projection-style shared surface. |
| `src/schema.rs` | schema export fns including `component_manifest()` | Publishes JSON Schemas for shared types | external schema consumers/tooling | **Keep**, but note it increases public-surface pressure on `ComponentManifest`. |

## Classification of Existing Types

### Runtime contract oriented

- [`src/schemas/component/v0_6_0/describe.rs`](/projects/ai/greentic-ng/greentic-types/src/schemas/component/v0_6_0/describe.rs)
- [`src/contracts/wit_map.rs`](/projects/ai/greentic-ng/greentic-types/src/contracts/wit_map.rs)
- [`src/schemas/common/schema_ir.rs`](/projects/ai/greentic-ng/greentic-types/src/schemas/common/schema_ir.rs)

These are the most clearly versioned and runtime-facing surfaces in the crate.

### Shared metadata oriented

- [`src/envelope.rs`](/projects/ai/greentic-ng/greentic-types/src/envelope.rs)
- [`src/cbor/canonical.rs`](/projects/ai/greentic-ng/greentic-types/src/cbor/canonical.rs)
- [`src/cbor_bytes.rs`](/projects/ai/greentic-ng/greentic-types/src/cbor_bytes.rs)
- [`src/schema_id.rs`](/projects/ai/greentic-ng/greentic-types/src/schema_id.rs)
- [`src/op_descriptor.rs`](/projects/ai/greentic-ng/greentic-types/src/op_descriptor.rs)

These are substrate types. They help define stable payloads, but they are not themselves the embedded component manifest contract.

### Authoring manifest oriented

- [`src/component.rs`](/projects/ai/greentic-ng/greentic-types/src/component.rs)

`ComponentManifest` is explicitly manifest-shaped and even includes authoring-only `dev_flows` at [`src/component.rs:17`](/projects/ai/greentic-ng/greentic-types/src/component.rs#L17) and [`src/component.rs:76`](/projects/ai/greentic-ng/greentic-types/src/component.rs#L76). That makes it the most authoring-adjacent type currently in this crate.

### Packaging/indexing oriented

- [`src/pack_manifest.rs`](/projects/ai/greentic-ng/greentic-types/src/pack_manifest.rs)
- [`src/pack/extensions/component_manifests.rs`](/projects/ai/greentic-ng/greentic-types/src/pack/extensions/component_manifests.rs)
- [`src/pack/extensions/component_sources.rs`](/projects/ai/greentic-ng/greentic-types/src/pack/extensions/component_sources.rs)
- [`src/pack/extensions/capabilities.rs`](/projects/ai/greentic-ng/greentic-types/src/pack/extensions/capabilities.rs)

These types already model the sort of reduced, versioned metadata that multiple repos can share without taking ownership of canonical authoring semantics.

## Audit Question 1: What component-related model already lives here?

### Relevant findings

1. `greentic-types` already owns a manifest-shaped `ComponentManifest`.
   Evidence: [`src/component.rs:36`](/projects/ai/greentic-ng/greentic-types/src/component.rs#L36).

2. The crate already publishes `ComponentManifest` as a public JSON Schema.
   Evidence: [`src/schema.rs:77`](/projects/ai/greentic-ng/greentic-types/src/schema.rs#L77).

3. The crate also owns a separate versioned runtime `ComponentDescribe` contract.
   Evidence: [`src/schemas/component/v0_6_0/describe.rs:31`](/projects/ai/greentic-ng/greentic-types/src/schemas/component/v0_6_0/describe.rs#L31).

4. The crate already contains pack-level metadata for component manifests and resolved component artifacts.
   Evidence: [`src/pack/extensions/component_manifests.rs:14`](/projects/ai/greentic-ng/greentic-types/src/pack/extensions/component_manifests.rs#L14), [`src/pack/extensions/component_sources.rs:16`](/projects/ai/greentic-ng/greentic-types/src/pack/extensions/component_sources.rs#L16).

5. The crate already has generic deterministic CBOR and schema reference helpers that a future embedded descriptor model could reuse.
   Evidence: [`src/cbor/canonical.rs:39`](/projects/ai/greentic-ng/greentic-types/src/cbor/canonical.rs#L39), [`src/envelope.rs:10`](/projects/ai/greentic-ng/greentic-types/src/envelope.rs#L10), [`src/schema_id.rs:60`](/projects/ai/greentic-ng/greentic-types/src/schema_id.rs#L60).

### Conclusion

This crate is not empty space waiting for manifest understanding. It already contains:

- one authoring-adjacent manifest shape
- one runtime describe shape
- several shared metadata/package projection shapes

So the correct question is not "should it learn manifests at all?" but "which layer should it own without becoming a second canonical manifest implementation?"

## Audit Question 2: Is there already a shared type matching `describe()`?

### Yes, mostly

The current shared type matching the versioned `describe()` payload is:

- [`ComponentDescribe`](/projects/ai/greentic-ng/greentic-types/src/schemas/component/v0_6_0/describe.rs#L34)
- plus the WIT schema mapping in [`WIT_RETURNS`](/projects/ai/greentic-ng/greentic-types/src/contracts/wit_map.rs#L19)

Tests treat that schema as canonical for the contract:

- fixture roundtrip: [`tests/schema_fixtures_v0_6_0.rs:31`](/projects/ai/greentic-ng/greentic-types/tests/schema_fixtures_v0_6_0.rs#L31)
- schema hash verification: [`tests/schema_fixtures_v0_6_0.rs:43`](/projects/ai/greentic-ng/greentic-types/tests/schema_fixtures_v0_6_0.rs#L43)
- WIT mapping coverage: [`tests/schema_fixtures_v0_6_0.rs:88`](/projects/ai/greentic-ng/greentic-types/tests/schema_fixtures_v0_6_0.rs#L88)

### But it is not the same as `ComponentManifest`

`ComponentManifest` and `ComponentDescribe` overlap on:

- `id`
- `version`
- operations
- config schema
- some capability information

They diverge on purpose:

- `ComponentManifest` is authoring/package oriented, with `world`, `supports`, `profiles`, `configurators`, `resources`, and `dev_flows`
- `ComponentDescribe` is runtime contract oriented, with `role`, `display_name`, `provided_capabilities`, `required_capabilities`, typed `SchemaIr`, metadata bag, redactions, constraints, and stable `schema_hash`

### Duplication assessment

There is clear duplication in concept, not exact struct shape:

- manifest-side operation info in [`src/component.rs:164`](/projects/ai/greentic-ng/greentic-types/src/component.rs#L164)
- describe-side operation info in [`src/schemas/component/v0_6_0/describe.rs:49`](/projects/ai/greentic-ng/greentic-types/src/schemas/component/v0_6_0/describe.rs#L49)

That duplication is acceptable if the rule is:

- manifest is an input/projection source
- describe is the runtime output contract

It becomes dangerous only if both are treated as canonical sources of truth for the same semantics.

### WIT-generated bindings

This crate does not appear to contain WIT-generated Rust payload structs. It contains a static schema mapping table instead. That means the shared Rust-side contract is effectively the versioned schema structs, not generated interface bindings.

## Audit Question 3: What part of the manifest should `greentic-types` understand?

### Recommendation: Option B, partial manifest projection types

Option A is too weak as stated because the crate already contains manifest-facing types and pack metadata about component manifests.

Option C is the wrong direction because it would create two competing owners of canonical manifest semantics.

Option B fits the codebase best:

- `greentic-types` should understand **stable manifest-derived projections**
- it should **not** own the full authoring file model or canonicalization logic

### What belongs in the shared projection layer

A reduced shared projection is appropriate for fields that are:

- versioned
- read by multiple repos
- stable after build
- safe to derive from the canonicalized manifest

That likely includes:

- identity and semantic version
- ABI or WIT contract/world/version identity
- role/profile/capability summaries
- setup/lifecycle summary
- operation list and schema references or hashes
- digest/hash references for embedded or sidecar metadata
- metadata useful for inspect/doctor/indexing

### What should stay out

`greentic-types` should not own:

- manifest file loading
- JSON schema validation of `component.manifest.json`
- normalization rules
- canonicalization rules
- deterministic CBOR byte layout decisions for the full authoring manifest
- build-time equivalence or mismatch checks between source manifest and embedded artifact

Those are all build-system semantics and belong in `greentic-component`.

## Audit Question 4: Should the embedded CBOR envelope live here?

### Recommendation

For MVP: **no immediate change required**.

Current generic helpers are close, but not sufficient as the final shared embedded section contract:

- generic envelope: [`src/envelope.rs:10`](/projects/ai/greentic-ng/greentic-types/src/envelope.rs#L10)
- canonical CBOR primitives: [`src/cbor/canonical.rs:39`](/projects/ai/greentic-ng/greentic-types/src/cbor/canonical.rs#L39)
- byte wrapper: [`src/cbor_bytes.rs:9`](/projects/ai/greentic-ng/greentic-types/src/cbor_bytes.rs#L9)

These support deterministic encoding and decoding, but they do **not** define:

- a component-specific embedded section payload schema
- a section name/identifier
- envelope version semantics for embedded descriptors
- extraction helpers tied to Wasm custom-section usage

### Recommended boundary

- If only `greentic-component` reads/writes the embedded section initially, keep the MVP implementation local there.
- If `greentic-pack`, `greentic-runner`, `greentic-dev`, or inspection tooling all need the same embedded payload quickly, then add to `greentic-types`:
  - versioned embedded descriptor envelope struct(s)
  - version enum/constants
  - decode helpers from bytes to typed payload
  - maybe validation helpers for envelope version/schema IDs

What should still remain outside `greentic-types` even then:

- Wasm custom section insertion logic
- build pipeline mismatch detection
- manifest canonicalization rules used to produce the embedded payload

## Audit Question 5: Boundary between `greentic-component` and `greentic-types`

### Recommended ownership

`greentic-component` should own:

- `component.manifest.json` file loading
- schema validation of authoring manifest
- normalization and defaults
- canonicalization of the authoring manifest
- deterministic CBOR embedding into Wasm
- build-time mismatch detection between source manifest, derived projections, and embedded artifact

`greentic-types` should own:

- versioned shared projections used across repos
- versioned `describe()` payload models
- generic canonical CBOR/schema-reference helpers
- optional embedded-descriptor envelope types if more than one repo consumes them
- pack/indexing/inspection metadata models shared across tooling

### Does this fit the current codebase?

Yes, mostly.

The current codebase already supports that split:

- runtime contract types are versioned and separated under `src/schemas/component/v0_6_0/`
- pack/indexing metadata lives in dedicated extension modules
- generic CBOR helpers are already factored out

The one awkward area is `ComponentManifest` in [`src/component.rs`](/projects/ai/greentic-ng/greentic-types/src/component.rs), because it is more authoring-adjacent than the rest of the crate and includes `dev_flows`. That does not force a redesign today, but it is evidence that the crate should **stop short of deepening manifest ownership further**.

## Audit Question 6: Would adding more manifest understanding reduce or increase drift?

### It depends on what is added

Adding **reduced, versioned projections** would reduce drift.

Examples:

- a shared embedded descriptor payload
- a stable manifest-derived metadata projection for inspect/doctor/indexing
- common decode helpers for embedded CBOR

Adding a **full canonical manifest model** would increase drift.

Why:

- `greentic-component` would still need to own build-time manifest logic
- downstream repos would start depending on `greentic-types` for a second copy of manifest semantics
- normalization/canonicalization changes would need to stay perfectly synchronized across crates

That is exactly the two-owners problem this PR is trying to avoid.

## Agreement Analysis Across the Three Surfaces

| Surface | Owner | Purpose |
| --- | --- | --- |
| external `component.manifest.json` | `greentic-component` | authoring/build input |
| embedded CBOR descriptor | build artifact | passive artifact metadata |
| `describe()` output | `greentic-interfaces` contract, modeled here in `greentic-types` | runtime/contract self-description |

### What `greentic-types` should model across these surfaces

It should model the **stable shared overlap**, not the full authoring source.

That overlap is:

- versioned runtime `describe()` payloads
- stable manifest-derived projection payloads used by multiple repos
- embedded envelope types only when shared reuse exists
- schema IDs, canonical CBOR wrappers, and decoding helpers

It should not model:

- authoring-only source layout
- canonicalization procedure
- build-time transformation rules

## Gaps and Overlaps

### Existing overlap

- `ComponentManifest` and `ComponentDescribe` both express component identity/version/operations/configuration in different forms.
- pack extension metadata already references manifest files and resolved artifacts.
- generic `Envelope` overlaps partially with what an embedded descriptor envelope might need.

### Missing shared piece

What is not yet present is a **component-specific, versioned embedded descriptor model** distinct from:

- full authoring manifest
- runtime `describe()` output
- pack manifest indexing

That is the likely missing shared abstraction if multiple repos need to consume embedded metadata.

## Follow-up PR Suggestions

1. Do not add full manifest canonicalization logic to `greentic-types`.
2. Treat the existing `ComponentManifest` as a shared manifest-shaped DTO, not the owner of canonical manifest semantics.
3. If cross-repo embedded-section consumers appear, add a new versioned module for something like:
   - `EmbeddedComponentDescriptorV1`
   - `EmbeddedComponentDescriptorEnvelopeV1`
   - decode/validate helpers from canonical CBOR bytes
4. Define the embedded projection from manifest-derived data, but keep the derivation algorithm in `greentic-component`.
5. Document a one-way relationship:
   - authoring manifest -> canonicalized/validated in `greentic-component`
   - reduced embedded/shared projection -> modeled in `greentic-types`
   - `describe()` output -> runtime contract, modeled separately in `greentic-types`
6. Avoid extending `src/component.rs` with more authoring-only fields unless they are already proven to be multi-repo shared contracts.

## Final Recommendation

The current codebase supports the following conclusion:

- `greentic-types` should **not** own the full authoring manifest model
- `greentic-component` should remain the canonical owner of manifest parsing, validation, normalization, canonicalization, and embedding
- `greentic-types` should continue to own shared runtime contract models and reduced metadata projections
- for MVP, embedded-manifest implementation can stay local to `greentic-component`
- if shared consumers emerge immediately, `greentic-types` is the right place for a **versioned embedded descriptor projection and decode helpers**, not for the full canonical manifest

That is the cleanest way to reduce drift without creating two competing manifest authorities.
