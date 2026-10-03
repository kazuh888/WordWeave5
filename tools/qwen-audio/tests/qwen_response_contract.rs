use qwen_audio::{validate_feedback, Assessment, ErrorCode, Feedback, ReferenceText, SafeError};
use serde_json::{json, Value};

const REFERENCE: &str = "The cat is sleeping.";
const OTHER_SENTENCE: &str = "I went to school yesterday.";

fn mismatch() -> Value {
    json!({
        "assessment": "reference_mismatch",
        "heard_text": OTHER_SENTENCE,
        "summary": "別の英文が聞き取れた。表示された例文を読み直して録音する。",
        "strengths": [],
        "improvements": [],
        "unassessable_reason": null
    })
}

fn validate(value: &Value) -> Result<Feedback, SafeError> {
    validate_feedback(
        &serde_json::to_vec(value).unwrap(),
        &ReferenceText::new(REFERENCE.to_owned()).unwrap(),
    )
}

// QS-AC-001/002: a declared mismatch is a valid result, never a pronunciation grade.
#[test]
fn valid_reference_mismatch_preserves_heard_sentence_and_confirmed_reference() {
    let reference = ReferenceText::new(REFERENCE.to_owned()).unwrap();
    let feedback = validate_feedback(&serde_json::to_vec(&mismatch()).unwrap(), &reference)
        .expect("a schema-conforming mismatch must be accepted");
    assert_eq!(
        serde_json::to_value(&feedback.assessment).unwrap(),
        json!("reference_mismatch")
    );
    assert_eq!(feedback.heard_text.as_deref(), Some(OTHER_SENTENCE));
    assert!(feedback.strengths.is_empty());
    assert!(feedback.improvements.is_empty());
    assert_eq!(feedback.unassessable_reason, None);
    assert_eq!(reference.as_str(), REFERENCE);
    assert_eq!(serde_json::to_value(feedback).unwrap(), mismatch());
}

// QS-AC-002/003: every field, including explicit nulls, remains mandatory.
#[test]
fn mismatch_requires_all_six_fields() {
    for field in [
        "assessment",
        "heard_text",
        "summary",
        "strengths",
        "improvements",
        "unassessable_reason",
    ] {
        let mut value = mismatch();
        value.as_object_mut().unwrap().remove(field);
        assert_eq!(
            validate(&value).unwrap_err().code(),
            ErrorCode::ResponseFieldsInvalid,
            "missing {field}"
        );
    }
}

#[test]
fn mismatch_rejects_null_blank_or_wrong_type_heard_text() {
    for text in [Value::Null, json!(""), json!(" \t\n　"), json!(42)] {
        let mut value = mismatch();
        value["heard_text"] = text;
        assert_eq!(
            validate(&value).unwrap_err().code(),
            ErrorCode::ResponseFieldsInvalid
        );
    }
}

#[test]
fn mismatch_rejects_grades_improvements_and_unassessable_reason() {
    for (field, replacement) in [
        ("strengths", json!(["clear pronunciation"])),
        (
            "improvements",
            json!([{
                "reference_excerpt": "cat",
                "observation": "pronunciation observation",
                "practice": "pronunciation practice"
            }]),
        ),
        ("unassessable_reason", json!("unable to hear")),
    ] {
        let mut value = mismatch();
        value[field] = replacement;
        assert_eq!(
            validate(&value).unwrap_err().code(),
            ErrorCode::ResponseFieldsInvalid,
            "inconsistent {field}"
        );
    }
}

#[test]
fn mismatch_rejects_unknown_fields_wrong_types_and_malformed_json() {
    for (field, replacement) in [
        ("score", json!(100)),
        ("assessment", json!("mismatch")),
        ("summary", json!(false)),
        ("strengths", Value::Null),
        ("improvements", json!({})),
    ] {
        let mut value = mismatch();
        value[field] = replacement;
        assert_eq!(
            validate(&value).unwrap_err().code(),
            ErrorCode::ResponseFieldsInvalid
        );
    }
    let reference = ReferenceText::new(REFERENCE.to_owned()).unwrap();
    let fenced = format!("```json\n{}\n```", mismatch());
    for (bytes, code) in [
        (b"{invalid".as_slice(), ErrorCode::ResponseJsonInvalid),
        (fenced.as_bytes(), ErrorCode::ResponseJsonInvalid),
        (b"null".as_slice(), ErrorCode::ResponseFieldsInvalid),
    ] {
        assert_eq!(
            validate_feedback(bytes, &reference).unwrap_err().code(),
            code
        );
    }
}

// QS-AC-003: scalar limits remain inclusive, independently of UTF-8 byte count.
#[test]
fn mismatch_text_limits_accept_exact_boundary_and_reject_one_more_scalar() {
    for (field, limit) in [("heard_text", 10_000), ("summary", 2_000)] {
        let mut value = mismatch();
        value[field] = json!("🌱".repeat(limit));
        assert!(validate(&value).is_ok(), "inclusive {field} scalar limit");
        value[field] = json!("🌱".repeat(limit + 1));
        assert_eq!(
            validate(&value).unwrap_err().code(),
            ErrorCode::ResponseFieldsInvalid
        );
        value[field] = json!(" \t　");
        assert_eq!(
            validate(&value).unwrap_err().code(),
            ErrorCode::ResponseFieldsInvalid
        );
    }
}

#[test]
fn mismatch_json_byte_limit_is_inclusive() {
    let reference = ReferenceText::new(REFERENCE.to_owned()).unwrap();
    let mut bytes = serde_json::to_vec(&mismatch()).unwrap();
    // JSON trailing whitespace is valid; fields remain inside their scalar limits.
    bytes.resize(65_536, b' ');
    assert!(validate_feedback(&bytes, &reference).is_ok());
    bytes.push(b' ');
    assert_eq!(
        validate_feedback(&bytes, &reference).unwrap_err().code(),
        ErrorCode::ResponseTooLarge
    );
}

// QS-AC-001/002/003: preserve both prior statuses and never infer a new status.
#[test]
fn assessed_different_transcription_is_not_automatically_reclassified() {
    let mut value = mismatch();
    value["assessment"] = json!("assessed");
    value["strengths"] = json!(["Some sounds were clear."]);
    value["improvements"] = json!([{
        "reference_excerpt": "cat",
        "observation": "Listen for the final consonant.",
        "practice": "Read cat again."
    }]);
    let feedback = validate(&value).unwrap();
    assert_eq!(feedback.assessment, Assessment::Assessed);
    assert_eq!(feedback.heard_text.as_deref(), Some(OTHER_SENTENCE));
    assert_eq!(feedback.improvements.len(), 1);
}

#[test]
fn unassessable_preserves_reason_and_never_fills_heard_text_from_reference() {
    let value = json!({
        "assessment": "unassessable",
        "heard_text": null,
        "summary": "音声を評価できない。再録音する。",
        "strengths": [],
        "improvements": [],
        "unassessable_reason": "無音である。"
    });
    let feedback = validate(&value).unwrap();
    assert_eq!(feedback.assessment, Assessment::Unassessable);
    assert_eq!(feedback.heard_text, None);
    assert_eq!(
        feedback.unassessable_reason.as_deref(),
        Some("無音である。")
    );
}
