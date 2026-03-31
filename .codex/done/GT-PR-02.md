# GT-PR-02 — Add Provider Installation Record model (shared across domains)

REPO: greenticai/greentic-types

GOAL
Define a shared provider installation record model so provisioning outputs can be persisted consistently by messaging/events/secrets.

DELIVERABLES
- Types:
  - ProviderInstallId (string newtype)
  - ProviderInstallRecord { tenant ctx, provider_id, install_id, pack_id, pack_version, created_at, updated_at, config_refs, secret_refs, webhook_state, subscriptions_state, metadata }
- serde + schemars (if repo uses schemars)
- tests for roundtrip + basic invariants

ACCEPTANCE
- messaging/events/secrets can store/read installs with same schema.

