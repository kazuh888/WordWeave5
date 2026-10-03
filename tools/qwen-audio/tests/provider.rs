#[path = "support/provider.rs"]
mod fixture;
mod support;

use fixture::{event, feedback, snapshot, sse, Reply, ScriptTransport, BASE, KEY, REFERENCE};
use qwen_audio::{evaluate, Assessment, ErrorCode, SafeError, SendDisposition, TransportFailure};
use serde_json::{json, Value};
use tokio_util::sync::CancellationToken;

#[tokio::test]
async fn single_request_uses_confirmed_audio_text_model_effort_and_token_limit() {
    use base64::Engine;
    let transport = ScriptTransport::success();
    let result = evaluate(snapshot(), transport.clone(), CancellationToken::new())
        .await
        .unwrap();
    assert_eq!(transport.count(), 1);
    let requests = transport.requests.lock().unwrap();
    let request = &requests[0];
    assert_eq!(request.endpoint, format!("{BASE}/chat/completions"));
    let body: Value = serde_json::from_slice(&request.body).unwrap();
    assert_eq!(body["model"], "qwen3.8-omni-flash");
    assert_eq!(body["reasoning_effort"], "medium");
    assert_eq!(body["max_tokens"], 4096);
    assert_eq!(body["modalities"], json!(["text"]));
    assert_eq!(body["stream"], true);
    assert_eq!(body["stream_options"]["include_usage"], true);
    // QU-AC-003: official JSON Object mode does not replace schema validation.
    assert_eq!(body["response_format"], json!({"type": "json_object"}));
    for forbidden in ["enable_thinking", "max_completion_tokens"] {
        assert!(body.get(forbidden).is_none());
    }
    let user = body["messages"]
        .as_array()
        .unwrap()
        .iter()
        .find(|m| m["role"] == "user")
        .unwrap();
    let content = user["content"].as_array().unwrap();
    assert!(content
        .iter()
        .any(|part| part["type"] == "text" && part["text"].as_str().unwrap().contains(REFERENCE)));
    let audio = content
        .iter()
        .find(|part| part["type"] == "input_audio")
        .unwrap();
    assert_eq!(audio["input_audio"]["format"], "wav");
    let data = audio["input_audio"]["data"]
        .as_str()
        .unwrap()
        .strip_prefix("data:;base64,")
        .unwrap();
    assert_eq!(
        base64::engine::general_purpose::STANDARD
            .decode(data)
            .unwrap(),
        support::wav(1, 8_000, 8)
    );
    assert!(!String::from_utf8_lossy(&request.body).contains(KEY));
    assert!(!request.debug.contains(KEY));
    assert!(!request.debug.contains(REFERENCE));
    assert_eq!(result.requested_model, "qwen3.8-omni-flash");
    assert_eq!(result.requested_effort, "medium");
    assert_eq!(result.actual_model, None);
    assert_eq!(result.actual_effort, None);
    assert_eq!(result.usage, None);
}

// QU-AC-001: an unrelated sentence is a valid mismatch only when declared by
// a schema-conforming result, rather than inferred from a parse failure.
#[tokio::test]
async fn valid_mismatch_preserves_estimated_sentence_without_pronunciation_feedback() {
    let value = json!({
        "assessment": "reference_mismatch",
        "heard_text": "We apologize for the delay.",
        "summary": "別の英文が聞き取れた。例文を読み直す。",
        "strengths": [], "improvements": [], "unassessable_reason": null
    });
    let transport = ScriptTransport::new(Reply::Chunks(200, vec![Ok(sse(value))]));
    let result = evaluate(snapshot(), transport.clone(), CancellationToken::new())
        .await
        .unwrap();
    assert_eq!(result.feedback.assessment, Assessment::ReferenceMismatch);
    assert_eq!(
        result.feedback.heard_text.as_deref(),
        Some("We apologize for the delay.")
    );
    assert!(result.feedback.strengths.is_empty());
    assert!(result.feedback.improvements.is_empty());
    assert_eq!(transport.count(), 1);
}

