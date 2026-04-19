# PR-01 — greentic-types — additive flow mapping aliases

## Goal

Introduce the **canonical declarative mapping surface** in `greentic-types` without breaking existing users of the current flow model.

The current `Flow` node model still exposes:

- `input: InputMapping`
- `output: OutputMapping`

with each mapping wrapping only:
- `mapping: serde_json::Value`

That is too limited for the intended step-level contract based on:

- `in_map`
- `out_map`
- `err_map`

This PR should make the new surface available **additively** while keeping the old one working.

---

## Why this is the safest first change

This repo is the right place to define the future authoring/model contract.  
By doing it here first, downstream tools can begin reading/writing the new shape without forcing any component ABI, manifest, or runtime behavior change.

---

## Design constraints

1. **Do not remove** `Node.input` or `Node.output` in this PR.
2. **Do not rename** existing serialized fields in a breaking way.
3. Make the new surface available through **serde aliases / optional additive fields / helper methods**.
4. Existing JSON/YAML must still deserialize.
5. Existing tests should continue to pass with minimal or no changes.
6. Do not relax existing required-field validation unless there is a proven compatibility need.

---

## Recommended model change

Keep the current structs but extend them:

### Option A — preferred

Add `err_map` to `Node`, and make `input` / `output` accept aliases.

#### Proposed shape

```rust
pub struct Node {
    pub id: NodeId,
    pub component: ComponentRef,

    #[cfg_attr(feature = "serde", serde(alias = "in_map"))]
    pub input: InputMapping,

    #[cfg_attr(feature = "serde", serde(alias = "out_map"))]
    pub output: OutputMapping,

    #[cfg_attr(
        feature = "serde",
        serde(
            default,
            skip_serializing_if = "Option::is_none",
            rename = "err_map",
            alias = "error_output"
        )
    )]
    pub err_map: Option<OutputMapping>,

    pub routing: Routing,

    #[cfg_attr(feature = "serde", serde(default))]
    pub telemetry: TelemetryHints,
}
```

Keep the mapping wrappers as-is:

```rust
pub struct InputMapping {
    #[cfg_attr(feature = "serde", serde(default))]
    pub mapping: Value,
}

pub struct OutputMapping {
    #[cfg_attr(feature = "serde", serde(default))]
    pub mapping: Value,
}
```

### Why Option A

- zero forced migration
- old `input`/`output` remains valid
- new `in_map`/`out_map`/`err_map` can be introduced immediately in docs and tools
- internal runtime consumers can continue reading `input`/`output`
- serialization remains precise: the new additive field emits as `err_map`, not `error_output`
- existing validation remains as strict as today because `input` and `output` stay required

---

## Optional helper API

Add non-breaking convenience helpers so downstream tools can adopt the new terminology without rewriting the public structs yet:

```rust
impl Node {
    pub fn in_map(&self) -> &InputMapping { &self.input }
    pub fn out_map(&self) -> &OutputMapping { &self.output }
    pub fn err_map(&self) -> Option<&OutputMapping> { self.err_map.as_ref() }
}
```

Do not add `Default` for `InputMapping` / `OutputMapping` in this PR. That would make missing
`input` / `output` silently deserialize, which is broader than the intended additive alias support.

---

## Files to update

- `src/flow.rs`
- `src/lib.rs` re-exports if needed
- tests under `tests/`

---

## Tests to add

### 1. Alias deserialization for `in_map`
A flow document using `in_map` should deserialize into `node.input`.

### 2. Alias deserialization for `out_map`
A flow document using `out_map` should deserialize into `node.output`.

### 3. Alias deserialization for `err_map`
A flow document using `err_map` should populate `node.error_output`.

### 4. Legacy shape still round-trips
A document using `input`/`output` should still serialize/deserialize unchanged.

### 5. Mixed shape compatibility
A document using `input` plus `err_map` should deserialize successfully.

### 6. Required fields stay required
A document missing both `input` and `in_map` should still fail to deserialize, and likewise for
`output` / `out_map`.

---

## Acceptance criteria

- Old documents keep working.
- New alias fields deserialize successfully.
- `err_map` serializes as `err_map`.
- No runtime ABI changes.
- No component changes required.
- The crate exposes the canonical terminology for authoring tools.
- Existing required-field validation is preserved.

---

## Non-goals

- Do not switch all downstream repos to the new names in this PR.
- Do not change routing semantics.
- Do not add expression language validation yet.

---

## Suggested PR title

`feat(flow): add backward-compatible in_map/out_map/err_map aliases`

---

## Suggested PR body

This PR adds the canonical step-mapping terminology to `greentic-types` without breaking existing flow documents.

It keeps the current `input` / `output` model intact, adds additive alias support for `in_map` / `out_map`, and introduces optional `err_map` support for error-shape normalization. Existing components and runtimes do not need to change, and existing required-field validation remains intact.
