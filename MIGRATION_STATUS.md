# Migration status — greentic-types (Public-Launch Secrets)

> [!WARNING]
> This page tracks migration progress and compatibility status.
> For canonical v0.6 contract guidance, use [`docs/vision/canonical-v0.6.md`](docs/vision/canonical-v0.6.md).

- What changed: `SecretKey`, `SecretScope`, `SecretFormat`, and `SecretRequirement` are now defined here (not in `greentic-secrets-spec`) and exported via `greentic_types::secrets::*`. Use `SecretKey::parse` for validation. Secrets lists in capabilities (`SecretsCaps`), component host capabilities, deployment plans, and binding hints all use `Vec<SecretRequirement>`.
- What broke: serde/schema surface for secrets changed; any downstream expecting plain string keys or the old `SecretPlan` will fail to compile/deserialize. Schema consumers should refresh.
- New: Pack manifests/metadata now expose `secret_requirements: Vec<SecretRequirement>` so runners/deployers/distributors can read `.gtpack` requirements directly without custom parsing.
- Next repos to update: bump `greentic-types` to this version in pack tooling and hosts that read packs: `greentic-pack/packc` (write/aggregate), `greentic-runner/runner-host` (preflight/expose), `greentic-deployer` (preflight), `greentic-distributor` (API surface), `greentic-distributor-client` (DTO mapping).
- Distributor API: `ResolveComponentResponse` now exposes optional `secret_requirements: Option<Vec<SecretRequirement>>` so distributor clients/servers can surface pack/component requirements. Bump `greentic-distributor`, `greentic-distributor-client`, and any runner/deployer adapters consuming distributor responses.
- Distributor API pack status v2 now has optional `secret_requirements: Option<Vec<SecretRequirement>>`; bump the same downstreams to read/write the field.
