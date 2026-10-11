//! I1a: specification-derived DTO boundaries; no filesystem, credentials or API.
use std::collections::BTreeMap;

use acoustic_analysis::AnalysisProfile;
use practice_history::*;
use serde::{de::DeserializeOwned, Serialize};
use serde_json::{json, Value};

const TRIAL: &str = "12345678-1234-4234-8234-123456789abc";
const TRIAL_2: &str = "12345678-1234-4234-8234-123456789abd";
const RUN: &str = "abcdef01-1234-4234-8234-123456789abc";
const OP: &str = "abcdef02-1234-4234-8234-123456789abc";
const MIB: u64 = 1024 * 1024;

// Deliberately bypass parse(): a broken constructor must not break all fixtures.
fn decode<T: DeserializeOwned>(value: Value) -> T {
    serde_json::from_value(value).expect("synthetic fixture must decode")
}
fn hash(digit: char) -> Sha256 {
    decode(json!(digit.to_string().repeat(64)))
}
fn at() -> RecordedAt {
    RecordedAt {
        utc_millis: 1_700_000_000_000,
        offset_minutes: 540,
    }
}
fn blob() -> BlobRef {
    BlobRef {
        sha256: hash('a'),
        bytes: 100,
    }
}
fn audio() -> AudioSnapshot {
    AudioSnapshot {
        original: blob(),
        format: AudioFormat::Wav,
        rate: 16_000,
        channels: 1,
        frames: 16_000,
        origin: Origin::Imported,
        warnings: vec![],
    }
}
fn trial() -> TrialRevision {
    TrialRevision {
        schema_version: 1,
        id: decode(json!(TRIAL)),
        revision: 1,
        previous: None,
        created_at: at(),
        input: FrozenInput {
            audio: audio(),
            reference: ReferenceSnapshot {
                text: "Read this sentence.".into(),
                sha256: hash('b'),
                source: None,
            },
            goal: None,
            capture: CaptureProfile {
                device: None,
                distance: None,
                gain: None,
                environment: None,
                confirmed_at: None,
                version: hash('c'),
            },
        },
        claim: PracticeClaim::Reference,
        match_evidence: vec![],
        analysis: BTreeMap::new(),
        advice: BTreeMap::new(),
        reflections: vec![],
        trashed_at: None,
    }
}
fn analysis() -> AnalysisRun {
    AnalysisRun {
        schema_version: 1,
        id: decode(json!(RUN)),
        trial: decode(json!(TRIAL)),
        audio: hash('a'),
        pcm: hash('d'),
        decoder: DecoderIdentity {
            name: "fixture-decoder".into(),
            version: "1".into(),
            binary_sha256: None,
        },
        profile: AnalysisProfile::v1(),
        state: AnalysisRunState::Running,
        started_at: at(),
        ended_at: None,
        result: None,
        failure: None,
    }
}
fn advice() -> AdviceRun {
    AdviceRun {
        schema_version: 1,
        id: decode(json!(RUN)),
        trial: decode(json!(TRIAL)),
        audio: hash('a'),
        reference: hash('b'),
        confirmed_at: at(),
        confirmation: hash('e'),
        destination: "https://fixture.invalid".into(),
        versions: AdviceVersions {
            provider: "fixture".into(),
            prompt: "1".into(),
            parser: "1".into(),
        },
        state: AdviceRunState::Sending,
        send: SendCertainty::MayHaveBeenSent,
        started_at: at(),
        ended_at: None,
        result: None,
        failure: None,
    }
}
fn result() -> AdviceResult {
    AdviceResult {
        schema_version: 1,
        assessment: AdviceAssessment::Assessed,
        heard_text: Some("Read this sentence.".into()),
        summary: "Synthetic feedback.".into(),
        strengths: vec!["Synthetic strength.".into()],
        improvements: vec![AdviceImprovement {
            reference_excerpt: "sentence".into(),
            observation: "Synthetic observation.".into(),
            practice: "Repeat once.".into(),
        }],
        unassessable_reason: None,
        requested_model: "requested-model".into(),
        requested_effort: "requested-effort".into(),
        actual_model: None,
        actual_effort: None,
        usage: None,
    }
}
fn reflection() -> Reflection {
    Reflection {
        schema_version: 1,
        trial: decode(json!(TRIAL)),
        at: at(),
        text: "Synthetic reflection.".into(),
    }
}
fn root() -> HistoryRoot {
    HistoryRoot {
        schema_version: 1,
        revision: 1,
        next_save_sequence: 2,
        trials: BTreeMap::from([(
            decode(json!(TRIAL)),
            TrialHead {
                document: blob(),
                admitted_sequence: 1,
            },
        )]),
        receipts: BTreeMap::from([(
            decode(json!(OP)),
            OperationReceipt {
                request: hash('f'),
                committed_revision: 1,
                affected: vec![decode(json!(TRIAL))],
            },
        )]),
    }
}
fn assert_code(value: Result<(), ContractError>, code: ContractErrorCode) {
    let error = value.expect_err("invalid fixture must be rejected, not repaired");
    assert_eq!(error.code, code);
    assert!(!error.field.is_empty());
}
fn required_options<T: DeserializeOwned + Serialize>(base: &T, keys: &[&str]) {
    for key in keys {
        let mut value = serde_json::to_value(base).unwrap();
        value[*key] = Value::Null;
        assert!(
            serde_json::from_value::<T>(value.clone()).is_ok(),
            "null is valid for {key}"
        );
        value.as_object_mut().unwrap().remove(*key);
        assert!(
            serde_json::from_value::<T>(value).is_err(),
            "missing {key} must not default to None"
        );
    }
}
fn unknown_field<T: DeserializeOwned + Serialize>(base: &T) {
    for key in [
        "api_key",
        "raw_response",
        "credential_store",
        "original_path",
        "future_field",
    ] {
        let mut value = serde_json::to_value(base).unwrap();
        value[key] = json!("synthetic-secret-sentinel");
        assert!(
            serde_json::from_value::<T>(value).is_err(),
            "unknown {key} must fail closed"
        );
    }
}

