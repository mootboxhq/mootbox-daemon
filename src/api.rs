use std::time::Duration;

use anyhow::Context as _;
use chrono::{DateTime, Utc};
use reqwest::header::{DATE, HeaderMap};
use reqwest::{Method, RequestBuilder, StatusCode, Url};
use serde::Serialize;
use serde::de::DeserializeOwned;
use uuid::Uuid;

use crate::config::WorkspaceProfile;

use crate::contracts::{
    ClaimPairingCodeRequest, ClaimPairingCodeResponse, ClaimRequest, ClaimResponse, CompleteRunRequest,
    CompleteRunResponse, Error as ErrorBody, ErrorErrorCategory, ExchangePairingRequest, ExchangePairingResponse,
    HealthResponse, HeartbeatRequest, HeartbeatResponse, MatchRepositoriesRequest, MatchRepositoriesResponse,
    RegisterRepositoryRequest, RegisterRepositoryResponse, RunEventRequest, RunEventResponse, StartPairingRequest,
    StartPairingResponse, UpsertConnectionRequest, UpsertConnectionResponse,
};

pub const RUNNER_PROTOCOL_HEADER: &str = "x-mootbox-runner-protocol";
pub const RUNNER_PROTOCOL_VERSION: u32 = 10;

const REQUEST_ID_HEADER: &str = "x-request-id";
const TRACEPARENT_HEADER: &str = "traceparent";
const REQUEST_TIMEOUT: Duration = Duration::from_secs(30);
const SPAN_ID_LENGTH: usize = 16;

#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error(
        "the Mootbox server requires a newer mootbox (this one speaks runner protocol {RUNNER_PROTOCOL_VERSION}): {message}; run mootbox update"
    )]
    UpgradeRequired { message: String },
    #[error("Mootbox refused the request ({category}, HTTP {status}): {message}")]
    Mootbox {
        status: u16,
        category: ErrorErrorCategory,
        message: String,
    },
    #[error("Mootbox answered HTTP {status} without a readable error")]
    UnreadableError { status: u16 },
    #[error("Mootbox answered HTTP {status} with an unexpected body: {source}")]
    UnexpectedBody { status: u16, source: serde_json::Error },
    #[error("cannot reach Mootbox: {0}")]
    Transport(#[from] reqwest::Error),
    #[error("cannot build the Mootbox URL for {path}: {reason}")]
    InvalidRoute { path: String, reason: String },
}

pub enum ServerHealth {
    Reported(HealthResponse),
    RouteMissing { server_date: Option<DateTime<Utc>> },
}

struct Call<'a, Request> {
    method: Method,
    path: String,
    body: Option<&'a Request>,
    trace_id: String,
    timeout: Duration,
}

impl<'a, Request> Call<'a, Request> {
    fn new(method: Method, path: impl Into<String>, body: &'a Request) -> Self {
        Self::build(method, path, Some(body))
    }

    fn without_body(method: Method, path: impl Into<String>) -> Self {
        Self::build(method, path, None)
    }

    fn build(method: Method, path: impl Into<String>, body: Option<&'a Request>) -> Self {
        Self {
            method,
            path: path.into(),
            body,
            trace_id: Uuid::new_v4().simple().to_string(),
            timeout: REQUEST_TIMEOUT,
        }
    }

    fn for_run(run_id: Uuid, action: &str, body: &'a Request) -> Self {
        Self {
            trace_id: run_id.simple().to_string(),
            ..Self::new(Method::POST, format!("/runner/runs/{run_id}/{action}"), body)
        }
    }
}

pub fn traceparent(trace_id: &str) -> String {
    let span_id = &Uuid::new_v4().simple().to_string()[..SPAN_ID_LENGTH];
    format!("00-{trace_id}-{span_id}-01")
}

pub fn claim_timeout(wait_ms: i64) -> Duration {
    REQUEST_TIMEOUT + Duration::from_millis(u64::try_from(wait_ms).unwrap_or_default())
}

impl ApiError {
    pub fn is_rejection(&self) -> bool {
        matches!(
            self,
            Self::UpgradeRequired { .. } | Self::Mootbox { .. } | Self::UnreadableError { .. }
        )
    }

    pub fn is_lease_lost(&self) -> bool {
        matches!(
            self,
            Self::Mootbox {
                category: ErrorErrorCategory::LeaseLost,
                ..
            }
        )
    }
}

pub struct MootboxClient {
    http: reqwest::Client,
    base_url: Url,
    credential: Option<String>,
}

impl MootboxClient {
    pub fn unauthenticated(base_url: Url) -> Self {
        Self::new(base_url, None)
    }

    pub fn authenticated(base_url: Url, credential: String) -> Self {
        Self::new(base_url, Some(credential))
    }

    pub fn for_profile(profile: &WorkspaceProfile) -> anyhow::Result<Self> {
        Ok(Self::authenticated(
            mootbox_url_of(profile)?,
            profile.credential.clone(),
        ))
    }

    pub fn new(base_url: Url, credential: Option<String>) -> Self {
        let http = reqwest::Client::new();
        Self {
            http,
            base_url,
            credential,
        }
    }

    pub async fn start_pairing(&self, request: &StartPairingRequest) -> Result<StartPairingResponse, ApiError> {
        self.send(Call::new(Method::POST, "/runner/pairings", request)).await
    }

