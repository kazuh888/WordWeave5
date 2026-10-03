#![cfg(windows)]
#[path = "../tools/qwen-audio/tests/fixtures/reuse/mod.rs"]
mod fixture;
use fixture::*;
use qwen_audio::*;
use std::sync::{atomic::Ordering, Arc};
use wordweave5::qwen_reading::{
    ReadingController, ReadingPhase, ReadingTarget, WORDWEAVE_QWEN_CREDENTIAL_TARGET,
};

fn controller(store: Arc<Store>, wire: Arc<Wire>) -> ReadingController {
    ReadingController::new(
        ReadingTarget::new(
            "synthetic-entry".into(),
            b"synthetic-version-1".to_vec(),
            REFERENCE.into(),
        )
        .unwrap(),
        store,
        wire,
    )
    .unwrap()
}
fn selected(wire: Arc<Wire>) -> ReadingController {
    let mut c = controller(Store::configured(), wire);
    c.set_audio("synthetic.wav".into(), audio()).unwrap();
    c
}
async fn settle(c: &mut ReadingController) {
    for _ in 0..100 {
        tokio::task::yield_now().await;
        c.poll();
        if c.view().phase != ReadingPhase::Running {
            return;
        }
    }
    panic!("immediate synthetic transport did not terminate");
}
fn unsent(error: SafeError, code: ErrorCode) {
    assert_eq!(error.code(), code);
    assert_eq!(error.disposition(), SendDisposition::NotSent);
}

#[tokio::test]
async fn validated_formats_keep_original_audio_until_explicit_confirmation() {
    use base64::Engine;
    for (bytes, format) in [
        (
            include_bytes!("../tools/qwen-audio/tests/fixtures/formats/tone.mp3").as_slice(),
            "mp3",
        ),
        (
            include_bytes!("../tools/qwen-audio/tests/fixtures/formats/tone.aac").as_slice(),
            "aac",
        ),
        (
            include_bytes!("../tools/qwen-audio/tests/fixtures/formats/tone.amr").as_slice(),
            "amr",
        ),
        (
            include_bytes!("../tools/qwen-audio/tests/fixtures/formats/tone.3gp").as_slice(),
            "3gp",
        ),
        (
            include_bytes!("../tools/qwen-audio/tests/fixtures/formats/tone.3gpp").as_slice(),
            "3gp",
        ),
    ] {
        let wire = Wire::new(Reply::Feedback(ASSESSED));
        let mut c = selected(wire.clone());
        let previous = c.prepare().unwrap();
        let input = AudioInput::decode(bytes.to_vec(), &tokio_util::sync::CancellationToken::new())
            .unwrap();
        c.set_validated_audio("synthetic selected audio".into(), input)
            .unwrap();
        assert_eq!(c.view().audio_format, Some(format));
        assert!(c
            .human_send(previous.id, &tokio::runtime::Handle::current())
            .is_err());
        assert_eq!(wire.count(), 0);
        let preview = c.prepare().unwrap();
        assert_eq!(preview.audio_format, format);
        c.human_send(preview.id, &tokio::runtime::Handle::current())
            .unwrap();
        settle(&mut c).await;
        let requests = wire.requests.lock().unwrap();
        assert_eq!(requests.len(), 1);
        let audio = &requests[0].1["messages"][1]["content"][1]["input_audio"];
        assert_eq!(audio["format"], format);
        let encoded = audio["data"].as_str().unwrap().split_once(',').unwrap().1;
        assert_eq!(
            base64::engine::general_purpose::STANDARD
                .decode(encoded)
                .unwrap(),
            bytes
        );
    }
}