#[test]
fn canonical_uuid_parsers_preserve_exact_ids_for_all_three_roles() {
    assert_eq!(TrialId::parse(TRIAL).unwrap().as_str(), TRIAL);
    assert_eq!(RunId::parse(RUN).unwrap().as_str(), RUN);
    assert_eq!(OperationId::parse(OP).unwrap().as_str(), OP);
}

#[test]
fn uuid_parsers_reject_noncanonical_spelling_without_normalizing() {
    for bad in [
        TRIAL.to_uppercase(),
        TRIAL.replace('-', ""),
        format!("{{{TRIAL}}}"),
        format!(" {TRIAL}"),
        "not-an-id".into(),
    ] {
        assert_eq!(
            TrialId::parse(&bad).unwrap_err().code,
            ContractErrorCode::InvalidId
        );
        assert_eq!(
            RunId::parse(&bad).unwrap_err().code,
            ContractErrorCode::InvalidId
        );
        assert_eq!(
            OperationId::parse(&bad).unwrap_err().code,
            ContractErrorCode::InvalidId
        );
    }
}

#[test]
fn sha256_requires_exactly_64_lowercase_hex_characters() {
    let good = "0123456789abcdef".repeat(4);
    assert_eq!(Sha256::parse(&good).unwrap().as_str(), good);
    for bad in [
        "a".repeat(63),
        "a".repeat(65),
        "A".repeat(64),
        "g".repeat(64),
        format!(" {good}"),
    ] {
        assert_eq!(
            Sha256::parse(&bad).unwrap_err().code,
            ContractErrorCode::InvalidHash
        );
    }
}

#[test]
fn deserialized_identifiers_still_require_boundary_validation() {
    assert_code(
        decode::<TrialId>(json!("invalid")).validate(),
        ContractErrorCode::InvalidId,
    );
    assert_code(
        decode::<RunId>(json!("invalid")).validate(),
        ContractErrorCode::InvalidId,
    );
    assert_code(
        decode::<OperationId>(json!("invalid")).validate(),
        ContractErrorCode::InvalidId,
    );
    assert_code(
        decode::<Sha256>(json!("INVALID")).validate(),
        ContractErrorCode::InvalidHash,
    );
}

