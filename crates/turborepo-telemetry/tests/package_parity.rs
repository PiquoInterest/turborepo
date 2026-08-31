use std::{
    sync::{Arc, Mutex},
    time::Duration,
};

use tempfile::TempDir;
use turbopath::AbsoluteSystemPathBuf;
use turborepo_telemetry::{
    config::TelemetryConfig,
    package::{
        CreateTurboTelemetry, PackageInfo, PackageKind, PackageRuntimeInfo, PackageSendFuture,
        PackageTelemetryClient, PackageTelemetryOptions, PackageTelemetryRequest,
        PackageTelemetryTransport, TurboIgnoreTelemetry,
    },
};

#[derive(Clone, Default)]
struct RecordingTransport {
    requests: Arc<Mutex<Vec<PackageTelemetryRequest>>>,
}

impl RecordingTransport {
    fn requests(&self) -> Vec<PackageTelemetryRequest> {
        self.requests.lock().unwrap().clone()
    }
}

impl PackageTelemetryTransport for RecordingTransport {
    fn send(&self, request: PackageTelemetryRequest) -> PackageSendFuture {
        self.requests.lock().unwrap().push(request);
        Box::pin(async {})
    }
}

fn config(enabled: bool, salt: &str) -> (TempDir, TelemetryConfig) {
    let temp = tempfile::tempdir().unwrap();
    let root = AbsoluteSystemPathBuf::try_from(temp.path()).unwrap();
    let path = root.join_component("telemetry.json");
    std::fs::write(
        path.as_path(),
        format!(
            r#"{{
  "telemetry_enabled": {enabled},
  "telemetry_id": "telemetry-test-id",
  "telemetry_salt": "{salt}"
}}"#
        ),
    )
    .unwrap();
    let config = TelemetryConfig::new(path).unwrap();
    (temp, config)
}

fn client(
    kind: PackageKind,
    enabled: bool,
    batch_size: usize,
    transport: RecordingTransport,
) -> (TempDir, PackageTelemetryClient<RecordingTransport>) {
    let (temp, config) = config(enabled, "private-salt");
    let client = PackageTelemetryClient::new(
        "https://example.com",
        PackageInfo::new(kind, "1.0.0").unwrap(),
        PackageRuntimeInfo::new("v20.11.30", "Linux", "x64").unwrap(),
        config,
        PackageTelemetryOptions::new(batch_size, Duration::from_millis(250)).unwrap(),
        transport,
    )
    .unwrap();
    (temp, client)
}

#[tokio::test]
async fn batch_threshold_sends_exact_package_event_shape() {
    let transport = RecordingTransport::default();
    let (_temp, mut client) = client(PackageKind::CreateTurbo, true, 2, transport.clone());

    let first = client.track_command_status("test-command", "start");
    let second = client.track_command_status("test-command", "end");

    assert!(!client.has_pending_events());
    assert_eq!(transport.requests().len(), 1);
    let request = &transport.requests()[0];
    assert_eq!(request.endpoint.as_str(), "https://example.com/api/turborepo/v1/events");
    assert_eq!(request.telemetry_id, "telemetry-test-id");
    assert_eq!(request.user_agent, "create-turbo 1.0.0 v20.11.30 Linux x64");
    assert_eq!(request.events.len(), 2);
    assert_eq!(request.events[0].package, first);
    assert_eq!(request.events[1].package, second);

    let serialized = serde_json::to_value(&request.events).unwrap();
    assert_eq!(serialized[0]["package"]["key"], "command:test-command");
    assert_eq!(serialized[0]["package"]["value"], "start");
    assert_eq!(serialized[0]["package"]["package_name"], "create-turbo");
    assert_eq!(serialized[0]["package"]["package_version"], "1.0.0");
    assert!(serialized[0]["package"].get("parent_id").is_none());

    client.close().await;
}

#[tokio::test]
async fn partial_batch_is_flushed_on_close() {
    let transport = RecordingTransport::default();
    let (_temp, mut client) = client(PackageKind::CreateTurbo, true, 20, transport.clone());

    client.track_command_status("test-command", "start");
    assert!(client.has_pending_events());
    assert!(transport.requests().is_empty());

    client.close().await;
    assert_eq!(transport.requests().len(), 1);
    assert_eq!(transport.requests()[0].events.len(), 1);
}

