#[path = "fixtures/reuse/mod.rs"]
mod fixture;
use fixture::*;
use qwen_audio::*;
use tokio_util::sync::CancellationToken;

#[tokio::test]
async fn core_same_fixture_outcomes_and_metadata_absent_stays_unknown() {
    for (text, cannot) in [(ASSESSED, false), (UNASSESSABLE, true)] {
        let wire = Wire::new(Reply::Feedback(text));
        let result = evaluate(snapshot(), wire.clone(), CancellationToken::new())
            .await
            .unwrap();
        assert_eq!(result, expected(cannot));
        assert_eq!(wire.count(), 1);
        wire.assert_request(0, HOST, &audio());
    }
}
#[tokio::test]
async fn core_failure_classification_and_secret_response_are_safe() {
    for (reply, code) in [
        (Reply::Status(401), ErrorCode::Authentication),
        (Reply::Status(429), ErrorCode::RateLimited),
        (Reply::Status(500), ErrorCode::Provider),
        (Reply::Network, ErrorCode::Network),
        (
            Reply::Raw(b"data: invalid\n\n".to_vec()),
            ErrorCode::ResponseInvalid,
        ),
    ] {
        let wire = Wire::new(reply);
        let error = evaluate(snapshot(), wire.clone(), CancellationToken::new())
            .await
            .unwrap_err();
        assert_eq!(error.code(), code);
        assert_eq!(error.disposition(), SendDisposition::MayHaveBeenSent);
        assert!(!format!("{error:?} {error}").contains(KEY));
        assert_eq!(wire.count(), 1);
    }
}
#[tokio::test]
async fn core_secret_reflected_in_valid_result_is_rejected() {
    let value = ASSESSED.replace("語尾まで聞き取れる。", KEY);
    let wire = Wire::new(Reply::Raw(sse(&value)));
    let error = evaluate(snapshot(), wire, CancellationToken::new())
        .await
        .unwrap_err();
    assert_eq!(error.code(), ErrorCode::ResponseInvalid);
    assert!(!serde_json::to_string(&error).unwrap().contains(KEY));
}
#[cfg(feature = "gui")]
#[tokio::test]
async fn standalone_consumer_same_fixture_outcomes() {
    use qwen_audio::{gui::GuiController, SessionState};
    for (text, cannot) in [(ASSESSED, false), (UNASSESSABLE, true)] {
        let wire = Wire::new(Reply::Feedback(text));
        let mut gui =
            GuiController::new(REFERENCE.into(), Store::configured(), wire.clone()).unwrap();
        gui.select_audio(AudioInput::parse(audio()).unwrap(), "synthetic.wav".into())
            .unwrap();
        assert_eq!(wire.count(), 0);
        gui.send(&tokio::runtime::Handle::current()).unwrap();
        match gui.wait_for_terminal().await.unwrap().state {
            SessionState::Completed { result } => assert_eq!(result, expected(cannot)),
            other => panic!("expected completed fixture: {other:?}"),
        }
        wire.assert_request(0, HOST, &audio());
        assert_eq!(wire.count(), 1);
    }
}

#[cfg(feature = "gui")]
#[tokio::test]
async fn standalone_consumer_same_failure_classification() {
    use qwen_audio::{gui::GuiController, SessionState};
    for (reply, code) in [
        (Reply::Status(401), ErrorCode::Authentication),
        (Reply::Status(429), ErrorCode::RateLimited),
        (Reply::Status(500), ErrorCode::Provider),
        (Reply::Network, ErrorCode::Network),
        (
            Reply::Raw(b"data: invalid\n\n".to_vec()),
            ErrorCode::ResponseInvalid,
        ),
    ] {
        let wire = Wire::new(reply);
        let mut gui =
            GuiController::new(REFERENCE.into(), Store::configured(), wire.clone()).unwrap();
        gui.select_audio(AudioInput::parse(audio()).unwrap(), "synthetic.wav".into())
            .unwrap();
        gui.send(&tokio::runtime::Handle::current()).unwrap();
        match gui.wait_for_terminal().await.unwrap().state {
            SessionState::Failed { error } => {
                assert_eq!(error.code(), code);
                assert_eq!(error.disposition(), SendDisposition::MayHaveBeenSent);
                assert!(!serde_json::to_string(&error).unwrap().contains(KEY));
            }
            other => panic!("expected classified failure: {other:?}"),
        }
        assert_eq!(wire.count(), 1);
    }
}
