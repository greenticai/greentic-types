//! Telemetry helpers exposed by `greentic-types`.

#[cfg(feature = "otel-keys")]
mod keys;
mod span_context;

#[cfg(feature = "otel-keys")]
pub use keys::OtlpKeys;
pub use span_context::SpanContext;

/// Canonical [`TenantCtx`](crate::TenantCtx)`.attributes` keys that carry the
/// deploy-spec rollout identifiers (B11/C5) and the messaging endpoint
/// discriminator (M1.4) into telemetry.
///
/// The runtime (runner-host) populates these on the per-invocation
/// `TenantCtx`; the telemetry bridge ([`set_current_tenant_ctx`]) copies any
/// present value into the corresponding `TelemetryCtx` field. Riding the
/// existing free-form `attributes` map avoids adding typed fields to the
/// foundation `TenantCtx` — no serialized-shape or API cascade across the 30+
/// downstream crates. The producer of these values (the revision dispatcher
/// for rollout IDs, the ingress dispatcher for `MESSAGING_ENDPOINT_ID`) is
/// Phase D / M1.4c; until then the keys are simply absent.
///
/// `TEAM` is exposed as a key for parity with the other attribute names, but
/// the bridge first reads the typed [`TenantCtx::team_id`](crate::TenantCtx)
/// field (with [`team`](crate::TenantCtx) as fallback, matching
/// `TenantIdentity::from`); the attribute is unused for `team`.
pub mod attr_keys {
    /// Team scope within the tenant. Exposed for parity; the bridge reads
    /// `TenantCtx.team_id` (typed) first.
    pub const TEAM: &str = "gt.team";
    /// Billing principal (P6).
    pub const CUSTOMER_ID: &str = "gt.customer_id";
    /// `BundleDeployment` id (P6).
    pub const DEPLOYMENT_ID: &str = "gt.deployment_id";
    /// Application bundle id.
    pub const BUNDLE_ID: &str = "gt.bundle_id";
    /// Immutable revision id.
    pub const REVISION_ID: &str = "gt.revision_id";
    /// Pack id the invocation resolved to (C5).
    pub const PACK_ID: &str = "gt.pack_id";
    /// Env-pack kind backing the environment (C5, e.g. `greentic.deployer.k8s`).
    pub const ENV_PACK_KIND: &str = "gt.env_pack_kind";
    /// Rollout generation of the deployment's routing table (C5), encoded as
    /// the decimal string of a `u64`. Non-numeric values are skipped by the
    /// bridge.
    pub const GENERATION: &str = "gt.generation";
    /// Messaging endpoint id the inbound activity arrived on (M1.4, e.g.
    /// `teams-legal` vs `teams-accounting`). Disambiguates traces and logs
    /// when one env hosts multiple provider instances of the same type.
    pub const MESSAGING_ENDPOINT_ID: &str = "gt.messaging_endpoint_id";
}

#[cfg(feature = "telemetry-autoinit")]
use greentic_telemetry::set_current_telemetry_ctx;

#[cfg(feature = "telemetry-autoinit")]
pub use greentic_telemetry::{TelemetryConfig, TelemetryCtx, init_telemetry_auto};
#[cfg(feature = "telemetry-autoinit")]
pub use greentic_types_macros::main;
#[cfg(feature = "telemetry-autoinit")]
#[doc(hidden)]
pub use tokio::main as __tokio_main;

#[cfg(feature = "telemetry-autoinit")]
/// Installs the default Greentic telemetry stack using greentic-telemetry's auto configuration.
pub fn install_telemetry(service_name: &str) -> anyhow::Result<()> {
    init_telemetry_auto(TelemetryConfig {
        service_name: service_name.to_string(),
    })
}