#[test]
fn required_optional_keys_distinguish_null_from_missing_in_trial_input() {
    let t = trial();
    required_options(&t, &["previous", "trashed_at"]);
    required_options(&t.input, &["goal"]);
    required_options(&t.input.reference, &["source"]);
    required_options(
        &t.input.capture,
        &["device", "distance", "gain", "environment", "confirmed_at"],
    );
}

#[test]
fn required_optional_keys_distinguish_null_from_missing_in_runs_and_advice() {
    required_options(&analysis(), &["ended_at", "result", "failure"]);
    required_options(&analysis().decoder, &["binary_sha256"]);
    required_options(&advice(), &["ended_at", "result", "failure"]);
    required_options(
        &result(),
        &[
            "heard_text",
            "unassessable_reason",
            "actual_model",
            "actual_effort",
            "usage",
        ],
    );
    required_options(
        &AdviceUsage {
            prompt_tokens: None,
            completion_tokens: None,
            total_tokens: None,
        },
        &["prompt_tokens", "completion_tokens", "total_tokens"],
    );
}

#[test]
fn unknown_and_secret_fields_are_rejected_at_document_and_nested_boundaries() {
    unknown_field(&trial());
    unknown_field(&analysis());
    unknown_field(&advice());
    unknown_field(&result());
    unknown_field(&reflection());
    unknown_field(&root());
    unknown_field(&trial().input);
    unknown_field(&trial().input.audio);
    unknown_field(&trial().input.reference);
    unknown_field(&trial().input.capture);
    unknown_field(&result().improvements[0]);
    unknown_field(&blob());
}

#[test]
fn required_nonoptional_keys_and_schema_do_not_default_or_accept_null() {
    let original = serde_json::to_value(trial()).unwrap();
    for key in [
        "schema_version",
        "id",
        "revision",
        "created_at",
        "input",
        "claim",
        "analysis",
        "advice",
        "match_evidence",
        "reflections",
    ] {
        let mut value = original.clone();
        value.as_object_mut().unwrap().remove(key);
        assert!(
            serde_json::from_value::<TrialRevision>(value).is_err(),
            "missing {key}"
        );
        let mut value = original.clone();
        value[key] = Value::Null;
        assert!(
            serde_json::from_value::<TrialRevision>(value).is_err(),
            "null {key}"
        );
    }
}

#[test]
fn unknown_document_versions_fail_closed() {
    for version in [0, 2, u32::MAX] {
        let mut t = trial();
        t.schema_version = version;
        assert_code(t.validate(), ContractErrorCode::UnknownVersion);
        let mut a = analysis();
        a.schema_version = version;
        assert_code(a.validate(), ContractErrorCode::UnknownVersion);
        let mut a = advice();
        a.schema_version = version;
        assert_code(a.validate(), ContractErrorCode::UnknownVersion);
        let mut a = result();
        a.schema_version = version;
        assert_code(a.validate(), ContractErrorCode::UnknownVersion);
        let mut r = reflection();
        r.schema_version = version;
        assert_code(r.validate(), ContractErrorCode::UnknownVersion);
        let mut r = root();
        r.schema_version = version;
        assert_code(r.validate(), ContractErrorCode::UnknownVersion);
    }
}

#[test]
fn reference_is_nonblank_scalar_bounded_and_not_normalized() {
    let mut reference = trial().input.reference;
    reference.text = "  A\nＢ🙂  ".into();
    let before = reference.clone();
    assert!(reference.validate().is_ok());
    assert_eq!(reference, before);
    reference.text = "🙂".repeat(10_000);
    assert!(reference.validate().is_ok());
    reference.text.push('🙂');
    assert_code(reference.validate(), ContractErrorCode::LimitExceeded);
    for text in ["", " \r\n\t", "\u{3000}"] {
        reference.text = text.into();
        assert_code(reference.validate(), ContractErrorCode::InvalidValue);
    }
}

