//! Legacy v0.5.0 component schemas used for migration.
//!
//! This represents the legacy QA shape supported for migration.
//! It is not guaranteed to match all historical formats.

pub mod qa;

#[deprecated(
    since = "0.4.52",
    note = "use schemas::component::v0_6_0::ComponentQaSpec"
)]
pub use qa::LegacyComponentQaSpec;