#[cfg(feature = "telemetry-autoinit")]
/// Projects a [`TenantCtx`](crate::TenantCtx) into a [`TelemetryCtx`].
///
/// Copies `tenant`, `env`, and the optional `session`/`flow`/`node`/`provider`
/// fields. Reads `team` from the typed [`TenantCtx::team_id`](crate::TenantCtx)
/// field (with [`team`](crate::TenantCtx) as fallback, matching
/// `TenantIdentity::from`). The B11/C5 rollout identifiers
/// (`customer_id`/`deployment_id`/`bundle_id`/`revision_id`/`pack_id`/
/// `env_pack_kind`/`generation`) and the M1.4 `messaging_endpoint_id` ride the
/// free-form `attributes` map; the producers (revision dispatcher / ingress
/// dispatcher) land in Phase D / M1.4c and these keys are simply absent until
/// then. `generation` is parsed as `u64`; malformed values are skipped rather
/// than panicking — the bridge is a hot-path projection, not a validator.
/// Pure and allocation-light so it can be unit-tested without the task-local
/// slot.
pub fn tenant_ctx_to_telemetry(ctx: &crate::TenantCtx) -> TelemetryCtx {
    let mut telemetry = TelemetryCtx::new(ctx.tenant_id.as_ref()).with_env(ctx.env.as_str());
    if let Some(team) = ctx.team_id.as_ref().or(ctx.team.as_ref()) {
        telemetry = telemetry.with_team(team.as_str());
    }
    if let Some(session) = ctx.session_id() {
        telemetry = telemetry.with_session(session);
    }
    if let Some(flow) = ctx.flow_id() {
        telemetry = telemetry.with_flow(flow);
    }
    if let Some(node) = ctx.node_id() {
        telemetry = telemetry.with_node(node);
    }
    if let Some(provider) = ctx.provider_id() {
        telemetry = telemetry.with_provider(provider);
    }
    // B11/C5: rollout identifiers ride the free-form attributes map.
    if let Some(v) = ctx.attributes.get(attr_keys::CUSTOMER_ID) {
        telemetry = telemetry.with_customer_id(v);
    }
    if let Some(v) = ctx.attributes.get(attr_keys::DEPLOYMENT_ID) {
        telemetry = telemetry.with_deployment_id(v);
    }
    if let Some(v) = ctx.attributes.get(attr_keys::BUNDLE_ID) {
        telemetry = telemetry.with_bundle_id(v);
    }
    if let Some(v) = ctx.attributes.get(attr_keys::REVISION_ID) {
        telemetry = telemetry.with_revision_id(v);
    }
    if let Some(v) = ctx.attributes.get(attr_keys::PACK_ID) {
        telemetry = telemetry.with_pack_id(v);
    }
    if let Some(v) = ctx.attributes.get(attr_keys::ENV_PACK_KIND) {
        telemetry = telemetry.with_env_pack_kind(v);
    }
    if let Some(n) = ctx
        .attributes
        .get(attr_keys::GENERATION)
        .and_then(|v| v.parse::<u64>().ok())
    {
        telemetry = telemetry.with_generation(n);
    }
    // M1.4: messaging endpoint discriminator rides the same attributes map.
    if let Some(v) = ctx.attributes.get(attr_keys::MESSAGING_ENDPOINT_ID) {
        telemetry = telemetry.with_messaging_endpoint_id(v);
    }
    telemetry
}

#[cfg(feature = "telemetry-autoinit")]
/// Stores the tenant context into the task-local telemetry slot.
pub fn set_current_tenant_ctx(ctx: &crate::TenantCtx) {
    set_current_telemetry_ctx(tenant_ctx_to_telemetry(ctx));
}

#[cfg(all(test, feature = "telemetry-autoinit"))]
mod tests {
    use crate::{EnvId, TeamId, TenantCtx, TenantId};
    use std::str::FromStr;

    fn ctx() -> TenantCtx {
        TenantCtx::new(
            EnvId::from_str("prod-eu").unwrap_or_else(|err| panic!("{err}")),
            TenantId::from_str("acme").unwrap_or_else(|err| panic!("{err}")),
        )
    }

    #[test]
    fn bridge_copies_env_and_rollout_ids_from_attributes() {
        let mut c = ctx();
        c.attributes
            .insert(super::attr_keys::CUSTOMER_ID.into(), "cust-acme".into());
        c.attributes
            .insert(super::attr_keys::DEPLOYMENT_ID.into(), "01JTKS".into());
        c.attributes.insert(
            super::attr_keys::BUNDLE_ID.into(),
            "customer.support".into(),
        );
        c.attributes
            .insert(super::attr_keys::REVISION_ID.into(), "01JTKR".into());
        c.attributes.insert(
            super::attr_keys::PACK_ID.into(),
            "customer.support@1.2.0".into(),
        );
        c.attributes.insert(
            super::attr_keys::ENV_PACK_KIND.into(),
            "greentic.deployer.k8s".into(),
        );
        c.attributes
            .insert(super::attr_keys::GENERATION.into(), "3".into());
        c.attributes.insert(
            super::attr_keys::MESSAGING_ENDPOINT_ID.into(),
            "teams-legal".into(),
        );

        let t = super::tenant_ctx_to_telemetry(&c);
        assert_eq!(t.tenant, "acme");
        assert_eq!(t.env.as_deref(), Some("prod-eu"));
        assert_eq!(t.customer_id.as_deref(), Some("cust-acme"));
        assert_eq!(t.deployment_id.as_deref(), Some("01JTKS"));
        assert_eq!(t.bundle_id.as_deref(), Some("customer.support"));
        assert_eq!(t.revision_id.as_deref(), Some("01JTKR"));
        assert_eq!(t.pack_id.as_deref(), Some("customer.support@1.2.0"));
        assert_eq!(t.env_pack_kind.as_deref(), Some("greentic.deployer.k8s"));
        assert_eq!(t.generation.as_deref(), Some("3"));
        assert_eq!(t.messaging_endpoint_id.as_deref(), Some("teams-legal"));
    }

