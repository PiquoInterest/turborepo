use std::time::Duration;

use serde::{Deserialize, Serialize};
use thiserror::Error;
use url::Url;

use super::sanitize::checked_metadata;

const DEFAULT_BATCH_SIZE: usize = 20;
const DEFAULT_TIMEOUT: Duration = Duration::from_millis(250);
const MAX_BATCH_SIZE: usize = 100;
const MAX_REQUEST_TIMEOUT: Duration = Duration::from_secs(10);

#[derive(Debug, Error)]
pub enum PackageTelemetryError {
    #[error("invalid telemetry endpoint")]
    InvalidEndpoint,
    #[error("invalid telemetry metadata")]
    InvalidMetadata,
    #[error("invalid telemetry options")]
    InvalidOptions,
    #[error("invalid telemetry configuration")]
    InvalidConfig,
    #[error("unsafe telemetry configuration path")]
    UnsafeConfigPath,
    #[error("telemetry configuration path is unavailable")]
    ConfigPath,
    #[error("telemetry configuration I/O failed")]
    ConfigIo(#[source] std::io::Error),
    #[error("failed to create telemetry HTTP client")]
    HttpClient(#[source] reqwest::Error),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PackageKind {
    CreateTurbo,
    TurboIgnore,
}

impl PackageKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::CreateTurbo => "create-turbo",
            Self::TurboIgnore => "turbo-ignore",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackageInfo {
    kind: PackageKind,
    version: String,
}

impl PackageInfo {
    pub fn new(
        kind: PackageKind,
        version: impl Into<String>,
    ) -> Result<Self, PackageTelemetryError> {
        let version = checked_metadata(version.into())?;
        Ok(Self { kind, version })
    }

    pub const fn kind(&self) -> PackageKind {
        self.kind
    }

    pub fn name(&self) -> &'static str {
        self.kind.as_str()
    }

    pub fn version(&self) -> &str {
        &self.version
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackageRuntimeInfo {
    runtime_version: String,
    operating_system: String,
    architecture: String,
}

impl PackageRuntimeInfo {
    pub fn new(
        runtime_version: impl Into<String>,
        operating_system: impl Into<String>,
        architecture: impl Into<String>,
    ) -> Result<Self, PackageTelemetryError> {
        Ok(Self {
            runtime_version: checked_metadata(runtime_version.into())?,
            operating_system: checked_metadata(operating_system.into())?,
            architecture: checked_metadata(architecture.into())?,
        })
    }

    pub(super) fn user_agent(&self, package: &PackageInfo) -> String {
        format!(
            "{} {} {} {} {}",
            package.name(),
            package.version(),
            self.runtime_version,
            self.operating_system,
            self.architecture
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PackageTelemetryOptions {
    pub(super) batch_size: usize,
    pub(super) timeout: Duration,
}

impl PackageTelemetryOptions {
    pub fn new(batch_size: usize, timeout: Duration) -> Result<Self, PackageTelemetryError> {
        if !(1..=MAX_BATCH_SIZE).contains(&batch_size)
            || timeout.is_zero()
            || timeout > MAX_REQUEST_TIMEOUT
        {
            return Err(PackageTelemetryError::InvalidOptions);
        }
        Ok(Self {
            batch_size,
            timeout,
        })
    }
}

impl Default for PackageTelemetryOptions {
    fn default() -> Self {
        Self {
            batch_size: DEFAULT_BATCH_SIZE,
            timeout: DEFAULT_TIMEOUT,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PackageEvent {
    pub id: String,
    pub key: String,
    pub value: String,
    pub package_name: String,
    pub package_version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PackageEventEnvelope {
    pub package: PackageEvent,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackageTelemetryRequest {
    pub endpoint: Url,
    pub events: Vec<PackageEventEnvelope>,
    pub telemetry_id: String,
    pub session_id: String,
    pub user_agent: String,
    pub timeout: Duration,
}