#[tokio::test]
async fn disabled_config_never_queues_or_sends() {
    let transport = RecordingTransport::default();
    let (_temp, mut client) = client(PackageKind::CreateTurbo, false, 2, transport.clone());

    let event = client.track_command_status("test-command", "start");
    assert_eq!(event.key, "command:test-command");
    assert!(!client.has_pending_events());
    client.close().await;
    assert!(transport.requests().is_empty());
}

#[test]
fn salted_hash_matches_typescript_oracle() {
    let (_temp, config) = config(true, "private-salt");
    assert_eq!(
        config.one_way_hash_value("a-sensitive-value"),
        "568d39ba8435f9c37e80e01c6bb6e27d7b65b4edf837e44dee662ffc99206eec"
    );
}

#[test]
fn create_turbo_classifies_examples_without_retaining_credentials() {
    let transport = RecordingTransport::default();
    let (_temp, client) = client(PackageKind::CreateTurbo, false, 20, transport);
    let mut telemetry = CreateTurboTelemetry::new(client);

    assert_eq!(telemetry.track_option_example(Some("default")).unwrap().value, "default");
    assert_eq!(telemetry.track_option_example(Some("basic")).unwrap().value, "official");
    assert_eq!(
        telemetry
            .track_option_example(Some("https://user:ghp_secret@github.com/acme/private?token=secret"))
            .unwrap()
            .value,
        "github_url"
    );
    assert_eq!(
        telemetry
            .track_option_example(Some("https://user:secret@example.com/private"))
            .unwrap()
            .value,
        "other_url"
    );
    assert_eq!(
        telemetry
            .track_option_example(Some("git@github.com:acme/private"))
            .unwrap()
            .value,
        "official"
    );
    assert_eq!(
        telemetry
            .track_option_example_path(Some("private/path/with-secret"))
            .unwrap()
            .value,
        "provided"
    );
}

#[test]
fn create_turbo_optional_events_match_typescript_truthiness() {
    let transport = RecordingTransport::default();
    let (_temp, client) = client(PackageKind::CreateTurbo, false, 20, transport);
    let mut telemetry = CreateTurboTelemetry::new(client);

    assert!(telemetry.track_option_example(None).is_none());
    assert!(telemetry.track_option_skip_install(Some(false)).is_none());
    assert!(telemetry.track_option_skip_transforms(Some(false)).is_none());
    assert_eq!(
        telemetry.track_option_skip_install(Some(true)).unwrap().value,
        "true"
    );
    assert_eq!(
        telemetry.track_argument_directory(true).unwrap().value,
        "provided"
    );
    assert!(telemetry.track_argument_directory(false).is_none());
}

#[test]
fn turbo_ignore_task_allowlist_matches_typescript() {
    let transport = RecordingTransport::default();
    let (_temp, client) = client(PackageKind::TurboIgnore, false, 20, transport);
    let mut telemetry = TurboIgnoreTelemetry::new(client);

    for task in [
        "build",
        "test",
        "lint",
        "typecheck",
        "checktypes",
        "check-types",
        "type-check",
        "check",
    ] {
        assert_eq!(telemetry.track_option_task(Some(task)).unwrap().value, task);
    }
    assert_eq!(
        telemetry.track_option_task(Some("workspace#build")).unwrap().value,
        "other"
    );
    assert!(telemetry.track_option_task(None).is_none());
}

#[test]
fn turbo_ignore_tracks_presence_without_workspace_or_directory_values() {
    let transport = RecordingTransport::default();
    let (_temp, client) = client(PackageKind::TurboIgnore, false, 20, transport);
    let mut telemetry = TurboIgnoreTelemetry::new(client);

    assert_eq!(
        telemetry.track_argument_workspace(true).unwrap().value,
        "provided"
    );
    assert_eq!(
        telemetry.track_option_directory(Some("/private/path")).unwrap().value,
        "custom"
    );
    assert!(telemetry.track_argument_workspace(false).is_none());
    assert!(telemetry.track_option_directory(None).is_none());
}

#[test]
fn turbo_ignore_ci_and_max_buffer_values_match_typescript() {
    let transport = RecordingTransport::default();
    let (_temp, client) = client(PackageKind::TurboIgnore, false, 20, transport);
    let mut telemetry = TurboIgnoreTelemetry::new(client);

    assert_eq!(telemetry.track_ci(None).value, "unknown");
    assert_eq!(telemetry.track_ci(Some("GitHub Actions")).value, "GitHub Actions");
    assert_eq!(telemetry.track_option_max_buffer(Some(0)).unwrap().value, "0");
    assert!(telemetry.track_option_max_buffer(None).is_none());
}
