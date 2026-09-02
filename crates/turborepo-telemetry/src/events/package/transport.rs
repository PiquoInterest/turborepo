use std::{future::Future, pin::Pin};

use reqwest::redirect::Policy;
use tracing::debug;

use super::types::{PackageTelemetryError, PackageTelemetryRequest};

pub type PackageSendFuture = Pin<Box<dyn Future<Output = ()> + Send + 'static>>;

pub trait PackageTelemetryTransport: Clone + Send + Sync + 'static {
    fn send(&self, request: PackageTelemetryRequest) -> PackageSendFuture;
}

#[derive(Clone)]
pub struct ReqwestPackageTelemetryTransport {
    client: reqwest::Client,
}

impl ReqwestPackageTelemetryTransport {
    pub fn new() -> Result<Self, PackageTelemetryError> {
        let client = reqwest::Client::builder()
            .redirect(Policy::none())
            .build()
            .map_err(PackageTelemetryError::HttpClient)?;
        Ok(Self { client })
    }
}

impl PackageTelemetryTransport for ReqwestPackageTelemetryTransport {
    fn send(&self, request: PackageTelemetryRequest) -> PackageSendFuture {
        let client = self.client.clone();
        let task = async move {
            let send = client
                .post(request.endpoint)
                .header("Content-Type", "application/json")
                .header("x-turbo-telemetry-id", request.telemetry_id)
                .header("x-turbo-session-id", request.session_id)
                .header("User-Agent", request.user_agent)
                .json(&request.events)
                .send();

            match tokio::time::timeout(request.timeout, send).await {
                Ok(Ok(response)) => {
                    if let Err(error) = response.error_for_status() {
                        debug!("package telemetry request failed: {error}");
                    }
                }
                Ok(Err(error)) => debug!("package telemetry request failed: {error}"),
                Err(_) => debug!("package telemetry request timed out"),
            }
        };

        match tokio::runtime::Handle::try_current() {
            Ok(handle) => {
                let join = handle.spawn(task);
                Box::pin(async move {
                    if let Err(error) = join.await {
                        debug!("package telemetry task failed: {error}");
                    }
                })
            }
            Err(_) => {
                debug!("package telemetry dropped because no Tokio runtime is active");
                Box::pin(async {})
            }
        }
    }
}
