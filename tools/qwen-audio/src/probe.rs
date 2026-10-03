use crate::{
    ApiHost, ApiKey, Connection, TransportFailure, TransportFuture, TransportResponse,
    REQUESTED_MODEL,
};
use futures_util::StreamExt;
use serde::Deserialize;
use std::{sync::Arc, time::Duration};
use tokio_util::sync::CancellationToken;

const MAX_RESPONSE_BYTES: usize = 256 * 1024;
const PROBE_TIMEOUT: Duration = Duration::from_secs(30);

/// A model-list request derived only from an already validated connection.
/// Its key cannot be read by callers or included in diagnostics.
pub struct ProbeRequest {
    endpoint: String,
    key: ApiKey,
}

impl ProbeRequest {
    pub fn endpoint(&self) -> &str {
        &self.endpoint
    }
}

impl std::fmt::Debug for ProbeRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("ProbeRequest([redacted])")
    }
}

pub trait ProbeTransport: Send + Sync {
    fn send<'a>(&'a self, request: ProbeRequest) -> TransportFuture<'a>;
}

pub struct ReqwestProbeTransport {
    client: reqwest::Client,
}

impl ReqwestProbeTransport {
    pub fn new() -> Result<Self, ProbeError> {
        let client = reqwest::Client::builder()
            .redirect(reqwest::redirect::Policy::none())
            .https_only(true)
            .no_proxy()
            .connect_timeout(Duration::from_secs(10))
            .build()
            .map_err(|_| ProbeError::Network)?;
        Ok(Self { client })
    }
}

impl ProbeTransport for ReqwestProbeTransport {
    fn send<'a>(&'a self, request: ProbeRequest) -> TransportFuture<'a> {
        Box::pin(async move {
            let response = self
                .client
                .get(request.endpoint)
                .bearer_auth(request.key.value())
                .send()
                .await
                .map_err(|_| TransportFailure::Network)?;
            Ok(TransportResponse {
                status: response.status().as_u16(),
                body: Box::pin(response.bytes_stream().map(|chunk| {
                    chunk
                        .map(|bytes| bytes.to_vec())
                        .map_err(|_| TransportFailure::Network)
                })),
            })
        })
    }
}

/// Only the model-list connection and the exact requested model were confirmed.
/// This does not establish permission, cost, or quality of audio inference.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ProbeSuccess;

impl ProbeSuccess {
    pub fn message(&self) -> &'static str {
        "モデル一覧への接続を確認した。音声評価の可否・推論権限・費用・品質は保証しない"
    }
}

/// Closed, non-reflecting diagnostics: never retain the response body or key.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProbeError {
    InvalidConnection,
    Authentication,
    PermissionDenied,
    Unsupported,
    RateLimited,
    Provider,
    Network,
    Timeout,
    ResponseTooLarge,
    ResponseInvalid,
    ModelNotFound,
    Cancelled,
}

impl ProbeError {
    pub fn message(&self) -> &'static str {
        match self {
            Self::InvalidConnection => "接続先の形式を確認できない。地域とWorkspace URLを確認する",
            Self::Authentication => "認証が拒否された。この接続先のAPIキーを確認する",
            Self::PermissionDenied => "モデル一覧へのアクセスが拒否された。Workspaceとキーの権限を確認する",
            Self::Unsupported => "この接続先ではモデル一覧APIを利用できない。接続先のAPI対応を確認する",
            Self::RateLimited => "APIの利用制限に達した。時間を置いて接続を再確認する",
            Self::Provider => "接続先がモデル一覧の要求を受理できなかった。接続先の状態とAPI対応を確認する",
            Self::Network => "モデル一覧への通信に失敗した。ネットワークと接続先の状態を確認する",
            Self::Timeout => "接続確認が30秒以内に完了しなかった。時間を置いて再確認する",
            Self::ResponseTooLarge => "モデル一覧の応答が上限を超えた。接続先のAPI対応を確認する",
            Self::ResponseInvalid => "モデル一覧の応答形式を確認できない。接続先のAPI対応を確認する",
            Self::ModelNotFound => "モデル一覧を取得したがqwen3.8-omni-flashを確認できなかった。対象モデルの提供状況を確認する",
            Self::Cancelled => "接続確認を取り消した。必要であれば接続を再確認する",
        }
    }
}

impl std::fmt::Display for ProbeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.message())
    }
}

impl std::error::Error for ProbeError {}

pub async fn probe_connection(
    connection: Connection,
    transport: Arc<dyn ProbeTransport>,
    cancel: CancellationToken,
) -> Result<ProbeSuccess, ProbeError> {
    if cancel.is_cancelled() {
        return Err(ProbeError::Cancelled);
    }
    let host =
        ApiHost::parse(connection.host().as_str()).map_err(|_| ProbeError::InvalidConnection)?;
    let origin = host
        .as_str()
        .strip_suffix("/compatible-mode/v1")
        .ok_or(ProbeError::InvalidConnection)?;
    let request = ProbeRequest {
        endpoint: format!("{origin}/api/v1/models?model={REQUESTED_MODEL}&page_no=1&page_size=100"),
        key: connection.key().clone(),
    };
    let task = async {
        let mut response = transport
            .send(request)
            .await
            .map_err(|_| ProbeError::Network)?;
        match response.status {
            200..=299 => {}
            401 => return Err(ProbeError::Authentication),
            403 => return Err(ProbeError::PermissionDenied),
            404 | 405 | 501 => return Err(ProbeError::Unsupported),
            429 => return Err(ProbeError::RateLimited),
            300..=399 => return Err(ProbeError::Unsupported),
            _ => return Err(ProbeError::Provider),
        }
        let mut body = Vec::new();
        while let Some(chunk) = response.body.next().await {
            let chunk = chunk.map_err(|_| ProbeError::Network)?;
            if chunk.len() > MAX_RESPONSE_BYTES - body.len() {
                return Err(ProbeError::ResponseTooLarge);
            }
            body.extend_from_slice(&chunk);
        }
        parse_models(&body)
    };
    tokio::select! {
        biased;
        _ = cancel.cancelled() => Err(ProbeError::Cancelled),
        _ = tokio::time::sleep(PROBE_TIMEOUT) => Err(ProbeError::Timeout),
        result = task => result,
    }
}

fn parse_models(body: &[u8]) -> Result<ProbeSuccess, ProbeError> {
    #[derive(Deserialize)]
    struct ModelList {
        output: Models,
    }
    #[derive(Deserialize)]
    struct Models {
        models: Vec<Model>,
    }
    #[derive(Deserialize)]
    struct Model {
        model: String,
    }
    let value: serde_json::Value =
        serde_json::from_slice(body).map_err(|_| ProbeError::ResponseInvalid)?;
    match value.get("success").and_then(serde_json::Value::as_bool) {
        Some(true) => {}
        Some(false) => return Err(ProbeError::Provider),
        None => return Err(ProbeError::ResponseInvalid),
    }
    let response: ModelList =
        serde_json::from_value(value).map_err(|_| ProbeError::ResponseInvalid)?;
    if response
        .output
        .models
        .iter()
        .any(|entry| entry.model.trim().is_empty() || entry.model.chars().any(char::is_control))
    {
        return Err(ProbeError::ResponseInvalid);
    }
    if response
        .output
        .models
        .iter()
        .any(|entry| entry.model == REQUESTED_MODEL)
    {
        Ok(ProbeSuccess)
    } else {
        Err(ProbeError::ModelNotFound)
    }
}
