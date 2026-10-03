use std::sync::{Arc, Mutex};

use qwen_audio::{
    probe_connection, ApiHost, ApiKey, Connection, ProbeError, ProbeRequest, ProbeSuccess,
    ProbeTransport, Region, TransportFailure, TransportFuture, TransportResponse,
};
use serde_json::{json, Value};
use tokio_util::sync::CancellationToken;

const KEY: &str = "synthetic-probe-secret-DO-NOT-EMIT";

enum Reply {
    Chunks(u16, Vec<Result<Vec<u8>, TransportFailure>>),
    HeadersPending,
    BodyPending,
    Failure(TransportFailure),
}

struct ProbeFixture {
    reply: Mutex<Option<Reply>>,
    requests: Mutex<Vec<(String, String)>>,
    started: tokio::sync::Notify,
}

impl ProbeFixture {
    fn new(reply: Reply) -> Arc<Self> {
        Arc::new(Self {
            reply: Mutex::new(Some(reply)),
            requests: Mutex::new(Vec::new()),
            started: tokio::sync::Notify::new(),
        })
    }

    fn json(value: Value) -> Arc<Self> {
        Self::new(Reply::Chunks(
            200,
            vec![Ok(serde_json::to_vec(&value).unwrap())],
        ))
    }

    fn count(&self) -> usize {
        self.requests.lock().unwrap().len()
    }
}

impl ProbeTransport for ProbeFixture {
    fn send<'a>(&'a self, request: ProbeRequest) -> TransportFuture<'a> {
        Box::pin(async move {
            self.requests
                .lock()
                .unwrap()
                .push((request.endpoint().to_owned(), format!("{request:?}")));
            self.started.notify_one();
            let reply = self
                .reply
                .lock()
                .unwrap()
                .take()
                .expect("automatic retry is forbidden");
            match reply {
                Reply::Chunks(status, chunks) => Ok(TransportResponse {
                    status,
                    body: Box::pin(futures_util::stream::iter(chunks)),
                }),
                Reply::HeadersPending => std::future::pending().await,
                Reply::BodyPending => Ok(TransportResponse {
                    status: 200,
                    body: Box::pin(futures_util::stream::pending()),
                }),
                Reply::Failure(error) => Err(error),
            }
        })
    }
}

fn connection(region: Region) -> Connection {
    Connection::new(
        ApiHost::parse_for_region(
            &format!("https://probe-fixture.{}.maas.aliyuncs.com", region.id()),
            region,
        )
        .unwrap(),
        ApiKey::new(KEY.to_owned()).unwrap(),
    )
}

fn available() -> Value {
    json!({"success": true, "output": {"models": [{"model": "qwen3.8-omni-flash"}]}})
}

// QU-AC-012/013/016: all six approved Workspace authorities are preserved.
// The fake observes the endpoint; GET/no body/redirect policy is a separate
// ReqwestProbeTransport source check, since those are not public fake fields.
#[tokio::test]
async fn each_region_uses_one_same_authority_model_list_request_without_private_data() {
    for region in Region::ALL {
        let transport = ProbeFixture::json(available());
        let original = connection(region);
        let expected_origin = original
            .host()
            .as_str()
            .strip_suffix("/compatible-mode/v1")
            .unwrap();
        let expected = format!(
            "{expected_origin}/api/v1/models?model=qwen3.8-omni-flash&page_no=1&page_size=100"
        );
        assert_eq!(
            probe_connection(
                original.clone(),
                transport.clone(),
                CancellationToken::new()
            )
            .await,
            Ok(ProbeSuccess)
        );
        let requests = transport.requests.lock().unwrap();
        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0].0, expected);
        assert!(!requests[0].0.contains(KEY));
        assert!(!requests[0].0.contains("chat/completions"));
        assert!(!requests[0].1.contains(KEY));
        assert_eq!(
            original.host().region(),
            region,
            "probe does not mutate saved input"
        );
    }
    let message = ProbeSuccess.message();
    assert!(message.contains("モデル一覧"));
    for limit in ["音声評価", "推論権限", "費用", "品質"] {
        assert!(message.contains(limit));
    }
}

// QU-AC-014: a list without the exact identifier is a distinct safe outcome.
#[tokio::test]
async fn model_absence_empty_list_and_aliases_are_not_confirmed_as_supported() {
    for models in [
        json!([]),
        json!([{"model":"qwen3.8-omni"}]),
        json!([{"model":"qwen3.8-omni-flash-latest"}]),
        json!([{"model":" Qwen3.8-Omni-Flash "}]),
    ] {
        let transport = ProbeFixture::json(json!({"success":true,"output":{"models":models}}));
        assert_eq!(
            probe_connection(
                connection(Region::Tokyo),
                transport.clone(),
                CancellationToken::new()
            )
            .await,
            Err(ProbeError::ModelNotFound)
        );
        assert_eq!(transport.count(), 1);
    }
}

#[tokio::test]
async fn malformed_model_list_never_becomes_success_even_with_one_matching_entry() {
    for value in [
        json!(null),
        json!({"output":{"models":[]}}),
        json!({"success":"true","output":{"models":[]}}),
        json!({"success":true}),
        json!({"success":true,"output":{"models":{}}}),
        json!({"success":true,"output":{"models":[{"id":"qwen3.8-omni-flash"}]}}),
        json!({"success":true,"output":{"models":[{"model":42},{"model":"qwen3.8-omni-flash"}]}}),
    ] {
        let transport = ProbeFixture::json(value);
        assert_eq!(
            probe_connection(
                connection(Region::Tokyo),
                transport.clone(),
                CancellationToken::new()
            )
            .await,
            Err(ProbeError::ResponseInvalid)
        );
        assert_eq!(transport.count(), 1);
    }
    for bytes in [b"{broken".to_vec(), vec![0xff]] {
        let transport = ProbeFixture::new(Reply::Chunks(200, vec![Ok(bytes)]));
        assert_eq!(
            probe_connection(
                connection(Region::Tokyo),
                transport,
                CancellationToken::new()
            )
            .await,
            Err(ProbeError::ResponseInvalid)
        );
    }
}