#[test]
fn goals_and_reflections_count_unicode_scalars_without_truncation() {
    let mut goal = GoalSnapshot {
        text: "🙂".repeat(1_000),
        version: hash('c'),
    };
    assert!(goal.validate().is_ok());
    goal.text.push('🙂');
    let before = goal.clone();
    assert_code(goal.validate(), ContractErrorCode::LimitExceeded);
    assert_eq!(goal, before);
    let mut note = reflection();
    note.text = "🙂".repeat(2_000);
    assert!(note.validate().is_ok());
    note.text.push('🙂');
    assert_code(note.validate(), ContractErrorCode::LimitExceeded);
}

#[test]
fn audio_accepts_source_byte_and_duration_boundaries_independent_of_pcm_size() {
    for (rate, channels) in [(8_000, 1), (48_000, 2)] {
        let mut a = audio();
        a.rate = rate;
        a.channels = channels;
        a.frames = u64::from(rate) * 60;
        a.original.bytes = 6 * MIB;
        a.format = AudioFormat::Mp3;
        assert!(
            a.validate().is_ok(),
            "legal source; decoded PCM may exceed 6MiB"
        );
    }
}

#[test]
fn audio_rejects_empty_oversized_duration_rate_and_channel_inputs() {
    let mut cases = Vec::new();
    let mut a = audio();
    a.original.bytes = 6 * MIB + 1;
    cases.push(a);
    let mut a = audio();
    a.original.bytes = 0;
    cases.push(a);
    let mut a = audio();
    a.frames = 16_000 * 60 + 1;
    cases.push(a);
    let mut a = audio();
    a.frames = 0;
    cases.push(a);
    let mut a = audio();
    a.frames = u64::MAX;
    cases.push(a);
    for rate in [0, 7_999, 48_001] {
        let mut a = audio();
        a.rate = rate;
        cases.push(a);
    }
    for channels in [0, 3] {
        let mut a = audio();
        a.channels = channels;
        cases.push(a);
    }
    for a in cases {
        assert!(a.validate().is_err(), "invalid audio accepted: {a:?}");
    }
}

#[test]
fn imported_unconfirmed_audio_remains_reference_without_fabricated_practice() {
    let t = trial();
    assert!(t.validate().is_ok());
    let restored: TrialRevision = decode(serde_json::to_value(&t).unwrap());
    assert_eq!(restored.claim, PracticeClaim::Reference);
    assert!(restored.match_evidence.is_empty());
    assert_eq!(restored.input.goal, None);
    assert_eq!(restored.input.reference.source, None);
}

#[test]
fn practice_claim_cannot_omit_speaker_or_time_confirmation() {
    let valid = json!({"practice": {
        "performed_at": at(), "self_speaker_confirmed_at": at(),
        "time_confirmed_at": at(), "duplicate_confirmed": false
    }});
    assert!(serde_json::from_value::<PracticeClaim>(valid.clone()).is_ok());
    for key in [
        "performed_at",
        "self_speaker_confirmed_at",
        "time_confirmed_at",
        "duplicate_confirmed",
    ] {
        let mut value = valid.clone();
        value["practice"].as_object_mut().unwrap().remove(key);
        assert!(
            serde_json::from_value::<PracticeClaim>(value).is_err(),
            "missing {key}"
        );
    }
}

#[test]
fn unknown_capture_values_use_none_and_not_blank_known_strings() {
    let mut capture = trial().input.capture;
    assert!(capture.validate().is_ok());
    for field in ["device", "distance", "gain", "environment"] {
        let mut value = serde_json::to_value(&capture).unwrap();
        value[field] = json!(" \t ");
        assert!(
            decode::<CaptureProfile>(value).validate().is_err(),
            "blank {field}"
        );
    }
    capture.device = Some("synthetic microphone".into());
    assert!(capture.validate().is_ok());
}

#[test]
fn human_match_evidence_must_bind_frozen_audio_and_reference_hashes() {
    let mut t = trial();
    t.match_evidence.push(MatchEvidence::HumanConfirmed {
        at: at(),
        audio: hash('a'),
        reference: hash('b'),
    });
    assert!(t.validate().is_ok());
    for (audio, reference) in [(hash('d'), hash('b')), (hash('a'), hash('d'))] {
        t.match_evidence = vec![MatchEvidence::HumanConfirmed {
            at: at(),
            audio,
            reference,
        }];
        assert_code(t.validate(), ContractErrorCode::ReferenceMismatch);
    }
}

