# RFC: `QuestionKind::Table` for repeating structured data

**Status:** Draft — pending review
**Date:** 2026-05-02
**Author:** drafted by Claude under Bima's direction
**Audience:** `greentic-types` maintainers, `greentic-setup` maintainers, `greentic-pack` (`packc qa`) maintainers, anyone shipping a `*_json` setup question today

## 0. TL;DR

Pack-side QA only knows four `QuestionKind`s — `text`, `choice`, `number`, `bool`. So when a pack needs to collect a list of structured rows (links, OAuth providers, allow rules) the pack author falls back to "type a JSON array into a textarea", which is exactly the demo pain that triggered this RFC.

The wizard renderer (`qa-spec` crate) **already** has a `QuestionType::List { min_items, max_items, fields: Vec<QuestionSpec> }` variant with full validation; the pack→FormSpec bridge (`greentic-setup/src/qa/bridge.rs`) just doesn't emit it because the pack-side enum can't represent it. This RFC proposes lifting that asymmetry by adding a `Table` variant on the pack `QuestionKind` enum, mapping it through the bridge to the existing `List` shape, and teaching the CLI prompts and the persistence path to handle structured arrays natively.

## 1. Problem

### 1.1 Concrete operator pain

Today the only way to ask an operator for repeating structured data is `kind: string` whose value is a hand-typed JSON array. Two demo reviews (3Point/M5 Deep Research) flagged this as too sharp.

**Example A — `nav_links_json`** (`/home/bima-pangestu/Works/greentic/greentic-messaging-providers/packs/messaging-webchat-gui/assets/setup.yaml:239-283`).

What the operator types today:

```yaml
nav_links_json: |
  [{"label":{"en":"Help","id":"Bantuan","de":"Hilfe"},"url":"/help"},
   {"label":{"en":"Deep Research","id":"Riset Mendalam"},
    "num":"M5","url":"https://m5.example.com",
    "tooltip":{"eyebrow":{"en":"Module 5","id":"Modul 5"},
               "title":{"en":"When One Agent Isn't Enough"},
               "lede":{"en":"Use a <strong>planner</strong>..."}}}]
```

The pack ships a 45-line help block (`setup.yaml:245-283`) that is essentially a mini-schema rendered in prose, ending with the disclaimer "*A richer in-wizard editor (table/row-style input) is tracked as a follow-up — current QA spec only supports text/bool/choice/number kinds.*" That disclaimer is precisely what this RFC retires.

What the operator could type with a `Table` kind:

```yaml
nav_links:
  - label: { en: Help, id: Bantuan, de: Hilfe }
    url: /help
  - label: { en: Deep Research, id: Riset Mendalam }
    num: M5
    url: https://m5.example.com
    tooltip:
      eyebrow: { en: Module 5, id: Modul 5 }
      title:   { en: When One Agent Isn't Enough }
      lede:    { en: "Use a <strong>planner</strong>..." }
```

…with the wizard validating per row, per column.

**Example B — `oauth_providers`** (`messaging-provider-webchat-gui/src/lib.rs:306-390`, `compose_oauth_providers`). The pack hides JSON-array sprawl by exploding the array into a flat namespace (`oauth_enable_google`, `oauth_google_client_id`, `oauth_google_client_secret`, `oauth_microsoft_client_id`, … `oauth_custom_label`, `oauth_custom_auth_url`, `oauth_custom_token_url`, `oauth_custom_scopes`). The pack's `apply-answers` op then re-assembles a JSON array. This works for a known finite set (Google/Microsoft/GitHub + one custom slot) but doesn't scale: there's no way to register a second custom OIDC provider without shipping new pack code.

**Example C — operator-level `tenants` and `allow paths`** (`greentic-setup/src/plan.rs:316-336`). The setup engine internally models `Vec<TenantSelection { tenant_id, team, allow_paths: Vec<String> }>`, but the wizard QA spec exposes them as flat string ids (`operator.tenants`, `operator.allow.paths`) with bespoke parsing. A native `Table` kind would let this surface look like the underlying data.

### 1.2 Survey of repeating-shape questions in the workspace

