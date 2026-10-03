//! FEEDBACK-UI-001 U5: independent evidence diagnostics acceptance tests.
//! All conversations and entries below are synthetic and remain in memory.

use serde_json::json;
use wordweave5::{
    chat::Conversation,
    execution::Execution,
    material::{DiagnosticStage, EvidenceCause, MaterialFailure, Mode, Request},
    model::{self, Entry},
};

fn request_with_answer(answer: &str) -> (Request, Entry) {
    let baseline = model::parse_deck(model::BUILTIN_DECK).unwrap().remove(0);
    let mut conversation = Conversation::new();
    conversation
        .complete("合成の質問 🦊".into(), answer.into(), Execution::default())
        .unwrap();
    conversation.exchanges[0].for_material = true;
    let request = Request::new(
        &conversation,
        &baseline.base,
        Mode::Correct,
        Some(baseline.clone()),
    )
    .unwrap();
    let mut candidate = baseline;
    candidate.usage = "合成の説明へ訂正".into();
    (request, candidate)
}

fn response(entry: &Entry, path: &str, role: &str, quote: &str) -> String {
    json!({"entry":entry,"reasons":[{"path":path,"reason":"合成の変更理由",
        "quotes":[{"exchange_index":0,"role":role,"quote":quote}]}]})
    .to_string()
}

#[test]
fn n06_legacy_generation_diagnostic_remains_a_240_character_preview() {
    let answer = format!("{}固定元発言の終端 🦊", "あ".repeat(250));
    let (request, candidate) = request_with_answer(&answer);
    let error = request
        .build_response(&response(&candidate, "/usage", "assistant", "別の引用"))
        .unwrap_err();
    assert!(error.contains("…（省略）"));
    assert!(!error.contains("固定元発言の終端 🦊"));
}

fn diagnostic(failure: &MaterialFailure) -> &wordweave5::material::MaterialDiagnostic {
    failure
        .diagnostic()
        .expect("根拠検査失敗は型付き診断である")
}

#[test]
fn n02_n04_seventh_reason_uses_fourth_fixed_answer_and_keeps_both_full_texts() {
    let baseline = model::parse_deck(model::BUILTIN_DECK).unwrap().remove(0);
    let mut conversation = Conversation::new();
    for index in 0..4 {
        let answer = if index == 3 {
            format!("{}固定回答の終端 🦊", "答".repeat(320))
        } else {
            format!("合成回答 {index}")
        };
        conversation
            .complete(format!("合成質問 {index}"), answer, Execution::default())
            .unwrap();
    }
    conversation.exchanges[0].for_material = true;
    conversation.exchanges[3].for_material = true;
    let fixed_answer = conversation.exchanges[3].answer.clone();
    let request = Request::new(
        &conversation,
        &baseline.base,
        Mode::Correct,
        Some(baseline.clone()),
    )
    .unwrap();
    conversation.exchanges[3].answer = "後から編集した別の回答".into();
    let mut candidate = baseline;
    candidate.usage = "合成の説明へ訂正".into();
    let mut reasons: Vec<_> = (0..6)
        .map(|_| {
            json!({"path":"/usage","reason":"合成理由",
        "quotes":[{"exchange_index":0,"role":"assistant","quote":"合成回答 0"}]})
        })
        .collect();
    let wrong_quote = format!("{}引用の終端 🐺", "答".repeat(320));
    reasons.push(json!({"path":"/usage","reason":"合成理由",
        "quotes":[{"exchange_index":3,"role":"assistant","quote":wrong_quote}]}));
    reasons.push(json!({"path":"/meaning","reason":"後続理由",
        "quotes":[{"exchange_index":0,"role":"assistant","quote":"後続の誤引用"}]}));
    let text = json!({"entry":candidate,"reasons":reasons}).to_string();

    let failure = request.build_response_detailed(&text).unwrap_err();
    let detail = diagnostic(&failure);
    let evidence = detail.evidence.as_ref().unwrap();
    assert_eq!(detail.stage, DiagnosticStage::Generation);
    assert_eq!(detail.cause, EvidenceCause::QuoteMismatch);
    assert_eq!(detail.reason_number, Some(7));
    assert_eq!(detail.path.as_deref(), Some("/usage"));
    assert_eq!(evidence.exchange_index, 3);
    assert_eq!(evidence.role, "assistant");
    assert_eq!(evidence.quote, wrong_quote);
    assert_eq!(evidence.original.as_deref(), Some(fixed_answer.as_str()));
    assert!(!evidence.original.as_ref().unwrap().contains("後から編集"));
    assert!(!failure.legacy_message().contains("引用の終端 🐺"));
    assert_eq!(
        failure.legacy_message(),
        request.build_response(&text).unwrap_err()
    );
}

#[test]
fn n03_raw_evidence_keeps_literal_backslash_n_real_newline_and_unicode() {
    let original = " 前置き\\nそのまま\r\n**強調**\n- 箇条書き\t🦊 後置き ";
    let (request, candidate) = request_with_answer(original);
    let quote = " 前置き\\nそのまま\r\n**強調**\n- 箇条書き\t🐺 後置き ";
    let failure = request
        .build_response_detailed(&response(&candidate, "/usage", "assistant", quote))
        .unwrap_err();
    let evidence = diagnostic(&failure).evidence.as_ref().unwrap();
    assert_eq!(evidence.quote, quote);
    assert_eq!(evidence.original.as_deref(), Some(original));
    assert!(evidence.quote.contains("\\n"));
    assert!(evidence.quote.contains("\r\n"));
}