    pub async fn claim_pairing_code(
        &self,
        request: &ClaimPairingCodeRequest,
    ) -> Result<ClaimPairingCodeResponse, ApiError> {
        self.send(Call::new(Method::POST, "/runner/pairings/claim-code", request))
            .await
    }

    pub async fn exchange_pairing(
        &self,
        pairing_id: Uuid,
        request: &ExchangePairingRequest,
    ) -> Result<ExchangePairingResponse, ApiError> {
        let path = format!("/runner/pairings/{pairing_id}/exchange");
        self.send(Call::new(Method::POST, path, request)).await
    }

    pub async fn match_repository(
        &self,
        request: &MatchRepositoriesRequest,
    ) -> Result<MatchRepositoriesResponse, ApiError> {
        self.send(Call::new(Method::POST, "/runner/repositories/match", request))
            .await
    }

    pub async fn register_repository(
        &self,
        request: &RegisterRepositoryRequest,
    ) -> Result<RegisterRepositoryResponse, ApiError> {
        self.send(Call::new(Method::POST, "/runner/repositories", request))
            .await
    }

    pub async fn upsert_connection(
        &self,
        request: &UpsertConnectionRequest,
    ) -> Result<UpsertConnectionResponse, ApiError> {
        self.send(Call::new(Method::PUT, "/runner/connections", request)).await
    }

    pub async fn claim(&self, request: &ClaimRequest) -> Result<ClaimResponse, ApiError> {
        let call = Call {
            timeout: claim_timeout(request.wait_ms),
            ..Call::new(Method::POST, "/runner/claim", request)
        };
        self.send(call).await
    }

    pub async fn heartbeat(&self, run_id: Uuid, request: &HeartbeatRequest) -> Result<HeartbeatResponse, ApiError> {
        self.send(Call::for_run(run_id, "heartbeat", request)).await
    }

    pub async fn record_event(&self, run_id: Uuid, request: &RunEventRequest) -> Result<RunEventResponse, ApiError> {
        self.send(Call::for_run(run_id, "events", request)).await
    }

    pub async fn complete(&self, run_id: Uuid, request: &CompleteRunRequest) -> Result<CompleteRunResponse, ApiError> {
        self.send(Call::for_run(run_id, "complete", request)).await
    }

    pub async fn health(&self) -> Result<ServerHealth, ApiError> {
        let call = Call::<()>::without_body(Method::GET, "/runner/health");
        let response = self.request(call)?.send().await?;
        let status = response.status();
        let server_date = server_date_of(response.headers());
        let bytes = response.bytes().await?;
        if status.is_success() {
            return body_of(status, &bytes).map(ServerHealth::Reported);
        }
        match error_from_body(status, &bytes) {
            ApiError::Mootbox {
                category: ErrorErrorCategory::NotFound,
                ..
            } => Ok(ServerHealth::RouteMissing { server_date }),
            error => Err(error),
        }
    }

    fn request<Request: Serialize>(&self, call: Call<'_, Request>) -> Result<RequestBuilder, ApiError> {
        let url = self.base_url.join(&call.path).map_err(|error| ApiError::InvalidRoute {
            path: call.path.clone(),
            reason: error.to_string(),
        })?;
        let request = self
            .http
            .request(call.method, url)
            .timeout(call.timeout)
            .header(RUNNER_PROTOCOL_HEADER, RUNNER_PROTOCOL_VERSION.to_string())
            .header(REQUEST_ID_HEADER, Uuid::new_v4().to_string())
            .header(TRACEPARENT_HEADER, traceparent(&call.trace_id));
        let request = match call.body {
            Some(body) => request.json(body),
            None => request,
        };
        Ok(match &self.credential {
            Some(credential) => request.bearer_auth(credential),
            None => request,
        })
    }

    async fn send<Request: Serialize, Response: DeserializeOwned>(
        &self,
        call: Call<'_, Request>,
    ) -> Result<Response, ApiError> {
        let response = self.request(call)?.send().await?;
        let status = response.status();
        let bytes = response.bytes().await?;
        if !status.is_success() {
            return Err(error_from_body(status, &bytes));
        }
        body_of(status, &bytes)
    }
}

pub fn mootbox_url_of(profile: &WorkspaceProfile) -> anyhow::Result<Url> {
    Url::parse(&profile.mootbox_url).with_context(|| {
        format!(
            "mootbox_url {:?} in the daemon config is not a URL",
            profile.mootbox_url
        )
    })
}

fn body_of<Response: DeserializeOwned>(status: StatusCode, bytes: &[u8]) -> Result<Response, ApiError> {
    serde_json::from_slice(bytes).map_err(|source| ApiError::UnexpectedBody {
        status: status.as_u16(),
        source,
    })
}

fn server_date_of(headers: &HeaderMap) -> Option<DateTime<Utc>> {
    let text = headers.get(DATE)?.to_str().ok()?;
    DateTime::parse_from_rfc2822(text)
        .ok()
        .map(|date| date.with_timezone(&Utc))
}

fn error_from_body(status: StatusCode, bytes: &[u8]) -> ApiError {
    let Ok(body) = serde_json::from_slice::<ErrorBody>(bytes) else {
        return ApiError::UnreadableError {
            status: status.as_u16(),
        };
    };
    let message = body.error.message;
    match body.error.category {
        ErrorErrorCategory::UpgradeRequired => ApiError::UpgradeRequired { message },
        category => ApiError::Mootbox {
            status: status.as_u16(),
            category,
            message,
        },
    }
}
