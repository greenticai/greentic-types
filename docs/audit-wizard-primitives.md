# Audit: Wizard Primitives in `greentic-types`

Date: 2026-02-23

## Commands run

1. `rg -n "QaSpec|Question|Answer|validate_answers|SetupContract|QaSpecSource" src/`
2. `rg -n "Plan|Step|Action|Task" src/`
3. Manual inspection of `src/i18n_text.rs`, `src/schemas/component/v0_6_0/qa.rs`, and `src/schemas/pack/v0_6_0/qa.rs`.

## Findings

## QA/spec primitives already present
- `src/qa.rs` already defines reusable QA setup/validation primitives:
  - `QaSpecSource`
  - `SetupContract`
  - `ExampleAnswers`
  - `validate_answers(...)`
- Existing component and pack QA schemas are present:
  - `src/schemas/component/v0_6_0/qa.rs`
  - `src/schemas/pack/v0_6_0/qa.rs`

## Existing plan-like types
- The repository already has plan types for other domains:
  - `BuildPlan` in `src/supply_chain.rs`
  - `DeploymentPlan` and related structs in `src/deployment.rs`
  - Store plan/limits types in `src/store.rs`
- No generic wizard orchestration plan type was present before this PR.

## i18n primitives and QA coverage
- `I18nText` exists in `src/i18n_text.rs`.
- QA schema fields already use i18n-ready text:
  - `title`, `description`, `label`, `help`, `error`, and choice `label` use `I18nText`.
- Both component and pack QA specs provide i18n key collection helpers.

## Additions made in this PR
- Added top-level wizard orchestration primitives in `src/wizard.rs`:
  - `WizardId`
  - `WizardTarget`
  - `WizardMode`
  - `WizardPlanMeta`
  - `WizardPlan`
  - `WizardStep` with `EnsureDir`, `WriteFiles`, `RunCli`, and `Delegate`
- Re-exported wizard types from `src/lib.rs`.
- Added tests for serde round-trip and schema generation coverage.

## Rationale
- Reused existing QA and i18n primitives as-is.
- Added only minimal orchestration types needed for deterministic wizard planning and delegation.
