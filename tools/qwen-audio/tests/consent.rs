#![cfg(all(windows, feature = "gui"))]

#[path = "support/provider.rs"]
mod fixture;
mod support;

use fixture::{connection, Reply, ScriptTransport, BASE, KEY, REFERENCE};
use qwen_audio::{
    gui::{GuiController, GuiPhase},
    AudioInput, Connection, CredentialStore, ErrorCode, SafeError, SendDisposition, SessionState,
};
use std::sync::{Arc, Mutex};

struct Store {
    value: Mutex<Option<Connection>>,
    saved: Mutex<Vec<String>>,
    fail_read: bool,
    fail_write: bool,
}
impl Store {
    fn new(fail_read: bool, fail_write: bool) -> Arc<Self> {
        Arc::new(Self {
            value: Mutex::new(Some(connection())),
            saved: Mutex::new(Vec::new()),
            fail_read,
            fail_write,
        })
    }
}
impl CredentialStore for Store {
    fn load(&self) -> Result<Option<Connection>, SafeError> {
        if self.fail_read {
            return Err(SafeError::new(
                ErrorCode::CredentialRead,
                SendDisposition::NotSent,
            ));
        }
        Ok(self.value.lock().unwrap().clone())
    }
    fn save(&self, value: &Connection) -> Result<(), SafeError> {
        self.saved
            .lock()
            .unwrap()
            .push(value.host().as_str().to_owned());
        if self.fail_write {
            return Err(SafeError::new(
                ErrorCode::CredentialWrite,
                SendDisposition::NotSent,
            ));
        }
        *self.value.lock().unwrap() = Some(value.clone());
        Ok(())
    }
}
fn selected(controller: &mut GuiController) {
    controller
        .select_audio(
            AudioInput::parse(support::wav(1, 8_000, 8)).unwrap(),
            "synthetic.wav".to_owned(),
        )
        .unwrap();
}

#[tokio::test]
async fn selection_settings_and_poll_do_not_send_then_explicit_send_claims_once() {
    let transport = ScriptTransport::success();
    let store = Store::new(false, false);
    let mut controller =
        GuiController::new(REFERENCE.to_owned(), store.clone(), transport.clone()).unwrap();
    selected(&mut controller);
    controller.poll().unwrap();
    let view = controller.begin_settings().unwrap();
    assert!(view.credential_present);
    controller
        .set_settings_draft(BASE.to_owned(), None)
        .unwrap();
    controller.save_settings().unwrap();
    assert_eq!(store.saved.lock().unwrap().len(), 1);
    assert_eq!(transport.count(), 0);
    assert!(!controller.model().host_label.contains(KEY));
    let running = controller.send(&tokio::runtime::Handle::current()).unwrap();
    assert!(matches!(running.state, SessionState::Running));
    assert_eq!(
        controller
            .send(&tokio::runtime::Handle::current())
            .unwrap_err()
            .code(),
        ErrorCode::InvalidState
    );
    assert_eq!(
        controller
            .set_reference("changed".to_owned())
            .unwrap_err()
            .code(),
        ErrorCode::InvalidState
    );
    assert_eq!(
        controller
            .select_audio(
                AudioInput::parse(support::wav(1, 8_000, 1)).unwrap(),
                "different.wav".to_owned()
            )
            .unwrap_err()
            .code(),
        ErrorCode::InvalidState
    );
    assert_eq!(
        controller.begin_settings().err().unwrap().code(),
        ErrorCode::InvalidState
    );
    let terminal = controller.wait_for_terminal().await.unwrap();
    assert!(matches!(terminal.state, SessionState::Completed { .. }));
    assert_eq!(transport.count(), 1);
    assert_eq!(controller.poll().unwrap(), terminal);
    assert_eq!(
        controller
            .send(&tokio::runtime::Handle::current())
            .unwrap_err()
            .code(),
        ErrorCode::InvalidState
    );
    let next = controller.new_session().unwrap();
    assert_ne!(next.session_id, terminal.session_id);
    assert_eq!(
        transport.count(),
        1,
        "new practice needs fresh explicit send"
    );
}

#[tokio::test]
async fn invalid_reference_and_missing_audio_leave_waiting_without_send() {
    let transport = ScriptTransport::success();
    let mut controller =
        GuiController::new(String::new(), Store::new(false, false), transport.clone()).unwrap();
    selected(&mut controller);
    controller.set_reference(" \n\u{3000}".to_owned()).unwrap();
    assert_eq!(
        controller
            .send(&tokio::runtime::Handle::current())
            .unwrap_err()
            .code(),
        ErrorCode::InvalidReference
    );
    assert!(matches!(controller.model().phase, GuiPhase::AwaitingUser));
    assert_eq!(transport.count(), 0);
    let mut empty = GuiController::new(
        REFERENCE.to_owned(),
        Store::new(false, false),
        transport.clone(),
    )
    .unwrap();
    assert!(empty.send(&tokio::runtime::Handle::current()).is_err());
    assert_eq!(transport.count(), 0);
}

