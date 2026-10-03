#![cfg(feature = "mcp")]

#[path = "support/provider.rs"]
mod fixture;
#[path = "support/mcp.rs"]
mod ipc_fixture;
mod support;

use ipc_fixture::{assert_tool_error, content, Harness, Launcher};
use qwen_audio::{ChildMessage, EvaluationResult};
use serde_json::{json, Value};
use std::sync::{atomic::Ordering, Arc};

fn result() -> EvaluationResult {
    EvaluationResult {
        feedback: serde_json::from_value(fixture::feedback()).unwrap(),
        requested_model: "qwen3.8-omni-flash".to_owned(),
        requested_effort: "medium".to_owned(),
        actual_model: None,
        actual_effort: None,
        usage: None,
    }
}
async fn start(harness: &mut Harness) -> Value {
    content(
        &harness
            .tool(
                "start_reading_session",
                json!({"reference_text":fixture::REFERENCE}),
            )
            .await,
    )
}
async fn get(harness: &mut Harness, id: &Value) -> Value {
    content(
        &harness
            .tool("get_reading_result", json!({"session_id":id}))
            .await,
    )
}

#[tokio::test]
async fn protocol_lists_only_three_tools_and_rejects_unknown_method() {
    let mut harness = Harness::new(Arc::new(Launcher::default())).await;
    let list = harness.rpc("tools/list", json!({})).await;
    let mut names: Vec<_> = list["result"]["tools"]
        .as_array()
        .unwrap()
        .iter()
        .map(|t| t["name"].as_str().unwrap())
        .collect();
    names.sort();
    assert_eq!(
        names,
        [
            "cancel_reading_session",
            "get_reading_result",
            "start_reading_session"
        ]
    );
    assert_eq!(
        harness.rpc("unknown-method", json!({})).await["error"]["code"],
        -32601
    );
    harness.raw(b"not-json\n").await;
    assert_eq!(harness.receive().await["error"]["code"], -32700);
    harness.finish().await;
}

#[tokio::test]
async fn mcp_arguments_cannot_smuggle_audio_key_host_path_or_send_consent() {
    let launcher = Arc::new(Launcher::default());
    let mut harness = Harness::new(launcher.clone()).await;
    for field in ["audio", "api_key", "host", "path", "approved"] {
        let mut args = json!({"reference_text":fixture::REFERENCE});
        args[field] = json!("sentinel");
        let response = harness.tool("start_reading_session", args).await;
        assert!(
            response.get("error").is_some() || response["result"]["isError"] == true,
            "unknown field {field} cannot start"
        );
    }
    for args in [
        json!({"reference_text":" \n"}),
        json!({"reference_text":42}),
    ] {
        let response = harness.tool("start_reading_session", args).await;
        assert!(response.get("error").is_some() || response["result"]["isError"] == true);
    }
    assert!(launcher.children.lock().unwrap().is_empty());
    harness.finish().await;
}

#[tokio::test]
async fn start_only_opens_gui_busy_get_are_passive_and_cancel_before_claim_cannot_grant() {
    let mut harness = Harness::new(Arc::new(Launcher::default())).await;
    let view = start(&mut harness).await;
    assert_eq!(view["status"], "awaiting_user");
    let id = &view["session_id"];
    let child = harness.child(0);
    assert_eq!(child.count_event("send_granted"), 0);
    assert_tool_error(
        &harness
            .tool(
                "start_reading_session",
                json!({"reference_text":fixture::REFERENCE}),
            )
            .await,
        "busy",
    );
    assert_eq!(get(&mut harness, id).await, view);
    assert_eq!(get(&mut harness, id).await, view);
    assert_eq!(harness.launcher.children.lock().unwrap().len(), 1);
    let cancelled = content(
        &harness
            .tool("cancel_reading_session", json!({"session_id":id}))
            .await,
    );
    assert_eq!(cancelled["status"], "cancelled");
    child.emit(ChildMessage::SendIntent {
        session_id: child.session(),
        reference_text: fixture::REFERENCE.to_owned(),
    });
    assert_eq!(get(&mut harness, id).await, cancelled);
    assert_eq!(child.count_event("send_granted"), 0);
    harness.finish().await;
    assert!(child.life.exited.load(Ordering::SeqCst));
}