#[test]
fn ai_evidence_must_refer_to_an_advice_run_in_the_same_trial() {
    for evidence in [
        MatchEvidence::AiAssessed {
            run: decode(json!(RUN)),
        },
        MatchEvidence::AiMismatch {
            run: decode(json!(RUN)),
        },
        MatchEvidence::AiUnassessable {
            run: decode(json!(RUN)),
        },
    ] {
        let mut t = trial();
        t.match_evidence.push(evidence);
        assert!(t.validate().is_err(), "dangling evidence must fail");
        t.analysis.insert(decode(json!(RUN)), blob());
        assert!(t.validate().is_err(), "analysis run is not advice evidence");
        t.analysis.clear();
        t.advice.insert(decode(json!(RUN)), blob());
        assert!(t.validate().is_ok());
    }
}

#[test]
fn conflicting_match_evidence_and_past_results_survive_roundtrip() {
    let mut t = trial();
    t.advice.insert(decode(json!(RUN)), blob());
    t.analysis.insert(
        decode(json!(OP)),
        BlobRef {
            sha256: hash('d'),
            bytes: 200,
        },
    );
    t.match_evidence = vec![
        MatchEvidence::HumanConfirmed {
            at: at(),
            audio: hash('a'),
            reference: hash('b'),
        },
        MatchEvidence::AiMismatch {
            run: decode(json!(RUN)),
        },
    ];
    let before = t.clone();
    assert!(
        t.validate().is_ok(),
        "conflict is preserved for comparison to exclude later"
    );
    assert_eq!(
        decode::<TrialRevision>(serde_json::to_value(&t).unwrap()),
        before
    );
}

#[test]
fn analysis_running_state_cannot_claim_an_end_or_saved_result() {
    let a = analysis();
    assert!(a.validate().is_ok());
    let mut bad = a.clone();
    bad.ended_at = Some(at());
    assert!(bad.validate().is_err());
    let mut bad = a;
    bad.result = Some(blob());
    assert!(bad.validate().is_err());
}

#[test]
fn analysis_success_states_require_result_and_terminal_time() {
    for state in [AnalysisRunState::Complete, AnalysisRunState::Partial] {
        let mut a = analysis();
        a.state = state;
        a.ended_at = Some(at());
        a.result = Some(blob());
        assert!(a.validate().is_ok());
        let mut bad = a.clone();
        bad.ended_at = None;
        assert!(bad.validate().is_err());
        let mut bad = a;
        bad.result = None;
        assert!(bad.validate().is_err());
    }
}

#[test]
fn advice_sending_state_cannot_claim_a_saved_result_or_completion() {
    let a = advice();
    assert!(a.validate().is_ok());
    let mut bad = a.clone();
    bad.ended_at = Some(at());
    assert!(bad.validate().is_err());
    let mut bad = a;
    bad.result = Some(blob());
    assert!(bad.validate().is_err());
}

#[test]
fn advice_three_response_states_require_result_and_terminal_time() {
    for state in [
        AdviceRunState::Assessed,
        AdviceRunState::Unassessable,
        AdviceRunState::ReferenceMismatch,
    ] {
        let mut a = advice();
        a.state = state;
        a.send = SendCertainty::ResponseReceived;
        a.ended_at = Some(at());
        a.result = Some(blob());
        assert!(a.validate().is_ok());
        let mut bad = a.clone();
        bad.ended_at = None;
        assert!(bad.validate().is_err());
        let mut bad = a;
        bad.result = None;
        assert!(bad.validate().is_err());
    }
}

#[test]
fn failure_codes_are_fixed_identifiers_not_raw_messages() {
    let mut failure = FailureCode {
        domain: FailureDomain::Advice,
        code: "timeout_2".into(),
    };
    assert!(failure.validate().is_ok());
    failure.code = "a".repeat(64);
    assert!(failure.validate().is_ok());
    for code in [
        "".into(),
        "a".repeat(65),
        "HTTP 403 secret".into(),
        "secret-token".into(),
        "エラー".into(),
        "https://fixture.invalid".into(),
    ] {
        failure.code = code;
        let error = failure.validate().unwrap_err();
        assert!(!error.field.contains(&failure.code) || failure.code.is_empty());
    }
}