Authoritative survey across `/home/bima-pangestu/Works/greentic/**/setup.yaml` and the operator-level wizard spec in `greentic-setup`:

| # | Source | Question id | Shape today | Notes |
|---|---|---|---|---|
| 1 | `messaging-webchat-gui/assets/setup.yaml:239` | `nav_links_json` | `kind: string` carrying a JSON array of `{ label, url, external?, num?, tooltip? }` | Multi-locale labels, optional nested `tooltip` object. The trigger for this RFC. |
| 2 | `messaging-provider-webchat-gui/src/lib.rs:306` | `oauth_providers` (composed) | Synthesised from 11 flat string fields in `setup.yaml`; written out as a JSON array | Capped at 3 well-known + 1 custom; can't add second custom. |
| 3 | `messaging-provider-webchat/src/describe.rs:567` | `oauth_providers` (config schema field) | Documented as "JSON array of configured OAuth providers" | Mirrors #2 at the schema level. |
| 4 | `greentic-setup/src/plan.rs:327` | `operator.tenants` | Flat string in the QA prompt; `Vec<TenantSelection>` internally | Each tenant has `tenant_id`, optional `team`, plus owns its own `allow_paths`. |
| 5 | `greentic-setup/src/plan.rs:332` | `operator.allow.paths` | Flat string (`PACK[/FLOW[/NODE]]`) parsed bespoke | Conceptually `Vec<{ pack, flow?, node? }>`. |
| 6 | `greentic-setup/src/plan.rs:322` | `operator.packs.refs` | Flat string (catalog + custom refs) | Conceptually `Vec<{ name, version?, source? }>`. |

Six existing pain points across two repos — five of them in `greentic-setup` / `greentic-messaging-providers` directly, and the sixth is `oauth_providers` which appears twice (composer + schema).

### 1.3 Why this compounds

`nav_links_json` started as `[{label, url}]`. Two iterations later it accepts: locale-keyed objects on `label` (5 locales possible), an optional `num`, an optional `tooltip` with three sub-fields (`eyebrow`, `title`, `lede`), each of which can independently be a string OR a locale-keyed object, and `lede` may contain inline HTML. That schema has eight optional axes. Asking an operator to keep that consistent in a JSON textarea is a support-load liability and a correctness hazard.

## 2. Goals / Non-goals

### Goals

- **G1** — Allow pack authors to declare a question whose answer is `Vec<Map<String, Value>>` (a table of structured rows) without forcing operators to author JSON.
- **G2** — Keep on-the-wire and on-disk compatibility with existing `*_json` string questions; existing packs must keep working byte-for-byte.
- **G3** — Reuse the `qa-spec` crate's existing `QuestionType::List` / `ListSpec` machinery (validation, render-payload, visibility) instead of inventing a parallel rendering path.
- **G4** — Provide a row-by-row CLI wizard experience (`add another? [y/N]`) and a row-array path through the answer-file (headless) flow.
- **G5** — Caller code (e.g. `sync_nav_links_to_tenant_config`) takes the array directly from `serde_json::Value::Array`, no `from_str` step.

### Non-goals

- **NG1** — No nested tables (table-inside-table). Each column is a scalar (string / bool / choice / number).
- **NG2** — No full form-builder DSL (no expressions on column visibility within a row, no per-column `visible_if`). If we need row-level conditional columns later, that's a follow-up RFC.
- **NG3** — No server-side semantic validation beyond column types + per-column `Constraint` (pattern, min/max). Domain validation (e.g. "url must be reachable") stays in the pack's `apply-answers` op.
- **NG4** — No web-admin UI table editor in this RFC's first phase. CLI + headless answers file ships first; web UI is tracked as a separate work item (see §5.2 and §7).
- **NG5** — No migration of `oauth_providers` away from its current composed-fields shape. `Table` would let us simplify it later, but the demo-blocking question is `nav_links`.

## 3. Proposed schema

### 3.1 Pack-side `QuestionKind` (`greentic-types`)