#[tokio::test]
async fn completed_result_is_retrievable_without_resend_and_committed_to_gui() {
    let mut harness = Harness::new(Arc::new(Launcher::default())).await;
    let view = start(&mut harness).await;
    let id = &view["session_id"];
    let child = harness.child(0);
    child.emit(ChildMessage::SendIntent {
        session_id: child.session(),
        reference_text: fixture::REFERENCE.to_owned(),
    });
    for _ in 0..10 {
        if get(&mut harness, id).await["status"] == "running" {
            break;
        }
    }
    assert_eq!(get(&mut harness, id).await["status"], "running");
    assert_eq!(child.count_event("send_granted"), 1);
    child.emit(ChildMessage::Completed {
        session_id: child.session(),
        result: result(),
    });
    let mut completed = get(&mut harness, id).await;
    for _ in 0..10 {
        if completed["status"] == "completed" {
            break;
        }
        completed = get(&mut harness, id).await;
    }
    assert_eq!(completed["status"], "completed");
    assert_eq!(
        completed["result"]["feedback"]["heard_text"],
        fixture::REFERENCE
    );
    assert_eq!(get(&mut harness, id).await, completed);
    assert_eq!(
        content(
            &harness
                .tool("cancel_reading_session", json!({"session_id":id}))
                .await
        ),
        completed
    );
    let serialized = completed.to_string();
    for secret in [
        fixture::KEY,
        fixture::BASE,
        "data:;base64,",
        "synthetic.wav",
    ] {
        assert!(!serialized.contains(secret));
    }
    tokio::task::yield_now().await;
    assert!(child
        .parents
        .lock()
        .unwrap()
        .iter()
        .any(|v| v["event"] == "committed" && v["view"]["status"] == "completed"));
    assert_eq!(child.count_event("send_granted"), 1);
    harness.finish().await;
}

#[tokio::test]
async fn parent_cancel_first_overrides_late_success_in_mcp_and_gui_committed_view() {
    let mut harness = Harness::new(Arc::new(Launcher::default())).await;
    let view = start(&mut harness).await;
    let id = &view["session_id"];
    let child = harness.child(0);
    child.emit(ChildMessage::SendIntent {
        session_id: child.session(),
        reference_text: fixture::REFERENCE.to_owned(),
    });
    for _ in 0..10 {
        if get(&mut harness, id).await["status"] == "running" {
            break;
        }
    }
    let cancelled = content(
        &harness
            .tool("cancel_reading_session", json!({"session_id":id}))
            .await,
    );
    assert_eq!(cancelled["status"], "cancelled");
    child.emit(ChildMessage::Completed {
        session_id: child.session(),
        result: result(),
    });
    for _ in 0..3 {
        assert_eq!(get(&mut harness, id).await, cancelled);
    }
    tokio::task::yield_now().await;
    let messages = child.parents.lock().unwrap();
    assert!(messages
        .iter()
        .any(|v| v["event"] == "committed" && v["view"]["status"] == "cancelled"));
    assert!(!messages
        .iter()
        .any(|v| v["event"] == "committed" && v["view"]["status"] == "completed"));
    drop(messages);
    harness.finish().await;
}

#[tokio::test]
async fn next_start_reaps_old_terminal_child_before_opening_new_one() {
    let mut harness = Harness::new(Arc::new(Launcher::default())).await;
    let first = start(&mut harness).await;
    let child = harness.child(0);
    harness
        .tool(
            "cancel_reading_session",
            json!({"session_id":first["session_id"]}),
        )
        .await;
    let next = start(&mut harness).await;
    assert_ne!(first["session_id"], next["session_id"]);
    assert_eq!(next["status"], "awaiting_user");
    assert_eq!(child.count_event("close"), 1);
    assert!(child.life.exited.load(Ordering::SeqCst));
    assert!(child.life.waits.load(Ordering::SeqCst) > 0);
    let new_child = harness.child(1);
    harness.finish().await;
    assert!(new_child.life.exited.load(Ordering::SeqCst));
}

