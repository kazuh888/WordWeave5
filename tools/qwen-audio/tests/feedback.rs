#[path = "support/provider.rs"]
mod fixture;
mod support;

use qwen_audio::{validate_feedback, Assessment, ErrorCode, ReferenceText};
use serde_json::{json, Value};

fn validate(value: &Value) -> Result<qwen_audio::Feedback, qwen_audio::SafeError> {
    validate_feedback(
        &serde_json::to_vec(value).unwrap(),
        &ReferenceText::new(fixture::REFERENCE.to_owned()).unwrap(),
    )
}

#[test]
fn assessed_and_unassessable_are_distinct_valid_results() {
    let assessed = validate(&fixture::feedback()).unwrap();
    assert_eq!(assessed.assessment, Assessment::Assessed);
    assert_eq!(assessed.heard_text.as_deref(), Some(fixture::REFERENCE));
    let value = json!({"assessment":"unassessable","heard_text":null,"summary":"安全に評価できない。","strengths":[],"improvements":[],"unassessable_reason":"音声を聞き取れない。"});
    let unable = validate(&value).unwrap();
    assert_eq!(unable.assessment, Assessment::Unassessable);
    assert_eq!(unable.heard_text, None, "never fill from reference");
    assert_eq!(
        unable.unassessable_reason.as_deref(),
        Some("音声を聞き取れない。")
    );
}

#[test]
fn every_field_including_explicit_null_fields_is_required() {
    for field in [
        "assessment",
        "heard_text",
        "summary",
        "strengths",
        "improvements",
        "unassessable_reason",
    ] {
        let mut value = fixture::feedback();
        value.as_object_mut().unwrap().remove(field);
        assert_eq!(
            validate(&value).unwrap_err().code(),
            ErrorCode::ResponseFieldsInvalid,
            "missing {field}"
        );
    }
}

#[test]
fn unknown_fields_types_enum_values_and_code_fences_are_rejected() {
    let mut unknown = fixture::feedback();
    unknown["score"] = json!(100);
    let mut nested = fixture::feedback();
    nested["improvements"][0]["score"] = json!(100);
    let mut wrong_type = fixture::feedback();
    wrong_type["summary"] = json!(123);
    let mut unknown_enum = fixture::feedback();
    unknown_enum["assessment"] = json!("limited");
    for value in [unknown, nested, wrong_type, unknown_enum] {
        assert_eq!(
            validate(&value).unwrap_err().code(),
            ErrorCode::ResponseFieldsInvalid
        );
    }
    let fenced = format!("```json\n{}\n```", fixture::feedback());
    assert!(validate_feedback(
        fenced.as_bytes(),
        &ReferenceText::new(fixture::REFERENCE.to_owned()).unwrap()
    )
    .is_err());
}

#[test]
fn assessment_heard_text_reason_and_arrays_must_be_consistent() {
    let mut heard_null = fixture::feedback();
    heard_null["heard_text"] = Value::Null;
    let mut reason = fixture::feedback();
    reason["unassessable_reason"] = json!("reason");
    let mut unable_with_heard = fixture::feedback();
    unable_with_heard["assessment"] = json!("unassessable");
    unable_with_heard["unassessable_reason"] = json!("reason");
    let mut unable_with_arrays = unable_with_heard.clone();
    unable_with_arrays["heard_text"] = Value::Null;
    for value in [heard_null, reason, unable_with_heard, unable_with_arrays] {
        assert_eq!(
            validate(&value).unwrap_err().code(),
            ErrorCode::ResponseFieldsInvalid
        );
    }
}

#[test]
fn natural_text_limits_count_scalars_and_reject_blank_values() {
    for (field, limit) in [("heard_text", 10_000), ("summary", 2_000)] {
        let mut value = fixture::feedback();
        value[field] = json!("🌱".repeat(limit));
        assert!(validate(&value).is_ok(), "inclusive {field} boundary");
        for text in ["🌱".repeat(limit + 1), " \n\u{3000}".to_owned()] {
            value[field] = json!(text);
            assert_eq!(
                validate(&value).unwrap_err().code(),
                ErrorCode::ResponseFieldsInvalid
            );
        }
    }
    let mut value = fixture::feedback();
    value["strengths"] = json!(["🌱".repeat(1_000)]);
    assert!(validate(&value).is_ok());
    value["strengths"] = json!(["🌱".repeat(1_001)]);
    assert!(validate(&value).is_err());
    for field in ["observation", "practice"] {
        let mut value = fixture::feedback();
        value["improvements"][0][field] = json!("a".repeat(1_000));
        assert!(validate(&value).is_ok());
        value["improvements"][0][field] = json!("a".repeat(1_001));
        assert!(validate(&value).is_err());
        value["improvements"][0][field] = json!(" \t");
        assert!(validate(&value).is_err());
    }
}

#[test]
fn arrays_are_bounded_and_excerpt_must_be_in_confirmed_reference() {
    let mut value = fixture::feedback();
    value["strengths"] = json!(vec!["strength"; 8]);
    let improvement = value["improvements"][0].clone();
    value["improvements"] = json!(vec![improvement.clone(); 8]);
    assert!(validate(&value).is_ok());
    value["strengths"] = json!(vec!["strength"; 9]);
    assert!(validate(&value).is_err());
    value["strengths"] = json!([]);
    value["improvements"] = json!(vec![improvement; 9]);
    assert!(validate(&value).is_err());
    value["improvements"] = json!([{"reference_excerpt":"invented phrase","observation":"observation","practice":"practice"}]);
    assert!(validate(&value).is_err());
    value["improvements"][0]["reference_excerpt"] = json!(" ");
    assert!(
        validate(&value).is_err(),
        "substring alone does not allow blank excerpt"
    );
}

#[test]
fn oversized_json_is_rejected_before_content_can_be_used() {
    let bytes = vec![b' '; 64 * 1024 + 1];
    assert!(validate_feedback(
        &bytes,
        &ReferenceText::new(fixture::REFERENCE.to_owned()).unwrap()
    )
    .is_err());
}