Add `Table` as an additive variant on **both** pack and component `QuestionKind` enums (the same crate exposes two — one for `pack/v0_6_0/qa.rs`, one for `component/v0_6_0/qa.rs` — and they need to stay in lockstep so a component's `qa-spec` op can return Tables too).

```rust
// greentic-types/src/schemas/pack/v0_6_0/qa.rs
// (mirror in component/v0_6_0/qa.rs)
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case", tag = "type"))]
pub enum QuestionKind {
    Text,
    Choice { options: Vec<ChoiceOption> },
    Number,
    Bool,
    /// Repeating structured rows. Each row is a `BTreeMap<String, Value>`
    /// keyed by `TableColumn::key`. The wizard renders an add-row / remove-row
    /// editor; persistence stores the answer as a JSON array of objects.
    Table {
        columns: Vec<TableColumn>,
        #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
        min_rows: Option<u16>,
        #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
        max_rows: Option<u16>,
        /// Label for the "add row" button / CLI prompt. Default: i18n key
        /// `qa.table.add_row` resolved by the renderer.
        #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
        add_label: Option<I18nText>,
        /// Message shown when the table has zero rows.
        #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
        empty_label: Option<I18nText>,
    },
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub struct TableColumn {
    /// JSON object key under which the value is stored on each row.
    pub key: String,
    /// Header label shown to the user.
    pub label: I18nText,
    /// Column data type. Narrowed enum — no nested Table.
    pub kind: ColumnKind,
    /// Whether this column must be filled on every row.
    #[cfg_attr(feature = "serde", serde(default))]
    pub required: bool,
    /// Inline help under the column input.
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub help: Option<I18nText>,
    /// Placeholder text for empty inputs.
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub placeholder: Option<I18nText>,
    /// Optional default value for new rows.
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub default: Option<Value>,
    /// Optional regex / numeric / length constraint.
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub constraint: Option<ColumnConstraint>,
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case", tag = "type"))]
pub enum ColumnKind {
    String,
    Bool,
    Number,
    Integer,
    Choice { options: Vec<ChoiceOption> },
}
```

### 3.2 Why `ColumnKind` is a separate enum, not recursive `QuestionKind`

Two options were considered:

- **Option A** — Recursive: `Table { columns: Vec<Question> }`, reusing the existing `Question` (and therefore `QuestionKind`) inside.
- **Option B** — Narrowed: `Table { columns: Vec<TableColumn> }` with `ColumnKind` excluding `Table` itself.

**Recommend Option B.** Reasons:

1. **Validation surface** — Recursive nesting requires every renderer (CLI prompt loop, Adaptive Card builder, future web table component) to handle arbitrary depth. The `qa-spec` `validate_list` already deliberately doesn't recurse beyond one level (`validate.rs:176`). Mirroring that constraint in the type system makes the contract obvious.
2. **JSON shape ambiguity** — `Question` carries `visible_if`, `skip_if`, `default: Value`. Most of those don't make sense for a column inside a row (you can't conditionally hide a column from row 3 only). Putting them on `TableColumn` would mean documenting "these fields are ignored on columns".
3. **Bridge symmetry** — `qa-spec::ListSpec.fields: Vec<QuestionSpec>` is technically recursive (a `QuestionSpec.list` could contain another list), but the renderer treats this as undefined behaviour. We don't want pack authors discovering that by accident.
4. **NG1** is already a goal. Encoding it in the type system is cheap.

If a future RFC genuinely needs nested tables (e.g. tenants → teams), it can promote `ColumnKind::Table { … }` then. Adding a variant is additive; removing one is not.

### 3.3 Wire format — YAML and JSON

**Pack `setup.yaml` author-time form (proposed, after Table lands):**

```yaml
- name: nav_links
  title: { key: webchat-gui.qa.setup.nav_links }
  kind:
    type: table
    min_rows: 0
    max_rows: 16
    add_label: { key: webchat-gui.qa.setup.nav_links.add_row }
    empty_label: { key: webchat-gui.qa.setup.nav_links.empty }
    columns:
      - key: label
        label: { key: webchat-gui.qa.setup.nav_links.label }
        kind: { type: string }
        required: true
        help:  { key: webchat-gui.qa.setup.nav_links.label.help }
      - key: url
        label: { key: webchat-gui.qa.setup.nav_links.url }
        kind: { type: string }
        required: true
        constraint: { pattern: "^/|^https?://" }
      - key: external
        label: { key: webchat-gui.qa.setup.nav_links.external }
        kind: { type: bool }
        default: false
      - key: num
        label: { key: webchat-gui.qa.setup.nav_links.num }
        kind: { type: string }
        placeholder: { key: webchat-gui.qa.setup.nav_links.num.placeholder }
  required: false
  group: Branding
```