#[test]
fn namespaces_never_read_legacy_record() {
    assert_eq!(
        WORDWEAVE_QWEN_CREDENTIAL_TARGET,
        "WordWeave5.QwenReading.Connection.v1"
    );
    let store = Store::new(None);
    let wire = Wire::new(Reply::Pending);
    let mut c = controller(store.clone(), wire.clone());
    assert!(!c.begin_settings().unwrap().credential_present);
    assert_eq!(store.loads.load(Ordering::SeqCst), 1);
    assert_eq!(store.saves.load(Ordering::SeqCst), 0);
    assert_eq!(wire.count(), 0);
}
#[test]
fn invalid_reference_boundaries_are_enforced() {
    for reference in [" \n\u{3000}".into(), "a".repeat(10_001)] {
        let error = ReadingTarget::new("id".into(), vec![1], reference)
            .err()
            .unwrap();
        unsent(error, ErrorCode::InvalidReference);
    }
    assert!(ReadingTarget::new("id".into(), vec![1], "😀".repeat(10_000)).is_ok());
}
#[tokio::test]
async fn prepare_calls_zero_human_send_calls_one() {
    let wire = Wire::new(Reply::Feedback(ASSESSED));
    let mut c = selected(wire.clone());
    let preview = c.prepare().unwrap();
    assert_eq!(preview.audio_label, "synthetic.wav");
    assert_eq!(preview.reference, REFERENCE);
    assert_eq!(preview.host, HOST);
    assert_eq!(preview.requested_model, "qwen3.8-omni-flash");
    assert_eq!(preview.requested_effort, "medium");
    assert_eq!(preview.audio_info.byte_len, audio().len());
    assert!(!preview.purpose.is_empty());
    assert!(!preview.recipient.is_empty());
    c.poll();
    assert_eq!(wire.count(), 0);
    c.human_send(preview.id, &tokio::runtime::Handle::current())
        .unwrap();
    settle(&mut c).await;
    assert_eq!(c.view().phase, ReadingPhase::Completed);
    assert_eq!(c.view().result, Some(expected(false)));
    wire.assert_request(0, HOST, &audio());
    assert_eq!(wire.count(), 1);
}
#[tokio::test]
async fn two_consumers_same_fixture_outcomes() {
    // Standalone GuiController/core tests use this exact fixture and independent expected().
    for (text, cannot) in [(ASSESSED, false), (UNASSESSABLE, true)] {
        let wire = Wire::new(Reply::Feedback(text));
        let mut c = selected(wire.clone());
        let p = c.prepare().unwrap();
        c.human_send(p.id, &tokio::runtime::Handle::current())
            .unwrap();
        settle(&mut c).await;
        assert_eq!(c.view().phase, ReadingPhase::Completed);
        assert_eq!(c.view().result, Some(expected(cannot)));
        assert!(c.view().error.is_none());
        wire.assert_request(0, HOST, &audio());
        assert_eq!(wire.count(), 1);
    }
}
#[tokio::test]
async fn double_send_calls_once_and_running_edits_are_rejected() {
    let wire = Wire::new(Reply::Pending);
    let mut c = selected(wire.clone());
    let p = c.prepare().unwrap();
    c.human_send(p.id, &tokio::runtime::Handle::current())
        .unwrap();
    assert!(c
        .human_send(p.id, &tokio::runtime::Handle::current())
        .is_err());
    assert!(c.set_audio("replacement".into(), audio()).is_err());
    assert!(c.clear_audio().is_err());
    assert!(c.begin_settings().is_err());
    wire.started.notified().await;
    assert_eq!(wire.count(), 1);
    c.cancel();
}
#[tokio::test]
async fn foreign_controller_prepared_id_calls_zero() {
    let wire = Wire::new(Reply::Pending);
    let mut a = selected(wire.clone());
    let mut b = selected(wire.clone());
    let first = a.prepare().unwrap();
    let second = b.prepare().unwrap();
    assert!(first.id != second.id);
    unsent(
        b.human_send(first.id, &tokio::runtime::Handle::current())
            .unwrap_err(),
        ErrorCode::InvalidState,
    );
    tokio::task::yield_now().await;
    assert_eq!(wire.count(), 0);
    assert_eq!(b.view().phase, ReadingPhase::Confirming);
}
#[tokio::test]
async fn cancel_before_send_calls_zero() {
    let wire = Wire::new(Reply::Pending);
    let mut c = selected(wire.clone());
    let p = c.prepare().unwrap();
    // Current-thread runtime has not polled the spawned worker at cancel time.
    c.human_send(p.id, &tokio::runtime::Handle::current())
        .unwrap();
    c.cancel();
    for _ in 0..5 {
        tokio::task::yield_now().await;
        c.poll();
    }
    assert_eq!(wire.count(), 0);
    assert_eq!(c.view().phase, ReadingPhase::Cancelled);
    assert_eq!(c.view().disposition, SendDisposition::NotSent);
}
#[tokio::test]
async fn confirming_cancel_and_close_call_zero() {
    for close in [false, true] {
        let wire = Wire::new(Reply::Pending);
        let mut c = selected(wire.clone());
        let p = c.prepare().unwrap();
        if close {
            c.close();
        } else {
            c.cancel();
        }
        assert!(c
            .human_send(p.id, &tokio::runtime::Handle::current())
            .is_err());
        c.poll();
        assert_eq!(wire.count(), 0);
        assert_eq!(c.view().disposition, SendDisposition::NotSent);
    }
}
#[tokio::test]
async fn cancel_after_transport_start_keeps_remote_uncertainty_and_drops_worker() {
    let wire = Wire::new(Reply::Pending);
    let mut c = selected(wire.clone());
    let p = c.prepare().unwrap();
    c.human_send(p.id, &tokio::runtime::Handle::current())
        .unwrap();
    wire.started.notified().await;
    c.cancel();
    for _ in 0..5 {
        tokio::task::yield_now().await;
        c.poll();
    }
    assert_eq!(c.view().phase, ReadingPhase::Cancelled);
    assert_eq!(c.view().disposition, SendDisposition::MayHaveBeenSent);
    assert!(c.view().result.is_none());
    assert_eq!(wire.count(), 1);
    assert_eq!(wire.dropped.load(Ordering::SeqCst), 1);
}
#[tokio::test]
async fn cancel_wins_over_ready_completion() {
    let wire = Wire::new(Reply::Feedback(ASSESSED));
    let mut c = selected(wire.clone());
    let p = c.prepare().unwrap();
    c.human_send(p.id, &tokio::runtime::Handle::current())
        .unwrap();
    // Yield lets immediate SSE finish but intentionally do not poll controller completion.
    for _ in 0..5 {
        tokio::task::yield_now().await;
    }
    assert_eq!(wire.dropped.load(Ordering::SeqCst), 1);
    c.cancel();
    c.poll();
    assert_eq!(c.view().phase, ReadingPhase::Cancelled);
    assert!(c.view().result.is_none());
    assert_eq!(c.view().disposition, SendDisposition::MayHaveBeenSent);
    assert_eq!(wire.count(), 1);
}
#[tokio::test]
async fn closed_dialog_drops_late_result_and_cannot_revive() {
    let wire = Wire::new(Reply::Feedback(ASSESSED));
    let mut c = selected(wire.clone());
    let p = c.prepare().unwrap();
    c.human_send(p.id, &tokio::runtime::Handle::current())
        .unwrap();
    for _ in 0..5 {
        tokio::task::yield_now().await;
    }
    c.close();
    c.cancel();
    c.poll();
    assert_eq!(c.view().phase, ReadingPhase::Closed);
    assert!(c.view().result.is_none());
    assert!(c.view().audio_info.is_none());
    assert!(c.view().preview.is_none());
    assert!(c.restart().is_err());
    assert!(c.set_audio("x".into(), audio()).is_err());
    assert!(c.begin_settings().is_err());
    assert!(c.prepare().is_err());
    assert!(c
        .human_send(p.id, &tokio::runtime::Handle::current())
        .is_err());
    assert_eq!(wire.count(), 1);
}
#[tokio::test]
async fn dropping_controller_aborts_active_worker_without_retry() {
    let wire = Wire::new(Reply::Pending);
    let mut c = selected(wire.clone());
    let p = c.prepare().unwrap();
    c.human_send(p.id, &tokio::runtime::Handle::current())
        .unwrap();
    wire.started.notified().await;
    drop(c);
    for _ in 0..5 {
        tokio::task::yield_now().await;
    }
    assert_eq!(wire.dropped.load(Ordering::SeqCst), 1);
    assert_eq!(wire.count(), 1);
}
#[tokio::test]
async fn invalidated_cancel_cannot_restart_or_send() {
    let wire = Wire::new(Reply::Pending);
    let mut c = selected(wire.clone());
    let p = c.prepare().unwrap();
    c.invalidate_target();
    c.cancel();
    c.cancel_settings();
    c.poll();
    assert_eq!(c.view().phase, ReadingPhase::Invalidated);
    unsent(c.restart().unwrap_err(), ErrorCode::InvalidState);
    assert!(c.set_audio("x".into(), audio()).is_err());
    assert!(c.begin_settings().is_err());
    assert!(c.prepare().is_err());
    unsent(
        c.human_send(p.id, &tokio::runtime::Handle::current())
            .unwrap_err(),
        ErrorCode::InvalidState,
    );
    assert_eq!(wire.count(), 0);
}
#[tokio::test]
async fn target_revision_invalidates_pending_and_late_result() {
    let wire = Wire::new(Reply::Feedback(ASSESSED));
    let mut c = selected(wire.clone());
    let p = c.prepare().unwrap();
    c.human_send(p.id, &tokio::runtime::Handle::current())
        .unwrap();
    for _ in 0..5 {
        tokio::task::yield_now().await;
    }
    c.invalidate_target();
    c.poll();
    assert_eq!(c.view().phase, ReadingPhase::Invalidated);
    assert!(c.view().result.is_none());
    assert!(c.restart().is_err());
    assert_eq!(wire.count(), 1);
}
#[tokio::test]
async fn restart_waits_for_reap_and_requires_new_confirmation() {
    let wire = Wire::new(Reply::Pending);
    let mut c = selected(wire.clone());
    let p = c.prepare().unwrap();
    c.human_send(p.id, &tokio::runtime::Handle::current())
        .unwrap();
    wire.started.notified().await;
    c.cancel();
    unsent(c.restart().unwrap_err(), ErrorCode::Busy);
    assert_eq!(c.view().phase, ReadingPhase::Cancelled);
    for _ in 0..5 {
        tokio::task::yield_now().await;
        c.poll();
    }
    c.restart().unwrap();
    assert_eq!(c.view().phase, ReadingPhase::Input);
    assert!(c.view().result.is_none());
    assert!(c.view().preview.is_none());
    assert!(c
        .human_send(p.id, &tokio::runtime::Handle::current())
        .is_err());
    assert_eq!(wire.count(), 1);
    let next = c.prepare().unwrap();
    assert!(next.id != p.id);
    assert_eq!(wire.count(), 1);
}
#[tokio::test(start_paused = true)]
async fn timeout_never_retries_and_preserves_uncertainty() {
    let wire = Wire::new(Reply::Pending);
    let mut c = selected(wire.clone());
    let p = c.prepare().unwrap();
    c.human_send(p.id, &tokio::runtime::Handle::current())
        .unwrap();
    wire.started.notified().await;
    tokio::time::advance(std::time::Duration::from_secs(179)).await;
    c.poll();
    assert_eq!(c.view().phase, ReadingPhase::Running);
    tokio::time::advance(std::time::Duration::from_secs(1)).await;
    settle(&mut c).await;
    assert_eq!(c.view().phase, ReadingPhase::Failed);
    let e = c.view().error.unwrap();
    assert_eq!(e.code(), ErrorCode::Timeout);
    assert_eq!(e.disposition(), SendDisposition::MayHaveBeenSent);
    tokio::time::advance(std::time::Duration::from_secs(600)).await;
    c.poll();
    assert_eq!(wire.count(), 1);
    assert!(c.view().result.is_none());
}
#[tokio::test]
async fn failure_never_retries_and_malformed_or_secret_response_never_displayed() {
    for (reply, code) in [
        (Reply::Status(401), ErrorCode::Authentication),
        (Reply::Status(429), ErrorCode::RateLimited),
        (Reply::Status(500), ErrorCode::Provider),
        (Reply::Network, ErrorCode::Network),
        (
            Reply::Raw(sse(&ASSESSED.replace("語尾まで聞き取れる。", KEY))),
            ErrorCode::ResponseInvalid,
        ),
        (
            Reply::Raw(b"data: {}\n\n".to_vec()),
            ErrorCode::ResponseInvalid,
        ),
    ] {
        let wire = Wire::new(reply);
        let mut c = selected(wire.clone());
        let p = c.prepare().unwrap();
        c.human_send(p.id, &tokio::runtime::Handle::current())
            .unwrap();
        settle(&mut c).await;
        assert_eq!(c.view().phase, ReadingPhase::Failed);
        let e = c.view().error.unwrap();
        assert_eq!(e.code(), code);
        assert_eq!(e.disposition(), SendDisposition::MayHaveBeenSent);
        assert!(!serde_json::to_string(&e).unwrap().contains(KEY));
        assert!(c.view().result.is_none());
        for _ in 0..5 {
            c.poll();
            tokio::task::yield_now().await;
        }
        assert_eq!(wire.count(), 1);
    }
}
#[tokio::test]
async fn connection_change_invalidates_preparation_and_does_not_reload_at_send() {
    let store = Store::configured();
    let wire = Wire::new(Reply::Feedback(ASSESSED));
    let mut c = controller(store.clone(), wire.clone());
    c.set_audio("x".into(), audio()).unwrap();
    let p = c.prepare().unwrap();
    c.begin_settings().unwrap();
    // QS-AC-008/014: changing destination requires a key explicitly entered for it.
    c.set_settings_draft(OTHER_HOST.into(), Some(KEY.into()))
        .unwrap();
    c.save_settings().unwrap();
    assert_eq!(wire.count(), 0);
    assert!(c.view().preview.is_none());
    assert!(c
        .human_send(p.id, &tokio::runtime::Handle::current())
        .is_err());
    let next = c.prepare().unwrap();
    assert_eq!(next.host, OTHER_HOST);
    *store.value.lock().unwrap() = None;
    c.human_send(next.id, &tokio::runtime::Handle::current())
        .unwrap();
    settle(&mut c).await;
    assert_eq!(store.loads.load(Ordering::SeqCst), 1);
    wire.assert_request(0, OTHER_HOST, &audio());
    assert_eq!(c.view().result, Some(expected(false)));
}