#[tokio::test]
async fn launch_failure_does_not_leave_busy_reservation() {
    let launcher = Arc::new(Launcher::default());
    launcher.fail_next.store(true, Ordering::SeqCst);
    let mut harness = Harness::new(launcher).await;
    assert_tool_error(
        &harness
            .tool(
                "start_reading_session",
                json!({"reference_text":fixture::REFERENCE}),
            )
            .await,
        "gui_launch",
    );
    assert_eq!(start(&mut harness).await["status"], "awaiting_user");
    harness.finish().await;
}

#[tokio::test(start_paused = true)]
async fn missing_ready_is_bounded_and_child_is_killed_reaped_and_slot_released() {
    let launcher = Arc::new(Launcher::default());
    launcher.no_ready.store(true, Ordering::SeqCst);
    let mut harness = Harness::new(launcher.clone()).await;
    assert_tool_error(
        &harness
            .tool(
                "start_reading_session",
                json!({"reference_text":fixture::REFERENCE}),
            )
            .await,
        "gui_launch",
    );
    let child = harness.child(0);
    assert!(child.life.exited.load(Ordering::SeqCst));
    assert!(child.life.waits.load(Ordering::SeqCst) > 0);
    launcher.no_ready.store(false, Ordering::SeqCst);
    assert_eq!(start(&mut harness).await["status"], "awaiting_user");
    harness.finish().await;
}

#[tokio::test(start_paused = true)]
async fn parent_eof_forces_stubborn_child_to_exit_and_waits_after_kill() {
    let launcher = Arc::new(Launcher::default());
    launcher.stubborn.store(true, Ordering::SeqCst);
    let mut harness = Harness::new(launcher).await;
    start(&mut harness).await;
    let child = harness.child(0);
    harness.finish().await;
    assert_eq!(child.life.kills.load(Ordering::SeqCst), 1);
    assert!(child.life.waits.load(Ordering::SeqCst) > 0);
    assert!(child.life.exited.load(Ordering::SeqCst));
}

#[tokio::test]
async fn arbitrary_valid_unknown_id_is_session_not_found_without_launch() {
    let mut harness = Harness::new(Arc::new(Launcher::default())).await;
    let unknown = "00000000-0000-4000-8000-000000000000";
    assert_tool_error(
        &harness
            .tool("get_reading_result", json!({"session_id":unknown}))
            .await,
        "session_not_found",
    );
    assert!(harness.launcher.children.lock().unwrap().is_empty());
    harness.finish().await;
}

#[tokio::test]
async fn seventeenth_terminal_evicts_only_oldest_and_get_does_not_refresh_retention() {
    let mut harness = Harness::new(Arc::new(Launcher::default())).await;
    let mut ids = Vec::new();
    for _ in 0..16 {
        let view = start(&mut harness).await;
        harness
            .tool(
                "cancel_reading_session",
                json!({"session_id":view["session_id"]}),
            )
            .await;
        ids.push(view["session_id"].clone());
    }
    assert_eq!(get(&mut harness, &ids[0]).await["status"], "cancelled");
    let newest = start(&mut harness).await;
    harness
        .tool(
            "cancel_reading_session",
            json!({"session_id":newest["session_id"]}),
        )
        .await;
    assert_tool_error(
        &harness
            .tool("get_reading_result", json!({"session_id":ids[0]}))
            .await,
        "session_not_found",
    );
    for id in &ids[1..] {
        assert_eq!(get(&mut harness, id).await["status"], "cancelled");
    }
    assert_eq!(
        get(&mut harness, &newest["session_id"]).await["status"],
        "cancelled"
    );
    let old_id = newest["session_id"].clone();
    harness.finish().await;
    let mut fresh = Harness::new(Arc::new(Launcher::default())).await;
    assert_tool_error(
        &fresh
            .tool("get_reading_result", json!({"session_id":old_id}))
            .await,
        "session_not_found",
    );
    fresh.finish().await;
}

