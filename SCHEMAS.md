# JSON Schema Publishing

> [!IMPORTANT]
> This page lists published schema artifacts.
> Canonical vs legacy contract policy lives in [`docs/vision/README.md`](docs/vision/README.md).

Schemas for the shared Greentic types are published to GitHub Pages so IDEs, CLIs, and CI jobs can validate documents consistently. Every schema lives under the stable base URL:

```
https://greenticai.github.io/greentic-types/schemas/v1/<name>.schema.json
```

The `bin/export-schemas.rs` helper (or `greentic_types::write_all_schemas`) materialises the schemas into `dist/schemas/v1/`. The GitHub Pages workflow runs the helper on every push to `master` and republishes the `dist/` directory.

The v1 flow/pack model now embeds flows inside `PackManifest` and publishes the Flow schema as `greentic.flow.v1` and the pack schema as `greentic.pack-manifest.v1`.

## Canonical URLs

| Type | Schema URL |
|------|------------|
| PackId | https://greenticai.github.io/greentic-types/schemas/v1/pack-id.schema.json |
| ComponentId | https://greenticai.github.io/greentic-types/schemas/v1/component-id.schema.json |
| FlowId | https://greenticai.github.io/greentic-types/schemas/v1/flow-id.schema.json |
| NodeId | https://greenticai.github.io/greentic-types/schemas/v1/node-id.schema.json |
| TenantContext | https://greenticai.github.io/greentic-types/schemas/v1/tenant-context.schema.json |
| HashDigest | https://greenticai.github.io/greentic-types/schemas/v1/hash-digest.schema.json |
| SemverReq | https://greenticai.github.io/greentic-types/schemas/v1/semver-req.schema.json |
| RedactionPath | https://greenticai.github.io/greentic-types/schemas/v1/redaction-path.schema.json |
| Capabilities | https://greenticai.github.io/greentic-types/schemas/v1/capabilities.schema.json |
| RepoSkin | https://greenticai.github.io/greentic-types/schemas/v1/repo-skin.schema.json |
| RepoAuth | https://greenticai.github.io/greentic-types/schemas/v1/repo-auth.schema.json |
| RepoTenantConfig | https://greenticai.github.io/greentic-types/schemas/v1/repo-tenant-config.schema.json |
| TenantDidDocument | https://greenticai.github.io/greentic-types/schemas/v1/tenant-did-document.schema.json |
| Flow (greentic.flow.v1) | https://greenticai.github.io/greentic-types/schemas/v1/flow.schema.json |
| FlowResolve (greentic.flow.resolve.v1) | https://greenticai.github.io/greentic-types/schemas/v1/flow-resolve.schema.json |
| FlowResolveSummary (greentic.flow.resolve-summary.v1) | https://greenticai.github.io/greentic-types/schemas/v1/flow-resolve-summary.schema.json |
| Node | https://greenticai.github.io/greentic-types/schemas/v1/node.schema.json |
| ComponentManifest | https://greenticai.github.io/greentic-types/schemas/v1/component-manifest.schema.json |
| PackManifest (greentic.pack-manifest.v1) | https://greenticai.github.io/greentic-types/schemas/v1/pack-manifest.schema.json |
| ValidationSeverity | https://greenticai.github.io/greentic-types/schemas/v1/validation-severity.schema.json |
| ValidationDiagnostic | https://greenticai.github.io/greentic-types/schemas/v1/validation-diagnostic.schema.json |
| ValidationReport | https://greenticai.github.io/greentic-types/schemas/v1/validation-report.schema.json |
| Limits | https://greenticai.github.io/greentic-types/schemas/v1/limits.schema.json |
| TelemetrySpec | https://greenticai.github.io/greentic-types/schemas/v1/telemetry-spec.schema.json |
| NodeSummary | https://greenticai.github.io/greentic-types/schemas/v1/node-summary.schema.json |
| NodeFailure | https://greenticai.github.io/greentic-types/schemas/v1/node-failure.schema.json |
| NodeStatus | https://greenticai.github.io/greentic-types/schemas/v1/node-status.schema.json |
| RunStatus | https://greenticai.github.io/greentic-types/schemas/v1/run-status.schema.json |
| TranscriptOffset | https://greenticai.github.io/greentic-types/schemas/v1/transcript-offset.schema.json |
| ToolsCaps | https://greenticai.github.io/greentic-types/schemas/v1/tools-caps.schema.json |
| SecretsCaps | https://greenticai.github.io/greentic-types/schemas/v1/secrets-caps.schema.json |
| BranchRef | https://greenticai.github.io/greentic-types/schemas/v1/branch-ref.schema.json |
| CommitRef | https://greenticai.github.io/greentic-types/schemas/v1/commit-ref.schema.json |
| GitProviderRef | https://greenticai.github.io/greentic-types/schemas/v1/git-provider-ref.schema.json |
| ScannerRef | https://greenticai.github.io/greentic-types/schemas/v1/scanner-ref.schema.json |
| WebhookId | https://greenticai.github.io/greentic-types/schemas/v1/webhook-id.schema.json |
| ProviderInstallId | https://greenticai.github.io/greentic-types/schemas/v1/provider-install-id.schema.json |
| ProviderInstallRecord | https://greenticai.github.io/greentic-types/schemas/v1/provider-install-record.schema.json |
| RepoRef | https://greenticai.github.io/greentic-types/schemas/v1/repo-ref.schema.json |
| ComponentRef | https://greenticai.github.io/greentic-types/schemas/v1/component-ref.schema.json |
| VersionRef | https://greenticai.github.io/greentic-types/schemas/v1/version-ref.schema.json |
| BuildRef | https://greenticai.github.io/greentic-types/schemas/v1/build-ref.schema.json |
| ScanRef | https://greenticai.github.io/greentic-types/schemas/v1/scan-ref.schema.json |
| AttestationRef | https://greenticai.github.io/greentic-types/schemas/v1/attestation-ref.schema.json |
| AttestationId | https://greenticai.github.io/greentic-types/schemas/v1/attestation-id.schema.json |
| PolicyRef | https://greenticai.github.io/greentic-types/schemas/v1/policy-ref.schema.json |
| PolicyInputRef | https://greenticai.github.io/greentic-types/schemas/v1/policy-input-ref.schema.json |
| StoreRef | https://greenticai.github.io/greentic-types/schemas/v1/store-ref.schema.json |
| RegistryRef | https://greenticai.github.io/greentic-types/schemas/v1/registry-ref.schema.json |
| OciImageRef | https://greenticai.github.io/greentic-types/schemas/v1/oci-image-ref.schema.json |
| ArtifactRef | https://greenticai.github.io/greentic-types/schemas/v1/artifact-ref.schema.json |
| SbomRef | https://greenticai.github.io/greentic-types/schemas/v1/sbom-ref.schema.json |
| SigningKeyRef | https://greenticai.github.io/greentic-types/schemas/v1/signing-key-ref.schema.json |
| SignatureRef | https://greenticai.github.io/greentic-types/schemas/v1/signature-ref.schema.json |
| StatementRef | https://greenticai.github.io/greentic-types/schemas/v1/statement-ref.schema.json |
| BuildLogRef | https://greenticai.github.io/greentic-types/schemas/v1/build-log-ref.schema.json |
| ApiKeyRef | https://greenticai.github.io/greentic-types/schemas/v1/api-key-ref.schema.json |
| BuildPlan | https://greenticai.github.io/greentic-types/schemas/v1/build-plan.schema.json |
| BuildStatus | https://greenticai.github.io/greentic-types/schemas/v1/build-status.schema.json |
| ScanRequest | https://greenticai.github.io/greentic-types/schemas/v1/scan-request.schema.json |
| ScanResult | https://greenticai.github.io/greentic-types/schemas/v1/scan-result.schema.json |
| SignRequest | https://greenticai.github.io/greentic-types/schemas/v1/sign-request.schema.json |
| VerifyRequest | https://greenticai.github.io/greentic-types/schemas/v1/verify-request.schema.json |
| VerifyResult | https://greenticai.github.io/greentic-types/schemas/v1/verify-result.schema.json |
| AttestationStatement | https://greenticai.github.io/greentic-types/schemas/v1/attestation-statement.schema.json |
| MetadataRecord | https://greenticai.github.io/greentic-types/schemas/v1/metadata-record.schema.json |
| RepoContext | https://greenticai.github.io/greentic-types/schemas/v1/repo-context.schema.json |
| StoreContext | https://greenticai.github.io/greentic-types/schemas/v1/store-context.schema.json |
| Bundle | https://greenticai.github.io/greentic-types/schemas/v1/bundle.schema.json |
| DesiredStateExportSpec | https://greenticai.github.io/greentic-types/schemas/v1/desired-state-export.schema.json |
| DesiredState | https://greenticai.github.io/greentic-types/schemas/v1/desired-state.schema.json |
| DesiredSubscriptionEntry | https://greenticai.github.io/greentic-types/schemas/v1/desired-subscription-entry.schema.json |
| ArtifactSelector | https://greenticai.github.io/greentic-types/schemas/v1/artifact-selector.schema.json |
| StoreFront | https://greenticai.github.io/greentic-types/schemas/v1/storefront.schema.json |
| StoreProduct | https://greenticai.github.io/greentic-types/schemas/v1/store-product.schema.json |
| StorePlan | https://greenticai.github.io/greentic-types/schemas/v1/store-plan.schema.json |
| CapabilityMap | https://greenticai.github.io/greentic-types/schemas/v1/capability-map.schema.json |
| Subscription | https://greenticai.github.io/greentic-types/schemas/v1/subscription.schema.json |
| Environment | https://greenticai.github.io/greentic-types/schemas/v1/environment.schema.json |
| RolloutStatus | https://greenticai.github.io/greentic-types/schemas/v1/rollout-status.schema.json |
| Theme | https://greenticai.github.io/greentic-types/schemas/v1/theme.schema.json |
| LayoutSection | https://greenticai.github.io/greentic-types/schemas/v1/layout-section.schema.json |
| Collection | https://greenticai.github.io/greentic-types/schemas/v1/collection.schema.json |
| ProductOverride | https://greenticai.github.io/greentic-types/schemas/v1/product-override.schema.json |
| StoreProductKind | https://greenticai.github.io/greentic-types/schemas/v1/store-product-kind.schema.json |
| VersionStrategy | https://greenticai.github.io/greentic-types/schemas/v1/version-strategy.schema.json |
| ConnectionKind | https://greenticai.github.io/greentic-types/schemas/v1/connection-kind.schema.json |
| PackOrComponentRef | https://greenticai.github.io/greentic-types/schemas/v1/pack-or-component-ref.schema.json |
| PlanLimits | https://greenticai.github.io/greentic-types/schemas/v1/plan-limits.schema.json |
| PriceModel | https://greenticai.github.io/greentic-types/schemas/v1/price-model.schema.json |
| SubscriptionStatus | https://greenticai.github.io/greentic-types/schemas/v1/subscription-status.schema.json |
| EventEnvelope | https://greenticai.github.io/greentic-types/schemas/v1/event-envelope.schema.json |
| EventProviderDescriptor | https://greenticai.github.io/greentic-types/schemas/v1/event-provider-descriptor.schema.json |
| ChannelMessageEnvelope | https://greenticai.github.io/greentic-types/schemas/v1/channel-message-envelope.schema.json |
| Attachment | https://greenticai.github.io/greentic-types/schemas/v1/attachment.schema.json |
| WorkerRequest | https://greenticai.github.io/greentic-types/schemas/v1/worker-request.schema.json |
| WorkerMessage | https://greenticai.github.io/greentic-types/schemas/v1/worker-message.schema.json |
| WorkerResponse | https://greenticai.github.io/greentic-types/schemas/v1/worker-response.schema.json |
| OtlpKeys | https://greenticai.github.io/greentic-types/schemas/v1/otlp-keys.schema.json |
| RunResult | https://greenticai.github.io/greentic-types/schemas/v1/run-result.schema.json |

> `OtlpKeys` and `RunResult` schemas are emitted when the `otel-keys` and `time` features are enabled respectively; both keep their canonical IDs.

`ComponentManifest` includes optional development-time flows under `dev_flows`, mapping flow identifiers to FlowIR-as-JSON documents. Authoring tools (for example, `greentic-dev` and `greentic-component`) can embed editable graphs here, while runtimes and deployers may ignore the section safely.

`WorkerRequest` / `WorkerMessage` / `WorkerResponse` schemas describe compatibility transport envelopes (`payload_json` fields). They are not the canonical runtime payload envelope for new v0.6 runtime contract design.

## Generating locally

```bash
cargo run --bin export-schemas --all-features
ls dist/schemas/v1
```

Use these URLs in IDE validation rules, manifests, or CI assertions so other repos stay in sync with the shared types.