#[tokio::test]
async fn declared_list_failure_is_provider_failure_not_model_absence() {
    let transport = ProbeFixture::json(json!({"success":false,"output":{"models":[]}}));
    assert_eq!(
        probe_connection(
            connection(Region::Tokyo),
            transport.clone(),
            CancellationToken::new()
        )
        .await,
        Err(ProbeError::Provider)
    );
    assert_eq!(transport.count(), 1);
}

// QU-AC-013/014: no fallback or retry, including a redirect response.
#[tokio::test]
async fn http_failure_classes_are_distinct_safe_and_single_attempt() {
    for (status, expected) in [
        (401, ProbeError::Authentication),
        (403, ProbeError::PermissionDenied),
        (404, ProbeError::Unsupported),
        (405, ProbeError::Unsupported),
        (501, ProbeError::Unsupported),
        (302, ProbeError::Unsupported),
        (307, ProbeError::Unsupported),
        (429, ProbeError::RateLimited),
        (500, ProbeError::Provider),
    ] {
        let raw = format!("{KEY} synthetic-private-response https://attacker.invalid");
        let transport = ProbeFixture::new(Reply::Chunks(status, vec![Ok(raw.into_bytes())]));
        let error = probe_connection(
            connection(Region::Tokyo),
            transport.clone(),
            CancellationToken::new(),
        )
        .await
        .unwrap_err();
        assert_eq!(error, expected, "status {status}");
        assert_eq!(transport.count(), 1);
        let display = format!("{error} {error:?}");
        for secret in [KEY, "synthetic-private-response", "attacker.invalid"] {
            assert!(!display.contains(secret));
        }
    }
}

#[tokio::test]
async fn chunked_model_list_accepts_capacity_boundary_and_rejects_first_excess_byte() {
    let mut bytes = serde_json::to_vec(&available()).unwrap();
    bytes.resize(256 * 1024, b' ');
    let transport = ProbeFixture::new(Reply::Chunks(
        200,
        bytes.chunks(101).map(|part| Ok(part.to_vec())).collect(),
    ));
    assert_eq!(
        probe_connection(
            connection(Region::Tokyo),
            transport.clone(),
            CancellationToken::new()
        )
        .await,
        Ok(ProbeSuccess)
    );
    assert_eq!(transport.count(), 1);
    let transport = ProbeFixture::new(Reply::Chunks(200, vec![Ok(bytes), Ok(vec![b' '])]));
    assert_eq!(
        probe_connection(
            connection(Region::Tokyo),
            transport.clone(),
            CancellationToken::new()
        )
        .await,
        Err(ProbeError::ResponseTooLarge)
    );
    assert_eq!(transport.count(), 1);
}

#[tokio::test]
async fn network_tls_and_body_disconnect_fail_without_automatic_retry() {
    for reply in [
        Reply::Failure(TransportFailure::Network),
        Reply::Failure(TransportFailure::Tls),
        Reply::Chunks(200, vec![Ok(b"{".to_vec()), Err(TransportFailure::Network)]),
    ] {
        let transport = ProbeFixture::new(reply);
        assert_eq!(
            probe_connection(
                connection(Region::Tokyo),
                transport.clone(),
                CancellationToken::new()
            )
            .await,
            Err(ProbeError::Network)
        );
        assert_eq!(transport.count(), 1);
    }
}

#[tokio::test]
async fn cancellation_before_poll_sends_nothing_and_after_send_drops_pending_response() {
    let cancel = CancellationToken::new();
    cancel.cancel();
    let transport = ProbeFixture::json(available());
    assert_eq!(
        probe_connection(connection(Region::Tokyo), transport.clone(), cancel).await,
        Err(ProbeError::Cancelled)
    );
    assert_eq!(transport.count(), 0);
    for reply in [Reply::HeadersPending, Reply::BodyPending] {
        let transport = ProbeFixture::new(reply);
        let cancel = CancellationToken::new();
        let task = tokio::spawn(probe_connection(
            connection(Region::Tokyo),
            transport.clone(),
            cancel.clone(),
        ));
        transport.started.notified().await;
        cancel.cancel();
        assert_eq!(task.await.unwrap(), Err(ProbeError::Cancelled));
        assert_eq!(transport.count(), 1);
    }
}

#[tokio::test(start_paused = true)]
async fn thirty_second_deadline_covers_headers_and_body_without_retry() {
    for reply in [Reply::HeadersPending, Reply::BodyPending] {
        let transport = ProbeFixture::new(reply);
        let task = tokio::spawn(probe_connection(
            connection(Region::Tokyo),
            transport.clone(),
            CancellationToken::new(),
        ));
        transport.started.notified().await;
        tokio::time::advance(std::time::Duration::from_secs(29)).await;
        assert!(!task.is_finished());
        tokio::time::advance(std::time::Duration::from_secs(1)).await;
        assert_eq!(task.await.unwrap(), Err(ProbeError::Timeout));
        assert_eq!(transport.count(), 1);
    }
}