#[tokio::test]
async fn child_terminal_candidate_is_revalidated_against_confirmed_reference() {
    let mut harness = Harness::new(Arc::new(Launcher::default())).await;
    let view = start(&mut harness).await;
    let child = harness.child(0);
    child.emit(ChildMessage::SendIntent {
        session_id: child.session(),
        reference_text: fixture::REFERENCE.to_owned(),
    });
    for _ in 0..10 {
        if get(&mut harness, &view["session_id"]).await["status"] == "running" {
            break;
        }
    }
    let mut candidate = result();
    candidate.feedback.improvements[0].reference_excerpt = "not in confirmed reference".to_owned();
    child.emit(ChildMessage::Completed {
        session_id: child.session(),
        result: candidate,
    });
    let mut observed = get(&mut harness, &view["session_id"]).await;
    for _ in 0..10 {
        if observed["status"] == "failed" {
            break;
        }
        observed = get(&mut harness, &view["session_id"]).await;
    }
    assert_eq!(observed["status"], "failed");
    assert!(observed.get("result").is_none());
    assert_eq!(get(&mut harness, &view["session_id"]).await, observed);
    harness.finish().await;
}

#[tokio::test]
async fn protocol_progress_metadata_is_allowed_for_start_get_and_cancel() {
    for token in [json!("progress-sentinel"), json!(42)] {
        let mut harness = Harness::new(Arc::new(Launcher::default())).await;
        let response = harness.rpc("tools/call",json!({"name":"start_reading_session","arguments":{"reference_text":fixture::REFERENCE},"_meta":{"progressToken":token}})).await;
        let view = content(&response);
        assert_eq!(view["status"], "awaiting_user");
        let id = &view["session_id"];
        let read = harness.rpc("tools/call",json!({"name":"get_reading_result","arguments":{"session_id":id},"_meta":{"progressToken":token}})).await;
        assert_eq!(content(&read), view);
        let cancel = harness.rpc("tools/call",json!({"name":"cancel_reading_session","arguments":{"session_id":id},"_meta":{"progressToken":token}})).await;
        assert_eq!(content(&cancel)["status"], "cancelled");
        assert_eq!(harness.child(0).count_event("send_granted"), 0);
        harness.finish().await;
    }
}

#[tokio::test]
async fn invalid_metadata_is_rejected_and_metadata_never_bypasses_tool_argument_validation() {
    let mut harness = Harness::new(Arc::new(Launcher::default())).await;
    for meta in [
        json!(null),
        json!("invalid"),
        json!([]),
        json!(42),
        json!({"progressToken":null}),
        json!({"progressToken":false}),
        json!({"progressToken":[]}),
        json!({"progressToken":{}}),
        json!({"progressToken":1.5}),
    ] {
        let response = harness.rpc("tools/call",json!({"name":"start_reading_session","arguments":{"reference_text":fixture::REFERENCE},"_meta":meta})).await;
        assert_eq!(response["error"]["code"], -32602, "invalid metadata {meta}");
    }
    for unknown in ["path", "api_key", "endpoint"] {
        let mut arguments = json!({"reference_text":fixture::REFERENCE});
        arguments[unknown] = json!("sentinel");
        let response = harness.rpc("tools/call",json!({"name":"start_reading_session","arguments":arguments,"_meta":{"progressToken":"allowed-token"}})).await;
        assert!(response.get("error").is_some() || response["result"]["isError"] == true);
    }
    assert!(harness.launcher.children.lock().unwrap().is_empty());
    harness.finish().await;
}

async fn claim(harness: &mut Harness, view: &Value, child: &ipc_fixture::ChildControl) {
    child.emit(ChildMessage::SendIntent {
        session_id: child.session(),
        reference_text: fixture::REFERENCE.to_owned(),
    });
    for _ in 0..10 {
        if get(harness, &view["session_id"]).await["status"] == "running" {
            break;
        }
        tokio::task::yield_now().await;
    }
    assert_eq!(get(harness, &view["session_id"]).await["status"], "running");
    assert_eq!(child.count_event("send_granted"), 1);
}

