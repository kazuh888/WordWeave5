use std::sync::{Arc, Mutex};

use qwen_audio::{
    ApiKey, AudioInput, Connection, EvaluationSnapshot, ProviderRequest, ReferenceText, TokyoHost,
    Transport, TransportFailure, TransportFuture, TransportResponse,
};
use serde_json::{json, Value};

pub const KEY: &str = "synthetic-secret-sentinel-DO-NOT-EMIT";
pub const BASE: &str = "https://fixture.ap-northeast-1.maas.aliyuncs.com/compatible-mode/v1";
pub const REFERENCE: &str = "The sun is bright.";

pub fn connection() -> Connection {
    Connection::new(
        TokyoHost::parse(BASE).unwrap(),
        ApiKey::new(KEY.to_owned()).unwrap(),
    )
}

pub fn snapshot() -> EvaluationSnapshot {
    EvaluationSnapshot::new(
        AudioInput::parse(crate::support::wav(1, 8_000, 8)).unwrap(),
        ReferenceText::new(REFERENCE.to_owned()).unwrap(),
        connection(),
    )
    .unwrap()
}

pub fn feedback() -> Value {
    json!({
        "assessment": "assessed", "heard_text": REFERENCE,
        "summary": "語尾まで聞き取れる。", "strengths": ["落ち着いて読んでいる。"],
        "improvements": [{"reference_excerpt": "bright", "observation": "強弱を練習する。", "practice": "brightを強めに読む。"}],
        "unassessable_reason": null
    })
}

pub fn event(value: Value) -> Vec<u8> {
    format!("data: {value}\r\n\r\n").into_bytes()
}

pub fn sse(value: Value) -> Vec<u8> {
    let mut bytes = event(
        json!({"choices":[{"index":0,"delta":{"content":value.to_string()},"finish_reason":null}]}),
    );
    bytes.extend(event(
        json!({"choices":[{"index":0,"delta":{},"finish_reason":"stop"}]}),
    ));
    bytes.extend_from_slice(b"data: [DONE]\r\n\r\n");
    bytes
}

pub enum Reply {
    Chunks(u16, Vec<Result<Vec<u8>, TransportFailure>>),
    HeadersNever,
    BodyNever,
    ChunksThenPending(Vec<Vec<u8>>),
    Fail(TransportFailure),
}

#[derive(Clone, Debug)]
pub struct RecordedRequest {
    pub endpoint: String,
    pub body: Vec<u8>,
    pub debug: String,
}

pub struct ScriptTransport {
    reply: Mutex<Option<Reply>>,
    pub requests: Mutex<Vec<RecordedRequest>>,
    pub started: tokio::sync::Notify,
}

impl ScriptTransport {
    pub fn new(reply: Reply) -> Arc<Self> {
        Arc::new(Self {
            reply: Mutex::new(Some(reply)),
            requests: Mutex::new(Vec::new()),
            started: tokio::sync::Notify::new(),
        })
    }

    pub fn success() -> Arc<Self> {
        Self::new(Reply::Chunks(200, vec![Ok(sse(feedback()))]))
    }

    pub fn count(&self) -> usize {
        self.requests.lock().unwrap().len()
    }
}

impl Transport for ScriptTransport {
    fn send<'a>(&'a self, request: ProviderRequest) -> TransportFuture<'a> {
        Box::pin(async move {
            self.requests.lock().unwrap().push(RecordedRequest {
                endpoint: request.endpoint().to_owned(),
                body: request.body().to_vec(),
                debug: format!("{request:?}"),
            });
            self.started.notify_one();
            let reply = self
                .reply
                .lock()
                .unwrap()
                .take()
                .expect("no automatic retry permitted");
            match reply {
                Reply::Chunks(status, chunks) => Ok(TransportResponse {
                    status,
                    body: Box::pin(futures_util::stream::iter(chunks)),
                }),
                Reply::HeadersNever => std::future::pending().await,
                Reply::BodyNever => Ok(TransportResponse {
                    status: 200,
                    body: Box::pin(futures_util::stream::pending()),
                }),
                Reply::ChunksThenPending(chunks) => {
                    use futures_util::StreamExt;
                    Ok(TransportResponse {
                        status: 200,
                        body: Box::pin(
                            futures_util::stream::iter(chunks.into_iter().map(Ok))
                                .chain(futures_util::stream::pending()),
                        ),
                    })
                }
                Reply::Fail(failure) => Err(failure),
            }
        })
    }
}