// QU-AC-002/003/018: distinguish known failure stages using fixed safe codes.
#[tokio::test]
async fn malformed_result_json_and_inconsistent_fields_remain_single_attempt_failures() {
    let mut inconsistent = feedback();
    inconsistent["assessment"] = json!("reference_mismatch");
    let mut missing = feedback();
    missing
        .as_object_mut()
        .unwrap()
        .remove("unassessable_reason");
    let content_cases = [
        ("{not-json".to_owned(), ErrorCode::ResponseJsonInvalid),
        (
            format!("```json\n{}\n```", feedback()),
            ErrorCode::ResponseJsonInvalid,
        ),
        (inconsistent.to_string(), ErrorCode::ResponseFieldsInvalid),
        (missing.to_string(), ErrorCode::ResponseFieldsInvalid),
    ];
    for (content, expected) in content_cases {
        let mut bytes = event(
            json!({"choices":[{"index":0,"delta":{"content":content},"finish_reason":"stop"}]}),
        );
        bytes.extend_from_slice(b"data: [DONE]\n\n");
        let transport = ScriptTransport::new(Reply::Chunks(200, vec![Ok(bytes)]));
        let error = evaluate(snapshot(), transport.clone(), CancellationToken::new())
            .await
            .unwrap_err();
        assert_eq!(error.code(), expected);
        assert_eq!(error.disposition(), SendDisposition::MayHaveBeenSent);
        assert_eq!(transport.count(), 1);
        let wire = serde_json::to_value(&error).unwrap();
        assert_eq!(
            serde_json::from_value::<SafeError>(wire.clone()).unwrap(),
            error
        );
        assert_eq!(wire.as_object().unwrap().len(), 3);
        let text = wire.to_string();
        assert!(!text.contains(KEY));
        assert!(!text.contains(REFERENCE));
        assert!(!text.contains("{not-json"));
    }
}

#[test]
fn staged_safe_errors_roundtrip_and_reject_message_or_field_injection() {
    for code in [
        ErrorCode::ResponseInvalid,
        ErrorCode::ResponseJsonInvalid,
        ErrorCode::ResponseFieldsInvalid,
    ] {
        for sent in [SendDisposition::NotSent, SendDisposition::MayHaveBeenSent] {
            let error = SafeError::new(code, sent);
            let wire = serde_json::to_value(&error).unwrap();
            assert_eq!(
                serde_json::from_value::<SafeError>(wire.clone()).unwrap(),
                error
            );
            assert_eq!(wire.as_object().unwrap().len(), 3);
            let mut injected = wire.clone();
            injected["message"] = json!(KEY);
            assert!(serde_json::from_value::<SafeError>(injected).is_err());
            let mut injected = wire;
            injected["raw_response"] = json!(REFERENCE);
            assert!(serde_json::from_value::<SafeError>(injected).is_err());
        }
    }
}

#[tokio::test]
async fn utf8_crlf_sse_split_at_every_byte_keeps_content_separate_from_reasoning() {
    let mut bytes = event(
        json!({"model":"returned-model", "reasoning_effort":"low", "choices":[{"index":0,"delta":{"content":feedback().to_string(),"reasoning_content":"ignored reasoning"},"finish_reason":"stop"}]}),
    );
    bytes.extend(event(
        json!({"choices":[], "usage":{"prompt_tokens":21,"completion_tokens":8}}),
    ));
    bytes.extend_from_slice(b"data: [DONE]\r\n\r\n");
    let transport = ScriptTransport::new(Reply::Chunks(
        200,
        bytes.into_iter().map(|byte| Ok(vec![byte])).collect(),
    ));
    let result = evaluate(snapshot(), transport.clone(), CancellationToken::new())
        .await
        .unwrap();
    assert_eq!(result.feedback.summary, "語尾まで聞き取れる。");
    assert_eq!(result.actual_model.as_deref(), Some("returned-model"));
    assert_eq!(result.actual_effort.as_deref(), Some("low"));
    let usage = result.usage.unwrap();
    assert_eq!(usage.prompt_tokens, Some(21));
    assert_eq!(usage.completion_tokens, Some(8));
    assert_eq!(usage.total_tokens, None, "never infer missing totals");
    assert_eq!(transport.count(), 1);
}

#[tokio::test(start_paused = true)]
async fn stop_and_done_commit_success_even_if_transport_never_sends_eof() {
    let transport = ScriptTransport::new(Reply::ChunksThenPending(vec![sse(feedback())]));
    let task = tokio::spawn(evaluate(
        snapshot(),
        transport.clone(),
        CancellationToken::new(),
    ));
    transport.started.notified().await;
    tokio::task::yield_now().await;
    let result = task
        .await
        .unwrap()
        .expect("[DONE] is the logical stream terminator");
    assert_eq!(result.feedback.heard_text.as_deref(), Some(REFERENCE));
    assert_eq!(transport.count(), 1);
}