#[tokio::test(start_paused = true)]
async fn parent_timeout_starts_at_running_claim_and_commits_safe_failure_to_child() {
    let mut harness = Harness::new(Arc::new(Launcher::default())).await;
    let view = start(&mut harness).await;
    let child = harness.child(0);
    tokio::time::advance(std::time::Duration::from_secs(181)).await;
    assert_eq!(
        get(&mut harness, &view["session_id"]).await,
        view,
        "user approval has no running deadline"
    );
    claim(&mut harness, &view, &child).await;
    tokio::time::advance(std::time::Duration::from_secs(179)).await;
    assert_eq!(
        get(&mut harness, &view["session_id"]).await["status"],
        "running"
    );
    tokio::time::advance(std::time::Duration::from_secs(2)).await;
    let mut failed = get(&mut harness, &view["session_id"]).await;
    for _ in 0..10 {
        if failed["status"] == "failed" {
            break;
        }
        tokio::task::yield_now().await;
        failed = get(&mut harness, &view["session_id"]).await;
    }
    assert_eq!(failed["status"], "failed");
    assert_eq!(failed["error"]["code"], "timeout");
    assert_eq!(failed["error"]["send_disposition"], "may_have_been_sent");
    assert!(failed.get("result").is_none());
    tokio::task::yield_now().await;
    assert!(child.count_event("cancel") > 0);
    assert!(child
        .parents
        .lock()
        .unwrap()
        .iter()
        .any(|v| v["event"] == "committed" && v["view"] == failed));
    child.emit(ChildMessage::Completed {
        session_id: child.session(),
        result: result(),
    });
    assert_eq!(
        get(&mut harness, &view["session_id"]).await,
        failed,
        "late candidate cannot override timeout"
    );
    assert_eq!(child.count_event("send_granted"), 1);
    harness.finish().await;
}

#[tokio::test(start_paused = true)]
async fn parent_deadline_never_overwrites_completed_or_cancelled_terminal_state() {
    for complete in [false, true] {
        let mut harness = Harness::new(Arc::new(Launcher::default())).await;
        let view = start(&mut harness).await;
        let child = harness.child(0);
        claim(&mut harness, &view, &child).await;
        let terminal = if complete {
            child.emit(ChildMessage::Completed {
                session_id: child.session(),
                result: result(),
            });
            let mut current = get(&mut harness, &view["session_id"]).await;
            for _ in 0..10 {
                if current["status"] == "completed" {
                    break;
                }
                current = get(&mut harness, &view["session_id"]).await;
            }
            assert_eq!(current["status"], "completed");
            current
        } else {
            content(
                &harness
                    .tool(
                        "cancel_reading_session",
                        json!({"session_id":view["session_id"]}),
                    )
                    .await,
            )
        };
        assert_eq!(
            terminal["status"],
            if complete { "completed" } else { "cancelled" }
        );
        tokio::time::advance(std::time::Duration::from_secs(181)).await;
        assert_eq!(get(&mut harness, &view["session_id"]).await, terminal);
        harness.finish().await;
    }
}

#[tokio::test(start_paused = true)]
async fn exited_gui_is_detected_even_when_stdout_pipe_is_kept_open() {
    for running in [false, true] {
        let mut harness = Harness::new(Arc::new(Launcher::default())).await;
        let view = start(&mut harness).await;
        let child = harness.child(0);
        if running {
            claim(&mut harness, &view, &child).await;
        }
        child.exit_process_keep_stdout();
        let mut observed = get(&mut harness, &view["session_id"]).await;
        for _ in 0..10 {
            if observed["status"] == "failed" {
                break;
            }
            tokio::time::advance(std::time::Duration::from_millis(100)).await;
            observed = get(&mut harness, &view["session_id"]).await;
        }
        assert_eq!(
            observed["status"], "failed",
            "process death must not wait for stdout EOF"
        );
        assert_eq!(observed["error"]["code"], "gui_disconnected");
        assert_eq!(
            observed["error"]["send_disposition"],
            if running {
                "may_have_been_sent"
            } else {
                "not_sent"
            }
        );
        assert!(observed.get("result").is_none());
        harness.finish().await;
    }
}
