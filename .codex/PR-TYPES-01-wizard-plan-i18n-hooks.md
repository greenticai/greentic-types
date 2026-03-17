# PR-TYPES-01 — Add wizard plan + delegation primitives (minimal) and i18n-ready QA text

**Repo:** `greentic-types`  
**Theme:** Delegating QA-driven wizards with deterministic replay, multi-frontend UI, and i18n.

## Outcomes
- Adds/extends wizard capability in this repo so **`greentic-dev wizard`** can delegate to it.
- Maintains **deterministic** behavior: **`apply()` produces a plan, execution is separate**.
- Reuses existing QA/schema primitives; avoid duplicating type systems.

## Non-goals
- No breaking CLI UX unless explicitly documented.
- No new “parallel QA types” if existing ones already exist (reuse).
## Why
To allow cross-repo wizard delegation and deterministic replay, we need shared primitives:
- wizard identity + mode + metadata
- deterministic plan steps (incl. delegate)
- i18n-ready text fields for QA prompts

`greentic-types` is the shared home for these primitives.

## Audit
Run and record:
- `rg -n "QaSpec|Question|Answer|validate_answers|SetupContract|QaSpecSource" src/`
- `rg -n "Plan|Step|Action|Task" src/` (look for existing plan types to reuse)
- Inspect existing i18n primitives (`I18nText`, profiles) and see if QA types already reference them.

Add `docs/audit-wizard-primitives.md` summarizing what exists and what will be added.

## Proposed additions (only if missing)
### A) Wizard plan primitives
Add `src/wizard.rs` (or `src/wizard/mod.rs`) with:
- `WizardId` (string newtype)
- `WizardTarget` (component/flow/pack/operator/dev/bundle)
- `WizardMode` (default/setup/update/remove + scaffold/build/new)
- `WizardPlan { meta, steps }`
- `WizardStep`:
  - `EnsureDir`
  - `WriteFiles`
  - `RunCli` (only as a bridge; prefer structured steps)
  - `Delegate { target, id, mode, prefilled_answers, output_map }`

Ensure serde + schemars support if this repo already exports schemas.

### B) i18n-ready text
If QA question types store plain strings, add (non-breaking if possible):
- `DisplayText { text?: String, i18n?: I18nKey, args?: Map }`
OR reuse existing `I18nText` directly.

## Tests
- serde round-trip of WizardPlan
- schema generation (if used)
- basic hashing helper (optional): compute spec hash / plan hash deterministically

## Definition of done
- Adds minimal new types without breaking downstream crates.
- No duplication of QA types; only small orchestration primitives.
## Codex prompt (copy/paste)

You are implementing **PR-TYPES-01**.  
**Pre-authorized:** create/update files, add tests, add docs, run formatting, add CI checks if needed.  
**Avoid destructive actions:** do not delete large subsystems; prefer additive refactors; keep backward compatibility unless the PR explicitly says otherwise.

Steps:
1) Perform the **Audit** tasks first and summarize findings in PR notes.
2) Implement the change list with minimal diffs aligned to the current repo patterns.
3) Add tests (unit + one integration/smoke test) and update docs.
4) Ensure `cargo fmt` + `cargo test` pass.

Repo-specific guidance:
- Prefer reuse: if a plan/task type already exists, extend it rather than adding new ones.
- Keep additions minimal and additive.
- Ensure serde compatibility and stable field naming.