#[test]
fn advice_missing_returned_metadata_is_preserved_as_none() {
    let r = result();
    assert!(r.validate().is_ok());
    let encoded = serde_json::to_value(&r).unwrap();
    assert_eq!(encoded["actual_model"], Value::Null);
    assert_eq!(encoded["actual_effort"], Value::Null);
    assert_eq!(encoded["usage"], Value::Null);
    let restored: AdviceResult = decode(encoded);
    assert_eq!(restored.actual_model, None);
    assert_eq!(restored.actual_effort, None);
    assert_eq!(restored.requested_model, "requested-model");
    assert_eq!(restored.requested_effort, "requested-effort");
}

#[test]
fn advice_usage_nullable_counters_are_not_fabricated_as_zero() {
    let usage = AdviceUsage {
        prompt_tokens: Some(10),
        completion_tokens: None,
        total_tokens: None,
    };
    assert!(usage.validate().is_ok());
    let restored: AdviceUsage = decode(serde_json::to_value(&usage).unwrap());
    assert_eq!(restored, usage);
    let invalid = json!({"prompt_tokens": -1, "completion_tokens": null, "total_tokens": null});
    assert!(serde_json::from_value::<AdviceUsage>(invalid).is_err());
}

#[test]
fn contract_error_field_never_contains_untrusted_reference_text() {
    let sentinel = "SYNTHETIC_SECRET_SENTINEL";
    let mut reference = trial().input.reference;
    reference.text = sentinel.repeat(1_000);
    let before = reference.clone();
    let error = reference.validate().unwrap_err();
    assert_eq!(error.code, ContractErrorCode::LimitExceeded);
    assert!(!error.field.contains(sentinel));
    assert!(!format!("{error:?}").contains(sentinel));
    assert_eq!(reference, before);
}

#[test]
fn blob_and_root_references_apply_distinct_nonempty_byte_limits() {
    for bytes in [1, 12 * MIB] {
        assert!(BlobRef { bytes, ..blob() }.validate().is_ok());
    }
    for bytes in [0, 12 * MIB + 1, u64::MAX] {
        assert!(BlobRef { bytes, ..blob() }.validate().is_err());
    }
    assert!(RootRef::new(BlobRef {
        bytes: 8 * MIB,
        ..blob()
    })
    .validate()
    .is_ok());
    assert_code(
        RootRef::new(BlobRef {
            bytes: 8 * MIB + 1,
            ..blob()
        })
        .validate(),
        ContractErrorCode::LimitExceeded,
    );
    assert!(RootRef::new(BlobRef { bytes: 0, ..blob() })
        .validate()
        .is_err());
    let reference = RootRef::new(blob());
    assert_eq!(
        serde_json::to_value(&reference).unwrap(),
        serde_json::to_value(blob()).unwrap()
    );
    assert_eq!(reference.as_blob(), &blob());
}

#[test]
fn recorded_at_validates_gregorian_and_offset_boundaries_without_rewriting() {
    for utc_millis in [-62_135_596_800_000, 253_402_300_799_999] {
        for offset_minutes in [-1439, 0, 1439] {
            let value = RecordedAt {
                utc_millis,
                offset_minutes,
            };
            assert!(value.validate().is_ok());
            assert_eq!(
                decode::<RecordedAt>(serde_json::to_value(value).unwrap()),
                value
            );
        }
    }
    for utc_millis in [-62_135_596_800_001, 253_402_300_800_000, i64::MIN, i64::MAX] {
        assert!(RecordedAt { utc_millis, ..at() }.validate().is_err());
    }
    for offset_minutes in [-1440, 1440, i16::MIN, i16::MAX] {
        assert!(RecordedAt {
            offset_minutes,
            ..at()
        }
        .validate()
        .is_err());
    }
}