#[test]
fn n07_markdown_display_must_not_change_strict_quote_matching() {
    let (request, candidate) = request_with_answer("Use **improve clarity** here.");
    assert!(request
        .build_response_detailed(&response(
            &candidate,
            "/usage",
            "assistant",
            "improve clarity"
        ))
        .is_ok());
    let (request, candidate) = request_with_answer("Use improve **clarity** here.");
    let failure = request
        .build_response_detailed(&response(
            &candidate,
            "/usage",
            "assistant",
            "improve clarity",
        ))
        .unwrap_err();
    assert_eq!(diagnostic(&failure).cause, EvidenceCause::QuoteMismatch);
}

#[test]
fn n02_n07_invalid_path_and_missing_item_win_before_empty_reason_or_quote() {
    let (request, candidate) = request_with_answer("固定回答");
    for (path, expected) in [
        ("/id", EvidenceCause::InvalidPath),
        ("/examples/99/note", EvidenceCause::MissingItem),
    ] {
        let text = json!({"entry":candidate,"reasons":[{"path":path,"reason":" \n",
            "quotes":[]}]})
        .to_string();
        let failure = request.build_response_detailed(&text).unwrap_err();
        let detail = diagnostic(&failure);
        assert_eq!(detail.cause, expected, "{path}");
        assert_eq!(detail.reason_number, Some(1));
        assert_eq!(detail.path.as_deref(), Some(path));
        assert!(detail.target_label.is_none());
        assert!(detail.evidence.is_none());
        assert_eq!(
            failure.legacy_message(),
            request.build_response(&text).unwrap_err()
        );
    }
}

#[test]
fn n02_missing_turn_and_invalid_role_never_borrow_another_utterance() {
    let (request, candidate) = request_with_answer("固定された回答 🦊");
    for (index, role, cause) in [
        (1, "assistant", EvidenceCause::MissingSnapshot),
        (0, "system", EvidenceCause::InvalidRole),
    ] {
        let text = json!({"entry":candidate,"reasons":[{"path":"/usage","reason":"合成理由",
            "quotes":[{"exchange_index":index,"role":role,"quote":"架空の引用"}]}]})
        .to_string();
        let failure = request.build_response_detailed(&text).unwrap_err();
        let detail = diagnostic(&failure);
        assert_eq!(detail.cause, cause);
        let evidence = detail.evidence.as_ref().unwrap();
        assert_eq!(evidence.exchange_index, index);
        assert_eq!(evidence.role, role);
        assert!(evidence.original.is_none());
        assert!(!failure.legacy_message().contains("固定された回答 🦊"));
    }
}

#[test]
fn n04_quote_limit_keeps_full_rejected_quote_but_accepts_exactly_2000_scalars() {
    let original = format!("{}終端 🦊", "界".repeat(2000));
    let (request, candidate) = request_with_answer(&original);
    let exactly_2000 = "界".repeat(2000);
    assert!(request
        .build_response_detailed(&response(&candidate, "/usage", "assistant", &exactly_2000))
        .is_ok());
    let too_long = format!("{}🐺", exactly_2000);
    let failure = request
        .build_response_detailed(&response(&candidate, "/usage", "assistant", &too_long))
        .unwrap_err();
    let detail = diagnostic(&failure);
    assert_eq!(detail.cause, EvidenceCause::LongQuote);
    assert_eq!(detail.evidence.as_ref().unwrap().quote, too_long);
    assert_eq!(
        detail.evidence.as_ref().unwrap().original.as_deref(),
        Some(original.as_str())
    );
}

#[test]
fn n01_n07_saved_draft_revalidation_preserves_serialized_proposal_and_legacy_error() {
    let (request, candidate) = request_with_answer("Use **improve clarity** here. 🦊");
    let valid = request
        .build_response(&response(
            &candidate,
            "/usage",
            "assistant",
            "**improve clarity**",
        ))
        .unwrap();
    let mut saved: wordweave5::material::Draft =
        serde_json::from_str(&serde_json::to_string(&valid).unwrap()).unwrap();
    saved.reasons[0].quotes[0].quote = "improve **clarity**".into();
    let before = serde_json::to_value(&saved).unwrap();

    let failure = saved.validate_evidence_detailed().unwrap_err();
    let detail = diagnostic(&failure);
    assert_eq!(detail.stage, DiagnosticStage::SavedDraft);
    assert_eq!(detail.cause, EvidenceCause::QuoteMismatch);
    assert_eq!(
        detail.evidence.as_ref().unwrap().quote,
        "improve **clarity**"
    );
    assert_eq!(
        detail.evidence.as_ref().unwrap().original.as_deref(),
        Some("Use **improve clarity** here. 🦊")
    );
    assert_eq!(
        failure.legacy_message(),
        saved.validate_evidence().unwrap_err()
    );
    assert_eq!(serde_json::to_value(&saved).unwrap(), before);
}

#[test]
fn n02_saved_draft_source_failure_precedes_reasons_and_invents_no_quote() {
    let (request, candidate) = request_with_answer("固定回答");
    let mut saved = request
        .build_response(&response(&candidate, "/usage", "assistant", "固定回答"))
        .unwrap();
    saved.source.snapshots[0].exchange_index = 1;
    saved.reasons[0].quotes[0].quote = "不一致の引用".into();
    let before = serde_json::to_value(&saved).unwrap();

    let failure = saved.validate_evidence_detailed().unwrap_err();
    let detail = diagnostic(&failure);
    assert_eq!(detail.stage, DiagnosticStage::SavedDraft);
    assert_eq!(detail.cause, EvidenceCause::SourceReference);
    assert!(detail.reason_number.is_none());
    assert!(detail.evidence.is_none());
    assert_eq!(
        failure.legacy_message(),
        saved.validate_evidence().unwrap_err()
    );
    assert_eq!(serde_json::to_value(&saved).unwrap(), before);
}
