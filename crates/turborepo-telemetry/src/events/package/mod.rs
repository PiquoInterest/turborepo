//! Rust implementation of the package-facing `@turbo/telemetry` contract.
//!
//! The JavaScript package has a distinct wire envelope used by `create-turbo`
//! and `turbo-ignore`. This module preserves that safe-input contract while
//! making data collection fail closed and bounding attacker-controlled inputs.

mod client;
mod config;
mod sanitize;
mod transport;
mod types;
mod wrappers;

pub use client::PackageTelemetryClient;
pub use config::PackageTelemetryConfig;
pub use sanitize::environment_value_is_truthy;
pub use transport::{PackageSendFuture, PackageTelemetryTransport, ReqwestPackageTelemetryTransport};
pub use types::{
    PackageEvent, PackageEventEnvelope, PackageInfo, PackageKind, PackageRuntimeInfo,
    PackageTelemetryError, PackageTelemetryOptions, PackageTelemetryRequest,
};
pub use wrappers::{CreateTurboTelemetry, TurboIgnoreTelemetry};

pub(super) const TELEMETRY_ENDPOINT: &str = "/api/turborepo/v1/events";
pub(super) const MAX_PENDING_REQUESTS: usize = 32;
pub(super) const MAX_CONFIG_BYTES: u64 = 64 * 1024;
pub(super) const MAX_METADATA_BYTES: usize = 256;
pub(super) const MAX_EVENT_KEY_BYTES: usize = 128;
pub(super) const MAX_EVENT_VALUE_BYTES: usize = 1_024;