#[test]
fn root_sequences_are_unique_positive_and_below_next_sequence() {
    let empty = HistoryRoot {
        schema_version: 1,
        revision: 0,
        next_save_sequence: 1,
        trials: BTreeMap::new(),
        receipts: BTreeMap::new(),
    };
    assert!(empty.validate().is_ok());
    assert!(root().validate().is_ok());
    let mut bad = root();
    bad.next_save_sequence = 1;
    assert!(bad.validate().is_err());
    let mut bad = root();
    bad.revision = 0;
    assert!(bad.validate().is_err());
    let mut bad = root();
    bad.trials.values_mut().next().unwrap().admitted_sequence = 0;
    assert!(bad.validate().is_err());
    let mut bad = root();
    bad.trials.insert(
        decode(json!(TRIAL_2)),
        TrialHead {
            document: blob(),
            admitted_sequence: 1,
        },
    );
    assert!(bad.validate().is_err());
    let mut bad = empty;
    bad.next_save_sequence = 0;
    assert!(bad.validate().is_err());
}

#[test]
fn receipts_cannot_claim_uncommitted_root_revisions() {
    let mut r = root();
    assert!(r.validate().is_ok());
    r.receipts.values_mut().next().unwrap().committed_revision = 0;
    assert!(r.validate().is_err());
    r.receipts.values_mut().next().unwrap().committed_revision = 2;
    assert!(r.validate().is_err());
    r.revision = 2;
    assert!(r.validate().is_ok());
    assert_eq!(decode::<HistoryRoot>(serde_json::to_value(&r).unwrap()), r);
}

#[test]
fn trial_revision_zero_is_invalid_and_later_results_do_not_replace_old_refs() {
    let mut t = trial();
    t.revision = 0;
    assert!(t.validate().is_err());
    t.revision = 2;
    t.previous = Some(blob());
    t.analysis.insert(decode(json!(RUN)), blob());
    t.analysis.insert(
        decode(json!(OP)),
        BlobRef {
            sha256: hash('d'),
            bytes: 200,
        },
    );
    assert!(t.validate().is_ok());
    let restored: TrialRevision = decode(serde_json::to_value(&t).unwrap());
    assert_eq!(restored, t);
    assert_eq!(restored.analysis.len(), 2);
}

#[test]
fn terminal_failures_require_end_and_fixed_failure_but_never_a_result() {
    for state in [
        AnalysisRunState::Failed,
        AnalysisRunState::Cancelled,
        AnalysisRunState::TimedOut,
        AnalysisRunState::Interrupted,
    ] {
        let mut a = analysis();
        a.state = state;
        a.ended_at = Some(at());
        a.failure = Some(FailureCode {
            domain: FailureDomain::Analysis,
            code: "interrupted".into(),
        });
        assert!(a.validate().is_ok());
        let mut bad = a.clone();
        bad.failure = None;
        assert!(bad.validate().is_err());
        let mut bad = a.clone();
        bad.ended_at = None;
        assert!(bad.validate().is_err());
        let mut bad = a;
        bad.result = Some(blob());
        assert!(bad.validate().is_err());
    }
    for state in [AdviceRunState::Failed, AdviceRunState::Interrupted] {
        let mut a = advice();
        a.state = state;
        a.ended_at = Some(at());
        a.failure = Some(FailureCode {
            domain: FailureDomain::Advice,
            code: "interrupted".into(),
        });
        assert!(a.validate().is_ok());
        let mut bad = a.clone();
        bad.failure = None;
        assert!(bad.validate().is_err());
        let mut bad = a.clone();
        bad.ended_at = None;
        assert!(bad.validate().is_err());
        let mut bad = a;
        bad.result = Some(blob());
        assert!(bad.validate().is_err());
    }
    let mut running = analysis();
    running.failure = Some(FailureCode {
        domain: FailureDomain::Analysis,
        code: "failed".into(),
    });
    assert!(running.validate().is_err());
    let mut success = advice();
    success.state = AdviceRunState::Assessed;
    success.send = SendCertainty::ResponseReceived;
    success.ended_at = Some(at());
    success.result = Some(blob());
    success.failure = Some(FailureCode {
        domain: FailureDomain::Advice,
        code: "failed".into(),
    });
    assert!(success.validate().is_err());
}