// QS-AC-008/013/014: failed blank-key replacement must not alter the confirmed snapshot.
#[tokio::test]
async fn changed_host_without_new_key_is_rejected_and_old_confirmation_is_preserved() {
    let store = Store::configured();
    let wire = Wire::new(Reply::Feedback(ASSESSED));
    let mut c = controller(store.clone(), wire.clone());
    c.set_audio("synthetic.wav".into(), audio()).unwrap();
    let prepared = c.prepare().unwrap();
    c.begin_settings().unwrap();
    c.set_settings_draft(OTHER_HOST.into(), None).unwrap();
    let error = c
        .save_settings()
        .expect_err("different host needs a new key");
    assert_eq!(error.disposition(), SendDisposition::NotSent);
    assert_eq!(store.saves.load(Ordering::SeqCst), 0);
    assert_eq!(
        store
            .value
            .lock()
            .unwrap()
            .as_ref()
            .unwrap()
            .host()
            .as_str(),
        HOST
    );
    let kept = c
        .view()
        .preview
        .expect("failed save must keep old confirmation");
    assert_eq!(kept.host, HOST);
    assert!(kept.id == prepared.id);
    assert_eq!(wire.count(), 0);
    c.cancel_settings();
    c.human_send(prepared.id, &tokio::runtime::Handle::current())
        .unwrap();
    settle(&mut c).await;
    wire.assert_request(0, HOST, &audio());
    assert_eq!(wire.count(), 1);
    assert_eq!(c.view().result, Some(expected(false)));
}
#[test]
fn failed_save_keeps_connection_and_calls_zero() {
    for fail in [false, true] {
        let store = Store::configured();
        store.fail_save.store(fail, Ordering::SeqCst);
        let wire = Wire::new(Reply::Pending);
        let mut c = controller(store.clone(), wire.clone());
        c.set_audio("x".into(), audio()).unwrap();
        let p = c.prepare().unwrap();
        c.begin_settings().unwrap();
        c.set_settings_draft(OTHER_HOST.into(), Some("synthetic-new-key".into()))
            .unwrap();
        if fail {
            unsent(c.save_settings().unwrap_err(), ErrorCode::CredentialWrite);
        } else {
            c.cancel_settings();
        }
        assert_eq!(
            store
                .value
                .lock()
                .unwrap()
                .as_ref()
                .unwrap()
                .host()
                .as_str(),
            HOST
        );
        let v = c.view();
        assert_eq!(v.phase, ReadingPhase::Confirming);
        let kept = v.preview.unwrap();
        assert_eq!(kept.host, HOST);
        assert!(kept.id == p.id);
        assert_eq!(wire.count(), 0);
    }
}
#[test]
fn invalid_host_and_explicit_empty_key_call_zero_and_preserve_old_settings() {
    for (host, key, code) in [
        ("https://example.com", None, ErrorCode::InvalidHost),
        (HOST, Some(String::new()), ErrorCode::InvalidKey),
    ] {
        let wire = Wire::new(Reply::Pending);
        let store = Store::configured();
        let mut c = controller(store.clone(), wire.clone());
        c.begin_settings().unwrap();
        c.set_settings_draft(host.into(), key).unwrap();
        unsent(c.save_settings().unwrap_err(), code);
        assert_eq!(store.saves.load(Ordering::SeqCst), 0);
        assert_eq!(
            store
                .value
                .lock()
                .unwrap()
                .as_ref()
                .unwrap()
                .host()
                .as_str(),
            HOST
        );
        assert_eq!(wire.count(), 0);
    }
}
#[test]
fn credential_read_failure_is_visible_and_settings_repair_remains_available() {
    let store = Store::new(None);
    store.fail_load.store(true, Ordering::SeqCst);
    let wire = Wire::new(Reply::Pending);
    let mut c = controller(store.clone(), wire.clone());
    unsent(c.view().error.unwrap(), ErrorCode::CredentialRead);
    assert!(!c.begin_settings().unwrap().credential_present);
    c.set_settings_draft(HOST.into(), Some(KEY.into())).unwrap();
    c.save_settings().unwrap();
    c.set_audio("x".into(), audio()).unwrap();
    assert!(c.prepare().is_ok());
    assert_eq!(wire.count(), 0);
}
#[tokio::test]
async fn invalid_inputs_call_zero_and_bad_audio_cannot_reuse_previous_input() {
    let wire = Wire::new(Reply::Pending);
    let mut c = selected(wire.clone());
    let p = c.prepare().unwrap();
    for (bad, code) in [
        (vec![], ErrorCode::AudioEmpty),
        (b"not wave".to_vec(), ErrorCode::AudioCorrupt),
        (wav(1, 192_000, 8), ErrorCode::AudioUnsupported),
        (wav(3, 8_000, 8), ErrorCode::AudioUnsupported),
        (wav(1, 8_000, 480_001), ErrorCode::AudioTooLong),
        (vec![0; MAX_AUDIO_BYTES + 1], ErrorCode::AudioTooLarge),
    ] {
        unsent(c.set_audio("invalid.wav".into(), bad).unwrap_err(), code);
        assert!(c.prepare().is_err());
        assert!(c.view().preview.is_none());
        assert!(c.view().audio_info.is_none());
        assert!(c
            .human_send(p.id, &tokio::runtime::Handle::current())
            .is_err());
        assert_eq!(wire.count(), 0);
    }
}
#[test]
fn audio_sixty_second_boundary_and_missing_credentials_are_local() {
    let wire = Wire::new(Reply::Pending);
    let mut c = selected(wire.clone());
    c.set_audio("60seconds.wav".into(), wav(1, 8_000, 480_000))
        .unwrap();
    assert_eq!(c.prepare().unwrap().audio_info.duration_seconds, 60.0);
    let mut unset = controller(Store::new(None), wire.clone());
    unset.set_audio("x".into(), audio()).unwrap();
    assert!(unset.prepare().is_err());
    assert_eq!(wire.count(), 0);
}
#[tokio::test]
async fn snapshot_survives_source_rewrite_or_delete() {
    let dir = std::env::temp_dir().join(format!(
        "ww-qwen-snapshot-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir(&dir).unwrap();
    let path = dir.join("synthetic.wav");
    for delete in [false, true] {
        std::fs::write(&path, audio()).unwrap();
        let wire = Wire::new(Reply::Feedback(ASSESSED));
        let mut c = controller(Store::configured(), wire.clone());
        c.set_audio(path.display().to_string(), std::fs::read(&path).unwrap())
            .unwrap();
        let p = c.prepare().unwrap();
        if delete {
            std::fs::remove_file(&path).unwrap();
        } else {
            std::fs::write(&path, b"replacement synthetic bytes").unwrap();
        }
        c.human_send(p.id, &tokio::runtime::Handle::current())
            .unwrap();
        settle(&mut c).await;
        wire.assert_request(0, HOST, &audio());
        if !delete {
            assert_eq!(
                std::fs::read(&path).unwrap(),
                b"replacement synthetic bytes"
            );
            std::fs::remove_file(&path).unwrap();
        }
    }
    std::fs::remove_dir(&dir).unwrap();
}