#[tokio::test]
async fn cancelling_settings_or_failed_atomic_save_preserves_old_connection() {
    for failed in [false, true] {
        let store = Store::new(false, failed);
        let transport = ScriptTransport::success();
        let mut controller =
            GuiController::new(REFERENCE.to_owned(), store.clone(), transport.clone()).unwrap();
        controller.begin_settings().unwrap();
        controller
            .set_settings_draft(
                "https://replacement.ap-northeast-1.maas.aliyuncs.com".to_owned(),
                Some("replacement-key".to_owned()),
            )
            .unwrap();
        if failed {
            assert_eq!(
                controller.save_settings().unwrap_err().code(),
                ErrorCode::CredentialWrite
            );
        } else {
            controller.cancel_settings();
        }
        assert_eq!(controller.model().host_label, BASE);
        assert_eq!(store.load().unwrap().unwrap().host().as_str(), BASE);
        selected(&mut controller);
        if failed {
            // QS-AC-014: a failed save leaves the editor open, so sending is blocked.
            assert_eq!(
                controller
                    .send(&tokio::runtime::Handle::current())
                    .unwrap_err()
                    .code(),
                ErrorCode::InvalidState
            );
            assert_eq!(transport.count(), 0);
            controller.cancel_settings();
        }
        controller.send(&tokio::runtime::Handle::current()).unwrap();
        controller.wait_for_terminal().await.unwrap();
        assert_eq!(
            transport.requests.lock().unwrap()[0].endpoint,
            format!("{BASE}/chat/completions")
        );
    }
}

#[tokio::test]
async fn credential_read_failure_keeps_repair_screen_and_no_send() {
    let transport = ScriptTransport::success();
    let store = Store::new(true, false);
    let mut controller =
        GuiController::new(REFERENCE.to_owned(), store, transport.clone()).unwrap();
    assert!(controller.model().notice.is_some());
    assert!(!controller.model().credential_present);
    assert!(controller.begin_settings().is_ok());
    selected(&mut controller);
    assert!(controller.send(&tokio::runtime::Handle::current()).is_err());
    assert_eq!(transport.count(), 0);
}

#[tokio::test]
async fn cancelled_before_send_is_not_sent_and_running_cancel_is_stable() {
    let transport = ScriptTransport::success();
    let mut controller = GuiController::new(
        REFERENCE.to_owned(),
        Store::new(false, false),
        transport.clone(),
    )
    .unwrap();
    selected(&mut controller);
    assert!(matches!(
        controller.cancel().unwrap().state,
        SessionState::Cancelled {
            send_disposition: SendDisposition::NotSent
        }
    ));
    assert_eq!(transport.count(), 0);
    let transport = ScriptTransport::new(Reply::BodyNever);
    let mut controller = GuiController::new(
        REFERENCE.to_owned(),
        Store::new(false, false),
        transport.clone(),
    )
    .unwrap();
    selected(&mut controller);
    controller.send(&tokio::runtime::Handle::current()).unwrap();
    transport.started.notified().await;
    let cancelled = controller.cancel().unwrap();
    assert!(matches!(
        cancelled.state,
        SessionState::Cancelled {
            send_disposition: SendDisposition::MayHaveBeenSent
        }
    ));
    assert_eq!(controller.wait_for_terminal().await.unwrap(), cancelled);
    assert_eq!(controller.poll().unwrap(), cancelled);
    assert_eq!(transport.count(), 1);
}

#[tokio::test]
async fn changed_source_after_selection_does_not_change_transmitted_snapshot() {
    use base64::Engine;
    let temp = support::TempData::new();
    let original = support::wav(1, 8_000, 8);
    let path = temp.file("synthetic.wav", &original);
    let transport = ScriptTransport::success();
    let mut controller = GuiController::new(
        REFERENCE.to_owned(),
        Store::new(false, false),
        transport.clone(),
    )
    .unwrap();
    controller
        .select_audio(
            qwen_audio::read_wav(&path).unwrap(),
            "synthetic.wav".to_owned(),
        )
        .unwrap();
    std::fs::write(&path, support::wav(2, 16_000, 9)).unwrap();
    std::fs::remove_file(&path).unwrap();
    controller.send(&tokio::runtime::Handle::current()).unwrap();
    controller.wait_for_terminal().await.unwrap();
    let requests = transport.requests.lock().unwrap();
    let body: serde_json::Value = serde_json::from_slice(&requests[0].body).unwrap();
    let user = body["messages"]
        .as_array()
        .unwrap()
        .iter()
        .find(|m| m["role"] == "user")
        .unwrap();
    let audio = user["content"]
        .as_array()
        .unwrap()
        .iter()
        .find(|v| v["type"] == "input_audio")
        .unwrap();
    let encoded = audio["input_audio"]["data"]
        .as_str()
        .unwrap()
        .strip_prefix("data:;base64,")
        .unwrap();
    assert_eq!(
        base64::engine::general_purpose::STANDARD
            .decode(encoded)
            .unwrap(),
        original
    );
}
