//! Telemetry helpers exposed by `greentic-types`.

#[cfg(feature = "otel-keys")]
mod keys;
mod span_context;

#[cfg(feature = "otel-keys")]
pub use keys::OtlpKeys;
pub use span_context::SpanContext;

/// Canonical [`TenantCtx`](crate::TenantCtx)`.attributes` keys that carry the
/// deploy-spec rollout identifiers into telemetry (B11).
///
/// The runtime (runner-host) populates these on the per-invocation
/// `TenantCtx`; the telemetry bridge ([`set_current_tenant_ctx`]) copies any
/// present value into the corresponding `TelemetryCtx` field. Riding the
/// existing free-form `attributes` map avoids adding typed fields to the
/// foundation `TenantCtx` — no serialized-shape or API cascade across the 30+
/// downstream crates. The producer of these values (the revision dispatcher)
/// is Phase D; until then the keys are simply absent.
pub mod attr_keys {
    /// Billing principal (P6).
    pub const CUSTOMER_ID: &str = "gt.customer_id";
    /// `BundleDeployment` id (P6).
    pub const DEPLOYMENT_ID: &str = "gt.deployment_id";
    /// Application bundle id.
    pub const BUNDLE_ID: &str = "gt.bundle_id";
    /// Immutable revision id.
    pub const REVISION_ID: &str = "gt.revision_id";
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
/// fields, plus the B11 rollout identifiers (`customer_id`/`deployment_id`/
/// `bundle_id`/`revision_id`) when present in [`attr_keys`]. Pure and
/// allocation-light so it can be unit-tested without the task-local slot.
pub fn tenant_ctx_to_telemetry(ctx: &crate::TenantCtx) -> TelemetryCtx {
    let mut telemetry = TelemetryCtx::new(ctx.tenant_id.as_ref()).with_env(ctx.env.as_str());
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
    // B11: rollout identifiers ride the free-form attributes map.
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
    telemetry
}

#[cfg(feature = "telemetry-autoinit")]
/// Stores the tenant context into the task-local telemetry slot.
pub fn set_current_tenant_ctx(ctx: &crate::TenantCtx) {
    set_current_telemetry_ctx(tenant_ctx_to_telemetry(ctx));
}

#[cfg(all(test, feature = "telemetry-autoinit"))]
mod tests {
    use crate::{EnvId, TenantCtx, TenantId};
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

        let t = super::tenant_ctx_to_telemetry(&c);
        assert_eq!(t.tenant, "acme");
        assert_eq!(t.env.as_deref(), Some("prod-eu"));
        assert_eq!(t.customer_id.as_deref(), Some("cust-acme"));
        assert_eq!(t.deployment_id.as_deref(), Some("01JTKS"));
        assert_eq!(t.bundle_id.as_deref(), Some("customer.support"));
        assert_eq!(t.revision_id.as_deref(), Some("01JTKR"));
    }

    #[test]
    fn bridge_leaves_rollout_ids_unset_when_absent() {
        let t = super::tenant_ctx_to_telemetry(&ctx());
        assert_eq!(t.env.as_deref(), Some("prod-eu"));
        assert!(t.customer_id.is_none());
        assert!(t.deployment_id.is_none());
        assert!(t.bundle_id.is_none());
        assert!(t.revision_id.is_none());
    }
}