    #[test]
    fn bridge_leaves_rollout_ids_unset_when_absent() {
        let t = super::tenant_ctx_to_telemetry(&ctx());
        assert_eq!(t.env.as_deref(), Some("prod-eu"));
        assert!(t.team.is_none());
        assert!(t.customer_id.is_none());
        assert!(t.deployment_id.is_none());
        assert!(t.bundle_id.is_none());
        assert!(t.revision_id.is_none());
        assert!(t.pack_id.is_none());
        assert!(t.env_pack_kind.is_none());
        assert!(t.generation.is_none());
        assert!(t.messaging_endpoint_id.is_none());
    }

    /// `team` is projected from the typed `TenantCtx.team_id` field, NOT the
    /// `attributes` map — keeps the bridge consistent with `TenantIdentity::from`
    /// and the rest of the typed-field projections (`session`, `flow`, ...).
    ///
    /// Bypasses the `with_team` builder (which sets BOTH `team_id` and `team`
    /// in lockstep) and directly assigns only `team_id`, so this test actually
    /// proves the `team_id`-first branch of the bridge's `.or()` chain.
    #[test]
    fn bridge_projects_team_from_typed_field() {
        let team = TeamId::from_str("support").unwrap_or_else(|err| panic!("{err}"));
        let mut c = ctx();
        c.team_id = Some(team); // typed slot only; legacy stays None
        c.team = None;
        let t = super::tenant_ctx_to_telemetry(&c);
        assert_eq!(t.team.as_deref(), Some("support"));
    }

    /// Fallback path: if only the legacy `team` slot is populated (no
    /// `team_id`), the bridge still picks it up — same precedent as
    /// `TenantIdentity::from`. Reachable via deserialization from an older
    /// `TenantCtx` schema that predates the `team_id` field.
    #[test]
    fn bridge_falls_back_to_legacy_team_slot() {
        let team = TeamId::from_str("support").unwrap_or_else(|err| panic!("{err}"));
        let mut c = ctx();
        c.team = Some(team); // legacy slot only; team_id stays None
        c.team_id = None;
        let t = super::tenant_ctx_to_telemetry(&c);
        assert_eq!(t.team.as_deref(), Some("support"));
    }

    /// Disambiguation: when both `team_id` (typed) and `team` (legacy) are
    /// populated with DIFFERENT values, the typed `team_id` wins — matches
    /// `TenantIdentity::from`'s `team_id.or(team)` precedent. Guards the
    /// bridge's `.or()` chain against accidental reversal in a future refactor.
    #[test]
    fn bridge_team_id_wins_over_legacy_team_slot() {
        let typed = TeamId::from_str("typed-team").unwrap_or_else(|err| panic!("{err}"));
        let legacy = TeamId::from_str("legacy-team").unwrap_or_else(|err| panic!("{err}"));
        let mut c = ctx();
        c.team_id = Some(typed);
        c.team = Some(legacy);
        let t = super::tenant_ctx_to_telemetry(&c);
        assert_eq!(t.team.as_deref(), Some("typed-team"));
    }

    /// Malformed `generation` attribute is silently dropped — the bridge is a
    /// hot-path projection, not a validator. A producer bug that writes a
    /// non-`u64` shouldn't crash every invocation; the attribute is treated as
    /// absent (same outcome as never being set).
    #[test]
    fn bridge_drops_non_u64_generation_silently() {
        let mut c = ctx();
        c.attributes
            .insert(super::attr_keys::GENERATION.into(), "not-a-number".into());
        let t = super::tenant_ctx_to_telemetry(&c);
        assert!(t.generation.is_none());
    }
}
