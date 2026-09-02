use uuid::Uuid;

use super::{
    MAX_PENDING_REQUESTS,
    config::PackageTelemetryConfig,
    sanitize::{bounded_event_key, bounded_event_value, public_value_or_other, telemetry_endpoint},
    transport::{PackageSendFuture, PackageTelemetryTransport},
    types::{
        PackageEvent, PackageEventEnvelope, PackageInfo, PackageRuntimeInfo, PackageTelemetryError,
        PackageTelemetryOptions, PackageTelemetryRequest,
    },
};

pub struct PackageTelemetryClient<T> {
    endpoint: url::Url,
    package: PackageInfo,
    runtime: PackageRuntimeInfo,
    config: PackageTelemetryConfig,
    options: PackageTelemetryOptions,
    transport: T,
    session_id: String,
    events: Vec<PackageEventEnvelope>,
    pending_requests: Vec<PackageSendFuture>,
}

impl<T: PackageTelemetryTransport> PackageTelemetryClient<T> {
    pub fn new(
        api: &str,
        package: PackageInfo,
        runtime: PackageRuntimeInfo,
        config: PackageTelemetryConfig,
        options: PackageTelemetryOptions,
        transport: T,
    ) -> Result<Self, PackageTelemetryError> {
        Ok(Self {
            endpoint: telemetry_endpoint(api)?,
            package,
            runtime,
            config,
            options,
            transport,
            session_id: Uuid::new_v4().to_string(),
            events: Vec::new(),
            pending_requests: Vec::new(),
        })
    }

    pub fn has_pending_events(&self) -> bool {
        !self.events.is_empty()
    }

    pub fn track_command_status(&mut self, command: &str, status: &str) -> PackageEvent {
        let command = public_value_or_other(command, 64);
        let status = public_value_or_other(status, 64);
        self.track(format!("command:{command}"), status, Sensitivity::Public)
    }

    pub fn track_command_warning(&mut self, warning: &str) -> PackageEvent {
        self.track(
            "warning".to_string(),
            warning.to_string(),
            Sensitivity::Sensitive,
        )
    }

    pub fn track_command_error(&mut self, error: &str) -> PackageEvent {
        self.track(
            "error".to_string(),
            error.to_string(),
            Sensitivity::Sensitive,
        )
    }

    pub(super) fn track_option(&mut self, option: &str, value: impl Into<String>) -> PackageEvent {
        self.track(
            format!("option:{}", public_value_or_other(option, 64)),
            value.into(),
            Sensitivity::Public,
        )
    }

    pub(super) fn track_argument(
        &mut self,
        argument: &str,
        value: impl Into<String>,
    ) -> PackageEvent {
        self.track(
            format!("argument:{}", public_value_or_other(argument, 64)),
            value.into(),
            Sensitivity::Public,
        )
    }

    pub(super) fn track_public(&mut self, key: &str, value: String) -> PackageEvent {
        self.track(key.to_string(), value, Sensitivity::Public)
    }

    fn track(&mut self, key: String, value: String, sensitivity: Sensitivity) -> PackageEvent {
        let value = match sensitivity {
            Sensitivity::Public => bounded_event_value(value, "other"),
            Sensitivity::Sensitive => self
                .config
                .one_way_hash_value(&bounded_event_value(value, "redacted")),
        };
        let event = PackageEvent {
            id: Uuid::new_v4().to_string(),
            key: bounded_event_key(key),
            value,
            package_name: self.package.name().to_string(),
            package_version: self.package.version().to_string(),
            parent_id: None,
        };

        if self.config.is_enabled() {
            self.events.push(PackageEventEnvelope {
                package: event.clone(),
            });
            if self.events.len() >= self.options.batch_size {
                self.flush_one_batch();
            }
        }

        event
    }

    fn flush_one_batch(&mut self) {
        if self.events.is_empty() {
            return;
        }

        let count = self.events.len().min(self.options.batch_size);
        let events: Vec<_> = self.events.drain(..count).collect();
        if !self.config.is_enabled() || self.pending_requests.len() >= MAX_PENDING_REQUESTS {
            return;
        }

        let request = PackageTelemetryRequest {
            endpoint: self.endpoint.clone(),
            events,
            telemetry_id: self.config.id().to_string(),
            session_id: self.session_id.clone(),
            user_agent: self.runtime.user_agent(&self.package),
            timeout: self.options.timeout,
        };
        self.pending_requests.push(self.transport.send(request));
    }

    pub async fn close(mut self) {
        while !self.events.is_empty() {
            self.flush_one_batch();
        }
        for request in self.pending_requests {
            request.await;
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum Sensitivity {
    Public,
    Sensitive,
}
