use super::{
    client::PackageTelemetryClient,
    sanitize::{classify_example, classify_package_manager, public_value_or_other},
    transport::PackageTelemetryTransport,
    types::PackageEvent,
};

const TASK_ALLOWLIST: [&str; 8] = [
    "build",
    "test",
    "lint",
    "typecheck",
    "checktypes",
    "check-types",
    "type-check",
    "check",
];

pub struct CreateTurboTelemetry<T> {
    client: PackageTelemetryClient<T>,
}

impl<T: PackageTelemetryTransport> CreateTurboTelemetry<T> {
    pub fn new(client: PackageTelemetryClient<T>) -> Self {
        Self { client }
    }

    pub fn track_option_example(&mut self, value: Option<&str>) -> Option<PackageEvent> {
        value.map(|value| self.client.track_option("example", classify_example(value)))
    }

    pub fn track_option_package_manager(&mut self, value: Option<&str>) -> Option<PackageEvent> {
        value.map(|value| {
            self.client
                .track_option("package_manager", classify_package_manager(value))
        })
    }

    pub fn track_option_skip_install(&mut self, value: Option<bool>) -> Option<PackageEvent> {
        value
            .filter(|value| *value)
            .map(|value| self.client.track_option("skip_install", value.to_string()))
    }

    pub fn track_option_skip_transforms(&mut self, value: Option<bool>) -> Option<PackageEvent> {
        value.filter(|value| *value).map(|value| {
            self.client
                .track_option("skip_transforms", value.to_string())
        })
    }

    pub fn track_option_turbo_version(&mut self, value: Option<&str>) -> Option<PackageEvent> {
        value.map(|value| {
            self.client
                .track_option("turbo_version", public_value_or_other(value, 128))
        })
    }

    pub fn track_option_example_path(&mut self, value: Option<&str>) -> Option<PackageEvent> {
        value.map(|_| self.client.track_option("example_path", "provided"))
    }

    pub fn track_argument_directory(&mut self, provided: bool) -> Option<PackageEvent> {
        provided.then(|| self.client.track_argument("project_directory", "provided"))
    }

    pub fn track_argument_package_manager(&mut self, value: Option<&str>) -> Option<PackageEvent> {
        value.map(|value| {
            self.client
                .track_argument("package_manager", classify_package_manager(value))
        })
    }

    pub fn track_command_status(&mut self, command: &str, status: &str) -> PackageEvent {
        self.client.track_command_status(command, status)
    }

    pub fn track_command_warning(&mut self, warning: &str) -> PackageEvent {
        self.client.track_command_warning(warning)
    }

    pub fn track_command_error(&mut self, error: &str) -> PackageEvent {
        self.client.track_command_error(error)
    }

    pub async fn close(self) {
        self.client.close().await;
    }
}

pub struct TurboIgnoreTelemetry<T> {
    client: PackageTelemetryClient<T>,
}

impl<T: PackageTelemetryTransport> TurboIgnoreTelemetry<T> {
    pub fn new(client: PackageTelemetryClient<T>) -> Self {
        Self { client }
    }

    pub fn track_ci(&mut self, name: Option<&str>) -> PackageEvent {
        self.client
            .track_public("ci", public_value_or_other(name.unwrap_or("unknown"), 128))
    }

    pub fn track_argument_workspace(&mut self, provided: bool) -> Option<PackageEvent> {
        provided.then(|| self.client.track_argument("workspace", "provided"))
    }

    pub fn track_option_task(&mut self, value: Option<&str>) -> Option<PackageEvent> {
        value.map(|value| {
            let value = if TASK_ALLOWLIST.contains(&value) {
                value
            } else {
                "other"
            };
            self.client.track_option("task", value)
        })
    }

    pub fn track_option_fallback(&mut self, value: Option<&str>) -> Option<PackageEvent> {
        value.map(|_| self.client.track_option("fallback", "provided"))
    }

    pub fn track_option_directory(&mut self, value: Option<&str>) -> Option<PackageEvent> {
        value.map(|_| self.client.track_option("directory", "custom"))
    }

    pub fn track_option_max_buffer(&mut self, value: Option<usize>) -> Option<PackageEvent> {
        value.map(|value| self.client.track_option("max_buffer", value.to_string()))
    }

    pub fn track_command_status(&mut self, command: &str, status: &str) -> PackageEvent {
        self.client.track_command_status(command, status)
    }

    pub fn track_command_warning(&mut self, warning: &str) -> PackageEvent {
        self.client.track_command_warning(warning)
    }

    pub fn track_command_error(&mut self, error: &str) -> PackageEvent {
        self.client.track_command_error(error)
    }

    pub async fn close(self) {
        self.client.close().await;
    }
}
