#![cfg(feature = "mcp")]

#[path = "support/provider.rs"]
mod fixture;
mod support;

use qwen_audio::{
    ErrorCode, EvaluationResult, SafeError, SendDisposition, SessionId, SessionManager,
    SessionState,
};

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

#[test]
fn one_active_session_rejects_busy_and_cannot_complete_before_send() {
    let mut manager = SessionManager::new();
    let view = manager.start().unwrap();
    assert!(matches!(view.state, SessionState::AwaitingUser));
    assert_eq!(manager.start().unwrap_err().code(), ErrorCode::Busy);
    assert_eq!(
        manager
            .complete(&view.session_id, result())
            .unwrap_err()
            .code(),
        ErrorCode::InvalidState
    );
    manager.mark_running(&view.session_id).unwrap();
    assert_eq!(manager.start().unwrap_err().code(), ErrorCode::Busy);
    assert_eq!(
        manager.mark_running(&view.session_id).unwrap_err().code(),
        ErrorCode::InvalidState
    );
}

#[test]
fn cancelled_first_remains_cancelled_after_late_complete_and_failure() {
    for running in [false, true] {
        let mut manager = SessionManager::new();
        let view = manager.start().unwrap();
        if running {
            manager.mark_running(&view.session_id).unwrap();
        }
        let cancelled = manager.cancel(&view.session_id).unwrap();
        let expected = if running {
            SendDisposition::MayHaveBeenSent
        } else {
            SendDisposition::NotSent
        };
        assert!(
            matches!(cancelled.state, SessionState::Cancelled { send_disposition } if send_disposition == expected)
        );
        assert_eq!(
            manager.complete(&view.session_id, result()).unwrap(),
            cancelled
        );
        assert_eq!(
            manager
                .fail(
                    &view.session_id,
                    SafeError::new(ErrorCode::Provider, expected)
                )
                .unwrap(),
            cancelled
        );
        assert_eq!(manager.cancel(&view.session_id).unwrap(), cancelled);
    }
}

#[test]
fn completion_first_is_stable_and_retrieval_does_not_consume_result() {
    let mut manager = SessionManager::new();
    let view = manager.start().unwrap();
    manager.mark_running(&view.session_id).unwrap();
    let done = manager.complete(&view.session_id, result()).unwrap();
    assert!(matches!(done.state, SessionState::Completed { .. }));
    assert_eq!(manager.cancel(&view.session_id).unwrap(), done);
    assert_eq!(manager.get(&view.session_id).unwrap(), done);
    assert_eq!(manager.get(&view.session_id).unwrap(), done);
    assert_eq!(
        manager.mark_running(&view.session_id).unwrap_err().code(),
        ErrorCode::InvalidState
    );
}

#[test]
fn failed_session_does_not_reopen_and_new_session_has_distinct_opaque_id() {
    let mut manager = SessionManager::new();
    let first = manager.start().unwrap();
    let failed = manager
        .fail(
            &first.session_id,
            SafeError::new(ErrorCode::GuiDisconnected, SendDisposition::NotSent),
        )
        .unwrap();
    assert_eq!(manager.cancel(&first.session_id).unwrap(), failed);
    let next = manager.start().unwrap();
    assert_ne!(first.session_id, next.session_id);
    assert_eq!(next.session_id.as_str().len(), 36);
    assert_eq!(next.session_id.as_str().as_bytes()[14], b'4');
}

#[test]
fn terminal_fifo_keeps_sixteen_without_read_promoting_oldest() {
    let mut manager = SessionManager::new();
    let mut ids = Vec::new();
    for _ in 0..16 {
        let view = manager.start().unwrap();
        manager.cancel(&view.session_id).unwrap();
        ids.push(view.session_id);
    }
    manager.get(&ids[0]).unwrap();
    let newest = manager.start().unwrap();
    manager.cancel(&newest.session_id).unwrap();
    assert_eq!(
        manager.get(&ids[0]).unwrap_err().code(),
        ErrorCode::SessionNotFound
    );
    for id in &ids[1..] {
        assert!(manager.get(id).is_ok());
    }
    assert!(manager.get(&newest.session_id).is_ok());
    assert_eq!(
        SessionManager::new()
            .get(&newest.session_id)
            .unwrap_err()
            .code(),
        ErrorCode::SessionNotFound
    );
}

#[test]
fn forged_or_invalid_session_ids_cannot_reach_existing_session() {
    for id in ["", "1", "00000000-0000-1000-8000-000000000000"] {
        assert!(SessionId::parse(id).is_err());
    }
    let valid_unknown = SessionId::parse("00000000-0000-4000-8000-000000000000").unwrap();
    assert_eq!(
        SessionManager::new()
            .get(&valid_unknown)
            .unwrap_err()
            .code(),
        ErrorCode::SessionNotFound
    );
}
