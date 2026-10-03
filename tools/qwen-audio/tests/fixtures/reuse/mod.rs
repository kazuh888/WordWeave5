#![allow(dead_code)]
// Shared independent oracle: no production serializers or parsers construct expectations.
use base64::Engine;
use qwen_audio::*;
use serde_json::{json, Value};
use std::sync::{
    atomic::{AtomicBool, AtomicUsize, Ordering},
    Arc, Mutex,
};

pub const REFERENCE: &str = "The sun is bright.";
pub const KEY: &str = "synthetic-integration-secret-never-real";
pub const HOST: &str = "https://fixture.ap-northeast-1.maas.aliyuncs.com/compatible-mode/v1";
pub const OTHER_HOST: &str =
    "https://replacement.ap-northeast-1.maas.aliyuncs.com/compatible-mode/v1";
pub const ASSESSED: &str = include_str!("assessed.json");
pub const UNASSESSABLE: &str = include_str!("unassessable.json");

pub fn wav(channels: u16, rate: u32, frames: usize) -> Vec<u8> {
    let data_len = frames * channels as usize * 2;
    let mut b = Vec::new();
    b.extend_from_slice(b"RIFF");
    b.extend_from_slice(&(36 + data_len as u32).to_le_bytes());
    b.extend_from_slice(b"WAVEfmt ");
    b.extend_from_slice(&16u32.to_le_bytes());
    b.extend_from_slice(&1u16.to_le_bytes());
    b.extend_from_slice(&channels.to_le_bytes());
    b.extend_from_slice(&rate.to_le_bytes());
    b.extend_from_slice(&(rate * channels as u32 * 2).to_le_bytes());
    b.extend_from_slice(&(channels * 2).to_le_bytes());
    b.extend_from_slice(&16u16.to_le_bytes());
    b.extend_from_slice(b"data");
    b.extend_from_slice(&(data_len as u32).to_le_bytes());
    b.resize(44 + data_len, 0);
    b
}
pub fn audio() -> Vec<u8> {
    wav(1, 8_000, 8)
}
pub fn connection() -> Connection {
    Connection::new(
        TokyoHost::parse(HOST).unwrap(),
        ApiKey::new(KEY.to_owned()).unwrap(),
    )
}
pub fn snapshot() -> EvaluationSnapshot {
    EvaluationSnapshot::new(
        AudioInput::parse(audio()).unwrap(),
        ReferenceText::new(REFERENCE.into()).unwrap(),
        connection(),
    )
    .unwrap()
}
pub fn sse(feedback: &str) -> Vec<u8> {
    format!(
        "data: {}\r\n\r\ndata: {}\r\n\r\ndata: [DONE]\r\n\r\n",
        json!({"choices":[{"index":0,"delta":{"content":feedback},"finish_reason":null}]}),
        json!({"choices":[{"index":0,"delta":{},"finish_reason":"stop"}]})
    )
    .into_bytes()
}
pub fn expected(unassessable: bool) -> EvaluationResult {
    EvaluationResult {
        feedback: Feedback {
            assessment: if unassessable {
                Assessment::Unassessable
            } else {
                Assessment::Assessed
            },
            heard_text: (!unassessable).then(|| REFERENCE.to_owned()),
            summary: if unassessable {
                "今回は評価できない。"
            } else {
                "語尾まで聞き取れる。"
            }
            .into(),
            strengths: if unassessable {
                vec![]
            } else {
                vec!["落ち着いて読めている。".into()]
            },
            improvements: if unassessable {
                vec![]
            } else {
                vec![Improvement {
                    reference_excerpt: "bright".into(),
                    observation: "強勢を練習する。".into(),
                    practice: "brightを強めに読む。".into(),
                }]
            },
            unassessable_reason: unassessable.then(|| "音声から英文を聞き取れない。".into()),
        },
        requested_model: "qwen3.8-omni-flash".into(),
        requested_effort: "medium".into(),
        actual_model: None,
        actual_effort: None,
        usage: None,
    }
}
pub struct Store {
    pub value: Mutex<Option<Connection>>,
    pub loads: AtomicUsize,
    pub saves: AtomicUsize,
    pub fail_load: AtomicBool,
    pub fail_save: AtomicBool,
}
impl Store {
    pub fn new(value: Option<Connection>) -> Arc<Self> {
        Arc::new(Self {
            value: Mutex::new(value),
            loads: AtomicUsize::new(0),
            saves: AtomicUsize::new(0),
            fail_load: AtomicBool::new(false),
            fail_save: AtomicBool::new(false),
        })
    }
    pub fn configured() -> Arc<Self> {
        Self::new(Some(connection()))
    }
}
impl CredentialStore for Store {
    fn load(&self) -> Result<Option<Connection>, SafeError> {
        self.loads.fetch_add(1, Ordering::SeqCst);
        if self.fail_load.load(Ordering::SeqCst) {
            return Err(SafeError::new(
                ErrorCode::CredentialRead,
                SendDisposition::NotSent,
            ));
        }
        Ok(self.value.lock().unwrap().clone())
    }
    fn save(&self, value: &Connection) -> Result<(), SafeError> {
        self.saves.fetch_add(1, Ordering::SeqCst);
        if self.fail_save.load(Ordering::SeqCst) {
            return Err(SafeError::new(
                ErrorCode::CredentialWrite,
                SendDisposition::NotSent,
            ));
        }
        *self.value.lock().unwrap() = Some(value.clone());
        Ok(())
    }
}
#[derive(Clone)]
pub enum Reply {
    Feedback(&'static str),
    Status(u16),
    Pending,
    Network,
    Raw(Vec<u8>),
}
pub struct Wire {
    reply: Reply,
    pub requests: Mutex<Vec<(String, Value)>>,
    pub started: tokio::sync::Notify,
    pub dropped: Arc<AtomicUsize>,
}
struct DropProbe(Arc<AtomicUsize>);
impl Drop for DropProbe {
    fn drop(&mut self) {
        self.0.fetch_add(1, Ordering::SeqCst);
    }
}
impl Wire {
    pub fn new(reply: Reply) -> Arc<Self> {
        Arc::new(Self {
            reply,
            requests: Mutex::new(vec![]),
            started: tokio::sync::Notify::new(),
            dropped: Arc::new(AtomicUsize::new(0)),
        })
    }
    pub fn count(&self) -> usize {
        self.requests.lock().unwrap().len()
    }
    pub fn assert_request(&self, index: usize, host: &str, bytes: &[u8]) {
        let requests = self.requests.lock().unwrap();
        let (endpoint, body) = &requests[index];
        assert_eq!(endpoint, &format!("{host}/chat/completions"));
        assert_eq!(body["model"], "qwen3.8-omni-flash");
        assert_eq!(body["reasoning_effort"], "medium");
        assert_eq!(body["messages"][1]["content"][0]["text"], REFERENCE);
        let data = body["messages"][1]["content"][1]["input_audio"]["data"]
            .as_str()
            .unwrap();
        assert_eq!(
            base64::engine::general_purpose::STANDARD
                .decode(data.strip_prefix("data:;base64,").unwrap())
                .unwrap(),
            bytes
        );
        assert!(!body.to_string().contains(KEY));
    }
}
impl Transport for Wire {
    fn send<'a>(&'a self, request: ProviderRequest) -> TransportFuture<'a> {
        // Record at the transport entry, not merely when its future is first polled.
        assert!(!format!("{request:?}").contains(KEY));
        self.requests.lock().unwrap().push((
            request.endpoint().to_owned(),
            serde_json::from_slice(request.body()).unwrap(),
        ));
        Box::pin(async move {
            let _drop = DropProbe(self.dropped.clone());
            self.started.notify_one();
            let (status, bytes) = match &self.reply {
                Reply::Feedback(f) => (200, sse(f)),
                Reply::Status(s) => (*s, KEY.as_bytes().to_vec()),
                Reply::Pending => return std::future::pending().await,
                Reply::Network => return Err(TransportFailure::Network),
                Reply::Raw(b) => (200, b.clone()),
            };
            Ok(TransportResponse {
                status,
                body: Box::pin(futures_util::stream::iter([Ok(bytes)])),
            })
        })
    }
}
