use std::{
    sync::{Arc, Mutex},
    time::Duration,
};

use tempfile::TempDir;
use turbopath::AbsoluteSystemPathBuf;
use turborepo_telemetry::events::package::{
    CreateTurboTelemetry, PackageInfo, PackageKind, PackageRuntimeInfo, PackageSendFuture,
    PackageTelemetryClient, PackageTelemetryConfig, PackageTelemetryOptions,
    PackageTelemetryRequest, PackageTelemetryTransport, TurboIgnoreTelemetry,
    environment_value_is_truthy,
};

trait TestMust {
    type Output;

    fn test_must(self, context: &str) -> Self::Output;
}

impl<T, E> TestMust for Result<T, E>
where
    E: std::fmt::Debug,
{
    type Output = T;

    fn test_must(self, context: &str) -> Self::Output {
        self.unwrap_or_else(|error| panic!("{context}: {error:?}"))
    }
}

impl<T> TestMust for Option<T> {
    type Output = T;

    fn test_must(self, context: &str) -> Self::Output {
        self.unwrap_or_else(|| panic!("{context}"))
    }
}

#[derive(Clone, Default)]
struct RecordingTransport {
    requests: Arc<Mutex<Vec<PackageTelemetryRequest>>>,
}

impl RecordingTransport {
    fn requests(&self) -> Vec<PackageTelemetryRequest> {
        self.requests
            .lock()
            .test_must("telemetry test invariant")
            .clone()
    }
}

impl PackageTelemetryTransport for RecordingTransport {
    fn send(&self, request: PackageTelemetryRequest) -> PackageSendFuture {
        self.requests
            .lock()
            .test_must("telemetry test invariant")
            .push(request);
        Box::pin(async {})
    }
}

fn config() -> (TempDir, PackageTelemetryConfig) {
    let temp = tempfile::tempdir().test_must("telemetry test invariant");
    let root = AbsoluteSystemPathBuf::try_from(temp.path()).test_must("telemetry test invariant");
    let path = root.join_component("telemetry.json");
    std::fs::write(
        path.as_path(),
        r#"{
  "telemetry_enabled": true,
  "telemetry_id": "telemetry-test-id",
  "telemetry_salt": "private-salt"
}"#,
    )
    .test_must("telemetry test invariant");
    let config = PackageTelemetryConfig::new(path).test_must("telemetry test invariant");
    (temp, config)
}

fn secure_client(
    transport: RecordingTransport,
) -> (TempDir, PackageTelemetryClient<RecordingTransport>) {
    let (temp, config) = config();
    let client = PackageTelemetryClient::new(
        "https://example.com",
        PackageInfo::new(PackageKind::CreateTurbo, "1.0.0").test_must("telemetry test invariant"),
        PackageRuntimeInfo::new("v20.11.30", "Linux", "x64").test_must("telemetry test invariant"),
        config,
        PackageTelemetryOptions::new(20, Duration::from_millis(250))
            .test_must("telemetry test invariant"),
        transport,
    )
    .test_must("telemetry test invariant");
    (temp, client)
}

#[test]
fn endpoint_rejects_credentials_fragments_and_non_https_schemes() {
    for endpoint in [
        "http://example.com",
        "file:///tmp/collector",
        "https://user:secret@example.com",
        "https://example.com/#fragment",
    ] {
        let (_temp, config) = config();
        let result = PackageTelemetryClient::new(
            endpoint,
            PackageInfo::new(PackageKind::CreateTurbo, "1.0.0")
                .test_must("telemetry test invariant"),
            PackageRuntimeInfo::new("v20.11.30", "Linux", "x64")
                .test_must("telemetry test invariant"),
            config,
            PackageTelemetryOptions::default(),
            RecordingTransport::default(),
        );
        assert!(result.is_err(), "accepted unsafe endpoint {endpoint}");
    }
}

#[test]
fn metadata_rejects_header_and_terminal_control_injection() {
    let (_temp, config) = config();
    assert!(PackageInfo::new(PackageKind::CreateTurbo, "1.0.0\r\nX-Evil: yes").is_err());
    assert!(PackageRuntimeInfo::new("v20\nsecret", "Linux", "x64").is_err());
    assert!(
        PackageTelemetryClient::new(
            "https://example.com",
            PackageInfo::new(PackageKind::CreateTurbo, "1.0.0")
                .test_must("telemetry test invariant"),
            PackageRuntimeInfo::new("v20.11.30", "Linux", "x64")
                .test_must("telemetry test invariant"),
            config,
            PackageTelemetryOptions::default(),
            RecordingTransport::default(),
        )
        .is_ok()
    );
}

