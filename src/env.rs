//! Environment-id resolution and the `dev` → `local` compatibility alias (A4b).
//!
//! Single source of truth shared by `greentic-setup` (which *writes* secrets and
//! dev-store entries), `greentic-start` (the runtime that *reads* them), and
//! `greentic-deployer` (which promotes them to a cloud secret manager). Keeping
//! the resolution here means the env segment used in `secrets://{env}/...` URIs
//! can never drift between the writer and the readers — the bug A4b introduced
//! when only `greentic-setup` adopted the `local` default.

use std::sync::atomic::{AtomicBool, Ordering};

/// Default environment id when nothing is set.
///
/// A4b flipped this from `dev` to `local` — the env `gtc setup` / `gtc start`
/// auto-create.
pub const DEFAULT_ENV_ID: &str = "local";

/// Legacy environment id, accepted via the compat alias and remapped to
/// [`DEFAULT_ENV_ID`].
pub const LEGACY_ENV_ID: &str = "dev";

/// Env-var that disables the [`LEGACY_ENV_ID`] → [`DEFAULT_ENV_ID`] alias.
///
/// Set to `1`, `true`, `yes`, or `on` (case-insensitive) to make any resolved
/// value of `dev` hard-fail with a remediation hint (intended for CI assertions
/// that no code-path still resolves to the legacy id).
pub const DISABLE_ALIAS_ENV_VAR: &str = "GREENTIC_DISABLE_DEV_ALIAS";

static WARNED: AtomicBool = AtomicBool::new(false);

/// Apply the `dev` → `local` compat alias.
///
/// Returns [`DEFAULT_ENV_ID`] for input equal to [`LEGACY_ENV_ID`]; returns the
/// input unchanged for any other value.
///
/// # Panics
/// Panics if [`DISABLE_ALIAS_ENV_VAR`] is set (truthy) and `env` is the legacy
/// id — the hard-fail expiry gate.
#[must_use]
pub fn apply_dev_alias(env: &str) -> String {
    if env != LEGACY_ENV_ID {
        return env.to_string();
    }
    assert!(
        !alias_disabled(),
        "environment `{LEGACY_ENV_ID}` is no longer accepted (set via {DISABLE_ALIAS_ENV_VAR}=1). \
         Migrate to `{DEFAULT_ENV_ID}`, or pass `--env {DEFAULT_ENV_ID}` / unset $GREENTIC_ENV."
    );
    if !WARNED.swap(true, Ordering::SeqCst) {
        eprintln!(
            "[greentic] env `{LEGACY_ENV_ID}` is deprecated; resolving as `{DEFAULT_ENV_ID}` for this \
             process. Set {DISABLE_ALIAS_ENV_VAR}=1 to hard-fail on `{LEGACY_ENV_ID}`."
        );
    }
    DEFAULT_ENV_ID.to_string()
}

/// Resolve the effective environment id.
///
/// Priority: `override_env` > `$GREENTIC_ENV` > [`DEFAULT_ENV_ID`]. The
/// [`apply_dev_alias`] compat alias is applied to the result, so a `dev` from
/// any source resolves to `local`.
#[must_use]
pub fn resolve_env(override_env: Option<&str>) -> String {
    let raw = override_env
        .map(ToString::to_string)
        .or_else(|| std::env::var("GREENTIC_ENV").ok())
        .unwrap_or_else(|| DEFAULT_ENV_ID.to_string());
    apply_dev_alias(&raw)
}

fn alias_disabled() -> bool {
    std::env::var(DISABLE_ALIAS_ENV_VAR)
        .ok()
        .map(|v| {
            matches!(
                v.trim().to_ascii_lowercase().as_str(),
                "1" | "true" | "yes" | "on"
            )
        })
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn non_legacy_passthrough() {
        assert_eq!(apply_dev_alias("local"), "local");
        assert_eq!(apply_dev_alias("prod"), "prod");
    }

    #[test]
    fn legacy_dev_remaps_to_local() {
        assert_eq!(apply_dev_alias(LEGACY_ENV_ID), DEFAULT_ENV_ID);
    }

    #[test]
    fn resolve_prefers_override_then_default() {
        // Explicit non-legacy override wins and is returned verbatim.
        assert_eq!(resolve_env(Some("staging")), "staging");
        // A legacy override is aliased.
        assert_eq!(resolve_env(Some(LEGACY_ENV_ID)), DEFAULT_ENV_ID);
    }
}