#[test]
fn advice_assessment_controls_heard_text_reason_and_feedback_arrays() {
    let assessed = result();
    assert!(assessed.validate().is_ok());
    let mut bad = assessed.clone();
    bad.heard_text = None;
    assert!(bad.validate().is_err());
    let mut bad = assessed.clone();
    bad.unassessable_reason = Some("Cannot assess.".into());
    assert!(bad.validate().is_err());
    let mut mismatch = assessed.clone();
    mismatch.assessment = AdviceAssessment::ReferenceMismatch;
    mismatch.strengths.clear();
    mismatch.improvements.clear();
    assert!(mismatch.validate().is_ok());
    let mut bad = mismatch.clone();
    bad.strengths.push("unexpected".into());
    assert!(bad.validate().is_err());
    let mut bad = mismatch.clone();
    bad.improvements = assessed.improvements.clone();
    assert!(bad.validate().is_err());
    let mut bad = mismatch.clone();
    bad.heard_text = None;
    assert!(bad.validate().is_err());
    let mut unable = mismatch;
    unable.assessment = AdviceAssessment::Unassessable;
    unable.heard_text = None;
    unable.unassessable_reason = Some("Synthetic missing speech.".into());
    assert!(unable.validate().is_ok());
    let mut bad = unable.clone();
    bad.unassessable_reason = None;
    assert!(bad.validate().is_err());
    let mut bad = unable.clone();
    bad.heard_text = Some("unexpected".into());
    assert!(bad.validate().is_err());
    let mut bad = unable.clone();
    bad.strengths.push("unexpected".into());
    assert!(bad.validate().is_err());
    let mut bad = unable;
    bad.improvements = assessed.improvements;
    assert!(bad.validate().is_err());
}

#[test]
fn advice_text_and_metadata_limits_reject_overflow_and_empty_values() {
    let mut r = result();
    r.summary = "🙂".repeat(2_000);
    r.heard_text = Some("🙂".repeat(10_000));
    r.strengths = vec!["🙂".repeat(1_000); 8];
    r.improvements = vec![
        AdviceImprovement {
            reference_excerpt: "🙂".repeat(1_000),
            observation: "🙂".repeat(1_000),
            practice: "🙂".repeat(1_000),
        };
        8
    ];
    r.requested_model = "m".repeat(128);
    r.requested_effort = "e".repeat(128);
    r.actual_model = Some("a".repeat(128));
    r.actual_effort = Some("b".repeat(128));
    assert!(r.validate().is_ok());
    for (field, invalid) in [
        ("summary", json!(" ")),
        ("summary", json!("a".repeat(2_001))),
        ("heard_text", json!("a".repeat(10_001))),
        ("strengths", json!([" "])),
        ("strengths", json!(["a".repeat(1_001)])),
        ("strengths", json!(vec!["a"; 9])),
        (
            "improvements",
            json!(vec![result().improvements[0].clone(); 9]),
        ),
    ] {
        let mut value = serde_json::to_value(&r).unwrap();
        value[field] = invalid;
        assert!(
            decode::<AdviceResult>(value).validate().is_err(),
            "invalid {field}"
        );
    }
    for field in [
        "requested_model",
        "requested_effort",
        "actual_model",
        "actual_effort",
    ] {
        for bad in ["".into(), "a".repeat(129), "unsafe\nvalue".into()] {
            let mut value = serde_json::to_value(&r).unwrap();
            value[field] = json!(bad);
            assert!(
                decode::<AdviceResult>(value).validate().is_err(),
                "invalid {field}"
            );
        }
    }
    for field in ["reference_excerpt", "observation", "practice"] {
        for bad in [" ".to_owned(), "a".repeat(1_001)] {
            let mut value = serde_json::to_value(&r).unwrap();
            value["improvements"][0][field] = json!(bad);
            assert!(
                decode::<AdviceResult>(value).validate().is_err(),
                "invalid improvement {field}"
            );
        }
    }
    let mut unable = result();
    unable.assessment = AdviceAssessment::Unassessable;
    unable.heard_text = None;
    unable.strengths.clear();
    unable.improvements.clear();
    unable.unassessable_reason = Some("a".repeat(2_000));
    assert!(unable.validate().is_ok());
    for bad in [" ".into(), "a".repeat(2_001)] {
        unable.unassessable_reason = Some(bad);
        assert!(unable.validate().is_err());
    }
}