Note: the i18n-keyed `label` and `tooltip` sub-fields from the original are intentionally **not** modelled as nested Tables. They stay as JSON sub-objects inside the cell value (`label: { en: …, id: … }`). Locale-keyed string editing is a separate UX concern (a keyed-map kind, future RFC). For v1 the operator types `{"en":"Help","id":"Bantuan"}` into the `label` cell — already a big improvement over typing the whole array.

**On-disk answer (CBOR / JSON, identical shape to today's `*_json` decoded form):**

```json
{
  "nav_links": [
    {"label": {"en": "Help", "id": "Bantuan"}, "url": "/help"},
    {"label": {"en": "Deep Research"}, "num": "M5", "url": "https://m5.example.com"}
  ]
}
```

**FormSpec emitted by the bridge** (existing `qa-spec` shape, no changes needed):

```json
{
  "id": "nav_links",
  "type": "list",
  "title": "Top-menu nav links",
  "list": {
    "min_items": 0,
    "max_items": 16,
    "fields": [
      { "id": "label", "type": "string", "title": "Label", "required": true },
      { "id": "url",   "type": "string", "title": "URL",   "required": true,
        "constraint": { "pattern": "^/|^https?://" } },
      { "id": "external", "type": "boolean", "title": "Open in new tab" },
      { "id": "num", "type": "string", "title": "Number prefix" }
    ]
  }
}
```

### 3.4 Backwards compatibility

- Existing `kind: string` questions whose answer is a JSON array string keep working untouched. The new `Table` kind is opt-in per question.
- Adding a variant to a `serde(tag = "type")`-encoded enum is **not** wire-compatible for older deserialisers — an older `greentic-types` deserialising a pack manifest containing `{"type":"table",…}` will fail with "unknown variant". This is the migration risk; see §6.
- Existing component `QuestionKind::InlineJson { schema }` (`component/v0_6_0/qa.rs:191`) stays. `Table` is the structured-form alternative; `InlineJson` remains the escape hatch for arbitrary JSON Schema with no row semantics.

## 4. Persistence

### 4.1 Today

`persist_qa_secrets` (`greentic-setup/src/qa/persist.rs:25-85`) calls `value_to_text(value)` on every answer (`persist.rs:320-325`) which on a `Value::String` carrying `"[{\"label\":\"Help\"}]"` writes that exact string into the dev secret store. Downstream callers like `sync_nav_links_to_tenant_config` then do `serde_json::from_str(answer)` to get back the array.

### 4.2 With Table

The wizard collects `Value::Array(Vec<Value::Object>)` for a Table answer. `value_to_text` already handles non-string values via `other.to_string()`, which serialises an `Array` to compact JSON — so the on-disk text representation is identical to what the operator types today. Downstream callers can switch to:

```rust
let nav_links: Vec<NavLink> =
    serde_json::from_value(config.get("nav_links").cloned().unwrap_or(Value::Array(vec![])))?;
```

instead of `serde_json::from_str(answer_string)`. Cleaner: one `from_value` instead of `from_str` then revalidate.

For a smoother migration, `persist_qa_secrets` can keep writing the JSON-text form to the dev store unchanged, and the read path (`tenant_config.rs`) can accept either a JSON string or a JSON array (it already does for `oauth_providers` — see `lib.rs:385-389` `compose_oauth_providers`). No persistence-format break.

## 5. Wizard rendering

Three rendering targets. Each is enumerated below with crate ownership.

### 5.1 CLI interactive wizard — `greentic-setup`

**Owner:** `greentic-setup/src/qa/prompts.rs`.

`ask_form_spec_question` (`prompts.rs:124`) currently dispatches by `QuestionType` and only handles scalar types (`String/Number/Boolean/Enum`). Add a `QuestionType::List` arm that:

1. Reads the question's `ListSpec.fields`.
2. Loops: print "Row 1 / N" header, prompt each field in order via the existing `ask_form_spec_question` recursion (treating each field as a one-off scalar question), accumulate into a `Map<String, Value>`.
3. After each row: prompt "Add another? [y/N]". Stop when the operator answers no, when `max_items` is reached, or when the operator hits Ctrl-D (and at least `min_items` rows have been added).
4. Re-validate on completion using the existing `qa-spec::validate_list` so the FormSpec validator stays the source of truth.

Render contract:

```text
  Top-menu nav links (optional)
  Optional list of links rendered in the topbar.

  Row 1 of (any)
    Label (required)
      > Help
    URL (required)
      > /help
    Open in new tab (optional) [yes/no]
      > no
    Number prefix (optional)
      > 

  Add another row? [y/N] y

  Row 2 of (any)
    Label (required)
      > {"en":"Deep Research","id":"Riset Mendalam"}
  …

  Add another row? [y/N] n
```

### 5.2 Web admin UI — TBD; no FormSpec table editor in the workspace today

**Owner:** TBD. The closest candidate in the workspace is `greentic-designer-admin/web` (React/Vite SPA), but its `Setup.tsx` (`web/src/pages/Setup.tsx`) is the first-operator bootstrap form, not a generic FormSpec renderer. The pack-setup wizard for end users renders via Adaptive Cards (`render_qa_card` in `greentic-setup/src/qa/wizard.rs:92`) from the host-side card REPL. Adaptive Cards 1.3 has no native repeating-row container — the renderer would need to either:

- emit a synthetic per-row `Container` block (one card section per row, "add row" as an `Action.Submit` with id `nav_links.__add_row`), driven by progressive multi-card flows; or
- fall back to a single `Input.Text` showing the JSON form (i.e. degrade to today's UX) when the surface can't render a table.

Recommendation: ship CLI + headless first (§5.1, §5.3); flag the web/AC table editor as a follow-up so it doesn't block fixing the demo-blocking pain. See §7.

### 5.3 Headless / answer-file path

**Owner:** `greentic-setup/src/setup_input.rs` (`SetupInputAnswers`) and `greentic-pack/crates/packc/src/cli/qa.rs` (the `packc qa apply` path).

The answer file already accepts native JSON arrays — the existing `*_json` string is the operator-side workaround, not a constraint of the file format. With Table the file becomes:

```yaml
# answers.yaml
messaging-webchat-gui:
  nav_links:
    - { label: { en: Help }, url: /help }
    - { label: { en: Deep Research }, num: M5, url: https://m5.example.com }
```

`validate_answers_against_form_spec` (`wizard.rs:126`) routes to `qa-spec::validate_list` automatically once the FormSpec carries the new `list` field. No new code in the headless path beyond making sure the bridge emits `QuestionType::List` when the source question is `kind: table`.

### 5.4 Renderer ownership summary

| Surface | Crate | New code? |
|---|---|---|
| Pack `QuestionKind::Table` enum + serde | `greentic-types` | Yes (the variant + `TableColumn`/`ColumnKind`). |
| Component `QuestionKind::Table` mirror | `greentic-types` (`component/v0_6_0/qa.rs`) | Yes (mirror variant). |
| Pack-author lint / `packc qa` | `greentic-pack/crates/packc/src/cli/qa.rs` | Map `Table` through to component-side `Table` when synthesising aggregate specs. |
| Pack→FormSpec bridge | `greentic-setup/src/setup_to_formspec.rs` + `qa/bridge.rs` | New: emit `QuestionType::List` with `ListSpec` populated from `TableColumn`s. |
| CLI prompt loop | `greentic-setup/src/qa/prompts.rs` | New: `QuestionType::List` arm. |
| Adaptive Card render | `qa-spec::render_card` | Already supports `List` (best-effort); revisit when web/AC table editor lands. |
| Persistence | `greentic-setup/src/qa/persist.rs` | None — `value_to_text` already serialises arrays to JSON text. |
| Tenant-config sync | `greentic-setup/src/tenant_config.rs` | Switch consumer from `from_str(answer_string)` to `from_value(answer_value)`. |

## 6. Migration path

### Phase 1 — `greentic-types`: ship the variant

Add `Table` variant to both pack and component `QuestionKind`. Bump `greentic-types` minor version. Additive enum variant; existing serializers continue to round-trip unchanged. Add a CBOR round-trip test and a JSON round-trip test mirroring the existing `serde_accepts_upgrade_and_emits_update_*` tests in `qa.rs`.

### Phase 2 — Wizard renderer support

In `greentic-setup`:

- Extend `setup_to_formspec.rs` to translate `QuestionKind::Table { columns, … }` into `QuestionType::List` + `ListSpec`.
- Extend `qa/prompts.rs::ask_form_spec_question` with a `List` arm.
- Add unit tests to `wizard.rs` mirroring the existing `validates_required_answers` style: row-required-column missing, row count below `min_rows`, row count above `max_rows`, headless answer-file path with array.

### Phase 3 — Canary migration

Migrate `nav_links_json` in `messaging-webchat-gui/assets/setup.yaml` to a `kind: table` question. **Keep** the legacy `nav_links_json: kind: string` question for **one release**, marked deprecated in help text, so existing answer files keep loading. The pack's `apply-answers` op reads either:

```rust
let links = answers.get("nav_links")
    .or_else(|| answers.get("nav_links_json").and_then(|v| /* parse JSON string */))
    .unwrap_or(Value::Array(vec![]));
```

After one release cycle, drop `nav_links_json`.

### Phase 4 — Wider migration

Migrate the remaining surveyed questions (table from §1.2):

- `oauth_providers` composer → `Table` (collapses 11 flat fields to 1 table; lifts the "1 custom only" cap).
- `operator.tenants` / `operator.allow.paths` / `operator.packs.refs` in `greentic-setup/src/plan.rs` → Tables. This is the larger lift because those questions feed bespoke parsers; they want plan-builder simplification in the same PR.

### 6.1 Cross-version compatibility risk

`#[serde(tag = "type")]` enums fail on unknown variants by default. Concretely: a runner pinned to `greentic-types 0.6.x` (without `Table`) reading a pack manifest produced against `greentic-types 0.7.x` (with `Table`) will fail to deserialise the pack's QA spec. The same applies to the dev-machine ↔ store-server ↔ runner-host split if they upgrade independently.

Mitigations, in increasing order of effort:

1. **Coordinate version bumps** (cheapest, today's pattern). All `greentic-*` Rust crates already pin to Rust 1.95 + a synchronised workspace version; ship Phase 1 and Phase 2 before any pack uses Phase 3. Document the floor in `greentic-types/CHANGELOG.md` + `greentic-pack/docs/`.
2. **Serde fallback** — derive a manual `Deserialize` that tolerates unknown variants and downgrades them to a transparent `QuestionKind::Unknown { raw: Value }`. Older callers can still iterate; they just can't render a Table. Useful if the runner upgrade lags pack release.
3. **Pack manifest version negotiation** — `pack.manifest.cbor` already carries a schema version; refuse to load a pack whose QA spec uses a kind newer than the runner supports, with a clear error. This is what the existing component `QuestionKind::AssetRef` migration relied on.

Recommend (1) for v1 + (3) as belt-and-braces. (2) is overkill given how tightly the workspace versions move.

## 7. Open questions

- **Web admin / Adaptive Card table editor.** No table-aware FormSpec renderer exists in the workspace today (`greentic-designer-admin/web/src/pages/Setup.tsx` is the operator-bootstrap form, not a generic wizard). The pack setup wizard's web surface is Adaptive Cards (`render_qa_card`), which has no native repeating container. Needs a dedicated design doc once CLI + headless lands. Open question: do we ship a custom `Action.Submit` round-trip (one card per add-row) or a wholly new SPA-side renderer reading FormSpec JSON directly?
- **`serde_yaml_bw` round-trip.** The `setup.yaml` parser in `greentic-setup` (`setup_to_formspec`) uses `serde_yaml_bw`. Need to confirm that nested `kind: { type: table, columns: […] }` round-trips cleanly without anchor / merge-key surprises. Likely fine but warrants a fixture test in Phase 1.
- **`qa-spec` external crate (0.5/0.6.0-dev) version policy.** `greentic-setup` depends on `qa-spec = "0.5"` from crates.io. The `List` machinery already exists there; no qa-spec bump needed for the renderer. But if Phase 4 wants per-column `visible_if` (NG2 follow-up), that's a `qa-spec` change too.
- **i18n key conventions for column labels.** Should `TableColumn.label` follow the same `<provider>.qa.setup.<question>.<column>` namespacing as today's flat questions, or do we introduce `<provider>.qa.setup.<question>.columns.<column>`? Maarten directives apply for webchat-gui — `nav_links` columns should follow whatever convention the existing `webchat-gui.qa.setup.*` keys use.
- **`greentic-runner` consumer.** Runner is `.git`-only on this machine; can't read its source locally. Need to confirm it deserialises pack QA specs (it shouldn't; it consumes flows). If it does, it's a Phase-1 affected crate.

## 8. Effort estimate

| Repo | Work | Days |
|---|---|---|
| `greentic-types` | `Table` variant on pack + component `QuestionKind`; `TableColumn` / `ColumnKind` types; CBOR + JSON round-trip tests; CHANGELOG. | 1.0 |
| `greentic-types` | `adapters/component_v0_5_0_to_v0_6_0.rs` — decide whether legacy `Table` can exist (it can't; emit unchanged) and add a guard test. | 0.25 |
| `greentic-pack` (`packc`) | Aggregate-spec stitching — `cli/qa.rs` already iterates `PackQuestionKind`; add a `Table` arm; lint that `min_rows ≤ max_rows`; doctor check that column keys are unique. | 0.75 |
| `greentic-setup` | `setup_to_formspec` translation `Table → ListSpec`. | 0.5 |
| `greentic-setup` | `qa/prompts.rs` — `QuestionType::List` interactive arm + tests for required-column / row-count bounds / Ctrl-D-with-min-rows. | 1.5 |
| `greentic-setup` | Persistence + `tenant_config.rs` consumer switch from `from_str` to `from_value`; preserve legacy-path fallback. | 0.5 |
| `messaging-webchat-gui` (canary) | Rewrite `nav_links_json` → `nav_links` Table in `setup.yaml`; keep deprecated alias for one release; update i18n keys; update `assets/webchat-gui` consumer if any. | 0.75 |
| `greentic-e2e` | New fixture `wizard-answers/webchat-nav-links-table.json`; expect-script that drives the row-by-row prompt. | 1.0 |
| Docs | `greentic/docs/00-start-here.md` schema-of-truth ordering update; `greentic-types/.codex/repo_overview.md` PRE-PR + POST-PR sync; `greentic-pack/docs/pack-format.md` pointer. | 0.5 |
| **Total Phase 1–3 (canary)** | | **~6.75 days** |
| Phase 4 — `oauth_providers`, `operator.tenants`, `operator.allow.paths`, `operator.packs.refs` migrations (separate PRs, separate effort). | (out of v1) | ~3–5 days |
| Phase 4 — web/Adaptive Card table renderer. | (separate spec) | TBD; not estimable from local repo state. |

### Test surface needing new coverage

- `greentic-types`: CBOR + JSON round-trip for `Table` in pack and component QA specs; legacy → v0.6 adapter rejects nested Tables defensively.
- `greentic-pack`: `packc qa` lints uniqueness of `TableColumn.key`; `packc qa apply` accepts Table answers.
- `greentic-setup`: bridge produces matching `ListSpec`; CLI prompt happy-path + each error path; headless answer file with array; persistence round-trip.
- `messaging-webchat-gui`: existing `nav_links_json` answers continue to load (legacy alias path) for one release; new `nav_links` answers produce identical tenant-config output to the legacy path.
- `greentic-e2e`: end-to-end run of `gtc setup` consuming a fixture answers file with Tables.