#[tokio::test]
async fn json_escaped_key_is_detected_after_feedback_and_metadata_decoding() {
    use qwen_audio::{
        ApiKey, AudioInput, Connection, EvaluationSnapshot, ReferenceText, TokyoHost,
    };
    let key = "synthetic-quote-\"-backslash-\\-secret";
    for metadata in [false, true] {
        let mut value = feedback();
        if !metadata {
            value["summary"] = json!(format!("prefix {key} suffix"));
        }
        let mut event_value = json!({"choices":[{"index":0,"delta":{"content":value.to_string()},"finish_reason":"stop"}]});
        if metadata {
            event_value["model"] = json!(key);
        }
        let mut bytes = event(event_value);
        bytes.extend_from_slice(b"data: [DONE]\n\n");
        let snapshot = EvaluationSnapshot::new(
            AudioInput::parse(support::wav(1, 8_000, 8)).unwrap(),
            ReferenceText::new(REFERENCE.to_owned()).unwrap(),
            Connection::new(
                TokyoHost::parse(BASE).unwrap(),
                ApiKey::new(key.to_owned()).expect("legal ASCII header key"),
            ),
        )
        .unwrap();
        let transport = ScriptTransport::new(Reply::Chunks(200, vec![Ok(bytes)]));
        let error = evaluate(snapshot, transport, CancellationToken::new())
            .await
            .unwrap_err();
        assert_eq!(error.code(), ErrorCode::ResponseInvalid);
        assert!(!serde_json::to_string(&error)
            .unwrap()
            .contains("synthetic-quote"));
    }
}

#[tokio::test]
async fn http_failure_classes_never_include_raw_body_and_never_retry() {
    for (status, expected) in [
        (401, ErrorCode::Authentication),
        (403, ErrorCode::Authentication),
        (429, ErrorCode::RateLimited),
        (302, ErrorCode::Provider),
        (400, ErrorCode::Provider),
        (500, ErrorCode::Provider),
    ] {
        let transport = ScriptTransport::new(Reply::Chunks(
            status,
            vec![Ok(format!("{KEY} private-raw-response").into_bytes())],
        ));
        let error = evaluate(snapshot(), transport.clone(), CancellationToken::new())
            .await
            .unwrap_err();
        assert_eq!(error.code(), expected, "HTTP {status}");
        assert_eq!(error.disposition(), SendDisposition::MayHaveBeenSent);
        assert!(!format!("{error} {error:?}").contains(KEY));
        assert!(!format!("{error} {error:?}").contains("private-raw-response"));
        assert_eq!(transport.count(), 1);
    }
}

#[tokio::test]
async fn network_tls_and_stream_disconnect_are_safe_single_attempt_failures() {
    for reply in [
        Reply::Fail(TransportFailure::Network),
        Reply::Fail(TransportFailure::Tls),
        Reply::Chunks(200, vec![Err(TransportFailure::Network)]),
    ] {
        let transport = ScriptTransport::new(reply);
        let error = evaluate(snapshot(), transport.clone(), CancellationToken::new())
            .await
            .unwrap_err();
        assert_eq!(error.code(), ErrorCode::Network);
        assert_eq!(error.disposition(), SendDisposition::MayHaveBeenSent);
        assert_eq!(transport.count(), 1);
    }
}

#[tokio::test]
async fn malformed_truncated_length_and_multiple_choice_sse_are_not_success() {
    let mut no_stop = event(
        json!({"choices":[{"index":0,"delta":{"content":feedback().to_string()},"finish_reason":null}]}),
    );
    no_stop.extend_from_slice(b"data: [DONE]\n\n");
    let mut length = event(
        json!({"choices":[{"index":0,"delta":{"content":feedback().to_string()},"finish_reason":"length"}]}),
    );
    length.extend_from_slice(b"data: [DONE]\n\n");
    let mut multi = event(
        json!({"choices":[{"index":0,"delta":{"content":feedback().to_string()},"finish_reason":"stop"},{"index":1,"delta":{},"finish_reason":"stop"}]}),
    );
    multi.extend_from_slice(b"data: [DONE]\n\n");
    let mut no_done = sse(feedback());
    no_done.truncate(no_done.len() - b"data: [DONE]\r\n\r\n".len());
    for bytes in [
        b"data: not-json\n\n".to_vec(),
        vec![0xff, b'\n'],
        no_stop,
        length,
        multi,
        no_done,
    ] {
        let transport = ScriptTransport::new(Reply::Chunks(200, vec![Ok(bytes)]));
        let error = evaluate(snapshot(), transport.clone(), CancellationToken::new())
            .await
            .unwrap_err();
        assert_eq!(error.code(), ErrorCode::ResponseInvalid);
        assert_eq!(transport.count(), 1);
    }
}