#[test]
fn invalid_batch_and_timeout_limits_are_rejected() {
    assert!(PackageTelemetryOptions::new(0, Duration::from_millis(250)).is_err());
    assert!(PackageTelemetryOptions::new(10_000, Duration::from_millis(250)).is_err());
    assert!(PackageTelemetryOptions::new(20, Duration::ZERO).is_err());
    assert!(PackageTelemetryOptions::new(20, Duration::from_secs(60)).is_err());
}

#[tokio::test]
async fn credential_bearing_example_never_enters_payload() {
    let transport = RecordingTransport::default();
    let (_temp, client) = secure_client(transport.clone());
    let mut telemetry = CreateTurboTelemetry::new(client);

    telemetry.track_option_example(Some(
        "https://user:ghp_secret@github.com/acme/private?token=secret#secret",
    ));
    telemetry.track_option_example_path(Some("private/path/with-secret"));
    telemetry.close().await;

    let requests = transport.requests();
    let serialized =
        serde_json::to_string(&requests[0].events).test_must("telemetry test invariant");
    for secret in [
        "ghp_secret",
        "token=secret",
        "private/path/with-secret",
        "user:ghp_secret",
    ] {
        assert!(!serialized.contains(secret), "payload leaked {secret}");
    }
}

#[tokio::test]
async fn warnings_errors_and_fallback_refs_are_not_sent_verbatim() {
    let transport = RecordingTransport::default();
    let (_temp, config) = config();
    let client = PackageTelemetryClient::new(
        "https://example.com",
        PackageInfo::new(PackageKind::TurboIgnore, "1.0.0").test_must("telemetry test invariant"),
        PackageRuntimeInfo::new("v20.11.30", "Linux", "x64").test_must("telemetry test invariant"),
        config,
        PackageTelemetryOptions::new(20, Duration::from_millis(250))
            .test_must("telemetry test invariant"),
        transport.clone(),
    )
    .test_must("telemetry test invariant");
    let mut telemetry = TurboIgnoreTelemetry::new(client);

    let warning = telemetry.track_command_warning("token=super-secret");
    let error = telemetry.track_command_error("Authorization: Bearer secret");
    let fallback = telemetry
        .track_option_fallback(Some("refs/heads/customer-private-branch"))
        .test_must("telemetry test invariant");

    assert_ne!(warning.value, "token=super-secret");
    assert_ne!(error.value, "Authorization: Bearer secret");
    assert_eq!(fallback.value, "provided");

    telemetry.close().await;
    let requests = transport.requests();
    let serialized =
        serde_json::to_string(&requests[0].events).test_must("telemetry test invariant");
    assert!(!serialized.contains("super-secret"));
    assert!(!serialized.contains("Bearer secret"));
    assert!(!serialized.contains("customer-private-branch"));
}

#[test]
fn case_insensitive_opt_out_and_debug_values_match_typescript() {
    for value in ["1", "true", "TRUE", "True", "tRuE"] {
        assert!(environment_value_is_truthy(value));
    }
    for value in ["0", "false", "yes", "", " true "] {
        assert!(!environment_value_is_truthy(value));
    }
}

#[cfg(unix)]
#[test]
fn symlinked_config_is_rejected_without_following_the_link() {
    use std::os::unix::fs::symlink;

    let temp = tempfile::tempdir().test_must("telemetry test invariant");
    let root = AbsoluteSystemPathBuf::try_from(temp.path()).test_must("telemetry test invariant");
    let target = root.join_component("target.json");
    let link = root.join_component("telemetry.json");
    std::fs::write(
        target.as_path(),
        r#"{"telemetry_enabled":true,"telemetry_id":"id","telemetry_salt":"salt"}"#,
    )
    .test_must("telemetry test invariant");
    symlink(target.as_path(), link.as_path()).test_must("telemetry test invariant");

    assert!(PackageTelemetryConfig::new(link).is_err());
    assert!(target.exists());
}