#[tokio::test]
async fn response_wire_and_assembled_content_limits_are_enforced() {
    let oversized_content = event(
        json!({"choices":[{"index":0,"delta":{"content":"a".repeat(64 * 1024 + 1)},"finish_reason":"stop"}]}),
    );
    for bytes in [vec![b'x'; 1024 * 1024 + 1], oversized_content] {
        let transport = ScriptTransport::new(Reply::Chunks(200, vec![Ok(bytes)]));
        let error = evaluate(snapshot(), transport.clone(), CancellationToken::new())
            .await
            .unwrap_err();
        assert_eq!(error.code(), ErrorCode::ResponseTooLarge);
        assert_eq!(transport.count(), 1);
    }
}

#[tokio::test]
async fn reflected_key_is_rejected_in_each_feedback_or_metadata_field() {
    let mut values = Vec::new();
    for field in ["heard_text", "summary"] {
        let mut value = feedback();
        value[field] = json!(KEY);
        values.push(sse(value));
    }
    for field in ["observation", "practice"] {
        let mut value = feedback();
        value["improvements"][0][field] = json!(KEY);
        values.push(sse(value));
    }
    let mut value = feedback();
    value["strengths"][0] = json!(KEY);
    values.push(sse(value));
    let mut value = feedback();
    value["assessment"] = json!("unassessable");
    value["heard_text"] = Value::Null;
    value["strengths"] = json!([]);
    value["improvements"] = json!([]);
    value["unassessable_reason"] = json!(KEY);
    values.push(sse(value));
    for field in ["model", "reasoning_effort"] {
        let mut bytes = event(
            json!({(field):KEY, "choices":[{"index":0,"delta":{"content":feedback().to_string()},"finish_reason":"stop"}]}),
        );
        bytes.extend_from_slice(b"data: [DONE]\n\n");
        values.push(bytes);
    }
    for bytes in values {
        let transport = ScriptTransport::new(Reply::Chunks(200, vec![Ok(bytes)]));
        let error = evaluate(snapshot(), transport, CancellationToken::new())
            .await
            .unwrap_err();
        assert_eq!(error.code(), ErrorCode::ResponseInvalid);
        assert!(!serde_json::to_string(&error).unwrap().contains(KEY));
    }
}

#[tokio::test]
async fn already_cancelled_input_never_polls_transport() {
    let cancel = CancellationToken::new();
    cancel.cancel();
    let transport = ScriptTransport::success();
    let error = evaluate(snapshot(), transport.clone(), cancel)
        .await
        .unwrap_err();
    assert_eq!(error.code(), ErrorCode::Cancelled);
    assert_eq!(error.disposition(), SendDisposition::NotSent);
    assert_eq!(transport.count(), 0);
}

#[tokio::test]
async fn cancellation_after_send_does_not_adopt_a_late_success() {
    let transport = ScriptTransport::new(Reply::BodyNever);
    let cancel = CancellationToken::new();
    let task = tokio::spawn(evaluate(snapshot(), transport.clone(), cancel.clone()));
    transport.started.notified().await;
    cancel.cancel();
    let error = task.await.unwrap().unwrap_err();
    assert_eq!(error.code(), ErrorCode::Cancelled);
    assert_eq!(error.disposition(), SendDisposition::MayHaveBeenSent);
    assert_eq!(transport.count(), 1);
}

#[tokio::test(start_paused = true)]
async fn one_total_deadline_covers_headers_and_body_without_retry() {
    for reply in [Reply::HeadersNever, Reply::BodyNever] {
        let transport = ScriptTransport::new(reply);
        let task = tokio::spawn(evaluate(
            snapshot(),
            transport.clone(),
            CancellationToken::new(),
        ));
        transport.started.notified().await;
        tokio::time::advance(std::time::Duration::from_secs(179)).await;
        assert!(!task.is_finished());
        tokio::time::advance(std::time::Duration::from_secs(2)).await;
        let error = task.await.unwrap().unwrap_err();
        assert_eq!(error.code(), ErrorCode::Timeout);
        assert_eq!(error.disposition(), SendDisposition::MayHaveBeenSent);
        assert_eq!(transport.count(), 1);
    }
}
