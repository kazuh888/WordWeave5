//! AH-AC10/18: saved JSON is a strict, lossless document boundary.
//! Expectations come from design-i0 sections 3/4, not `analyze` output.

use acoustic_analysis::{AnalysisProfile, AnalysisReport, SampleValue};
use serde_json::{json, Value};

fn profile_document() -> Value {
    json!({
        "schema_version": 1,
        "profile_id": "reading-v1",
        "preprocessing": "pcm16_mean_channels_v1",
        "energy": {
            "window_ms": 25, "hop_ms": 10,
            "low_threshold_dbfs": -40.0, "min_low_interval_ms": 200
        },
        "pitch": {
            "floor_hz": 50.0, "top_hz": 800.0, "step_seconds": 0.010,
            "max_candidates": 15, "very_accurate": false,
            "attenuation_at_top": 0.03, "silence_threshold": 0.09,
            "voicing_threshold": 0.50, "octave_cost": 0.055,
            "octave_jump_cost": 0.35, "voiced_unvoiced_cost": 0.14
        }
    })
}

fn report_document() -> Value {
    // One second of mono PCM16 at 16 kHz with a constant level below -40 dBFS.
    // Windows start every 160 samples and end after 400 samples, clipped at 16000.
    // The entire 1-second hop union is a low-energy interval. F0 has two finite
    // samples; linear quantiles are 110/150/190 Hz with a valid fraction of 1.
    let level = 20.0 * (32.0_f64 / 32768.0).log10();
    let rms_frames: Vec<Value> = (0..100)
        .map(|k| {
            let start = k * 160;
            let end = (start + 400).min(16000);
            json!({
                "time_seconds": (start + end) as f64 / 32000.0,
                "value": {"kind": "finite", "value": level}
            })
        })
        .collect();
    json!({
        "schema_version": 1,
        "metrics_version": "acoustic-metrics-v1",
        "library_version": env!("CARGO_PKG_VERSION"),
        "profile": profile_document(),
        "sample_rate": 16000, "channels": 1, "input_frames": 16000,
        "pitch_engine": {
            "name": "schema-fixture", "version": "1",
            "binary_sha256": null, "script_sha256": null
        },
        "status": "complete",
        "measures": {
            "duration_seconds": 1.0,
            "rms_dbfs": {"kind": "finite", "value": level},
            "rms_frames": rms_frames,
            "low_intervals": [{"start_seconds": 0.0, "end_seconds": 1.0}],
            "low_total_seconds": 1.0, "low_fraction": 1.0,
            "pitch": {
                "frames": [
                    {"time_seconds": 0.1, "value": {"kind": "finite", "value": 100.0}},
                    {"time_seconds": 0.2, "value": {"kind": "finite", "value": 200.0}}
                ],
                "median_hz": {"kind": "finite", "value": 150.0},
                "p10_hz": {"kind": "finite", "value": 110.0},
                "p90_hz": {"kind": "finite", "value": 190.0},
                "valid_frame_fraction": {"kind": "finite", "value": 1.0}
            }
        },
        "quality": [{"kind": "backend_warning", "fraction": null}]
    })
}

fn decode_report(document: &Value) -> Result<AnalysisReport, serde_json::Error> {
    serde_json::from_slice(&serde_json::to_vec(document).unwrap())
}

fn valid_report_document() -> Value {
    let document = report_document();
    decode_report(&document)
        .expect("the hand-written schema fixture must decode")
        .validate()
        .expect("the hand-written schema fixture must be semantically valid");
    document
}

fn assert_schema_error(document: &Value) {
    // Decode failure, rather than a later validate() error, is the schema gate.
    assert!(
        decode_report(document).is_err(),
        "schema_error expected: malformed saved JSON decoded successfully"
    );
}

#[test]
fn ac10_valid_report_roundtrip_preserves_every_document_field() {
    let document = valid_report_document();
    let decoded = decode_report(&document).unwrap();
    let saved = serde_json::to_value(&decoded).unwrap();
    assert_eq!(saved, document);
    let reread = decode_report(&saved).unwrap();
    assert_eq!(reread, decoded);
    assert!(reread.validate().is_ok());
}

#[test]
fn ac10_valid_profile_roundtrip_preserves_fixed_settings() {
    let document = profile_document();
    let decoded: AnalysisProfile =
        serde_json::from_slice(&serde_json::to_vec(&document).unwrap()).unwrap();
    assert!(decoded.validate().is_ok());
    assert_eq!(serde_json::to_value(decoded).unwrap(), document);
}

macro_rules! report_unknown_field_case {
    ($name:ident, $pointer:literal) => {
        #[test]
        fn $name() {
            let mut document = valid_report_document();
            document
                .pointer_mut($pointer)
                .unwrap()
                .as_object_mut()
                .unwrap()
                .insert("future_field_sentinel".into(), json!("harmless-marker"));
            assert_schema_error(&document);
        }
    };
}

report_unknown_field_case!(ac10_report_rejects_unknown_field, "");
report_unknown_field_case!(ac10_nested_profile_rejects_unknown_field, "/profile");
report_unknown_field_case!(ac10_energy_profile_rejects_unknown_field, "/profile/energy");
report_unknown_field_case!(ac10_pitch_profile_rejects_unknown_field, "/profile/pitch");
report_unknown_field_case!(ac10_engine_identity_rejects_unknown_field, "/pitch_engine");
report_unknown_field_case!(ac10_measures_reject_unknown_field, "/measures");
report_unknown_field_case!(ac10_pitch_measure_rejects_unknown_field, "/measures/pitch");
report_unknown_field_case!(
    ac10_rms_timed_value_rejects_unknown_field,
    "/measures/rms_frames/0"
);
report_unknown_field_case!(
    ac10_pitch_timed_value_rejects_unknown_field,
    "/measures/pitch/frames/0"
);
report_unknown_field_case!(
    ac10_time_interval_rejects_unknown_field,
    "/measures/low_intervals/0"
);
report_unknown_field_case!(
    ac10_finite_sample_value_rejects_unknown_field,
    "/measures/rms_dbfs"
);
report_unknown_field_case!(ac10_quality_rejects_unknown_field, "/quality/0");

#[test]
fn ac10_standalone_profile_rejects_unknown_field() {
    let mut document = profile_document();
    let baseline: AnalysisProfile =
        serde_json::from_slice(&serde_json::to_vec(&document).unwrap()).unwrap();
    assert!(baseline.validate().is_ok());
    document["future_field_sentinel"] = json!("harmless-marker");
    assert!(
        serde_json::from_slice::<AnalysisProfile>(&serde_json::to_vec(&document).unwrap()).is_err()
    );
}

macro_rules! sample_unknown_field_case {
    ($name:ident, $sample:expr) => {
        #[test]
        fn $name() {
            let mut document = $sample;
            serde_json::from_slice::<SampleValue>(&serde_json::to_vec(&document).unwrap())
                .expect("the sample fixture must decode before mutation");
            document["future_field_sentinel"] = json!("harmless-marker");
            assert!(
                serde_json::from_slice::<SampleValue>(&serde_json::to_vec(&document).unwrap())
                    .is_err()
            );
        }
    };
}

sample_unknown_field_case!(
    ac10_digital_silence_sample_rejects_unknown_field,
    json!({"kind": "digital_silence"})
);
sample_unknown_field_case!(
    ac10_missing_sample_rejects_unknown_field,
    json!({"kind": "missing", "value": "backend_unavailable"})
);

macro_rules! required_option_case {
    ($missing:ident, $null:ident, $parent:literal, $key:literal) => {
        #[test]
        fn $missing() {
            let mut document = valid_report_document();
            document
                .pointer_mut($parent)
                .unwrap()
                .as_object_mut()
                .unwrap()
                .remove($key)
                .expect("the required Option key exists before mutation");
            assert_schema_error(&document);
        }

        #[test]
        fn $null() {
            let mut document = valid_report_document();
            document.pointer_mut($parent).unwrap()[$key] = Value::Null;
            let decoded = decode_report(&document).expect("explicit Option null must decode");
            assert!(decoded.validate().is_ok());
            assert_eq!(serde_json::to_value(decoded).unwrap(), document);
        }
    };
}

required_option_case!(
    ac10_report_requires_pitch_engine_key,
    ac10_report_allows_explicit_null_pitch_engine,
    "",
    "pitch_engine"
);
required_option_case!(
    ac10_engine_requires_binary_hash_key,
    ac10_engine_allows_explicit_null_binary_hash,
    "/pitch_engine",
    "binary_sha256"
);
required_option_case!(
    ac10_engine_requires_script_hash_key,
    ac10_engine_allows_explicit_null_script_hash,
    "/pitch_engine",
    "script_sha256"
);
required_option_case!(
    ac10_quality_requires_fraction_key,
    ac10_quality_allows_explicit_null_fraction,
    "/quality/0",
    "fraction"
);

#[test]
fn ac10_nonoptional_report_and_nested_fields_reject_missing_keys_and_null() {
    for (parent, key) in [
        ("", "schema_version"),
        ("", "profile"),
        ("", "measures"),
        ("/profile", "schema_version"),
        ("/profile/energy", "window_ms"),
        ("/profile/pitch", "floor_hz"),
        ("/pitch_engine", "version"),
        ("/quality/0", "kind"),
        ("/measures", "duration_seconds"),
        ("/measures/pitch", "frames"),
        ("/measures/rms_frames/0", "time_seconds"),
        ("/measures/low_intervals/0", "end_seconds"),
        ("/measures/rms_dbfs", "value"),
    ] {
        let baseline = valid_report_document();
        let mut missing = baseline.clone();
        missing
            .pointer_mut(parent)
            .unwrap()
            .as_object_mut()
            .unwrap()
            .remove(key)
            .unwrap();
        assert_schema_error(&missing);
        let mut null = baseline;
        null.pointer_mut(parent).unwrap()[key] = Value::Null;
        assert_schema_error(&null);
    }
}

#[test]
fn ac10_sample_variants_roundtrip_preserves_missing_reason_and_digital_silence() {
    for document in [
        json!({"kind": "finite", "value": -60.0}),
        json!({"kind": "digital_silence"}),
        json!({"kind": "missing", "value": "unvoiced"}),
        json!({"kind": "missing", "value": "backend_unavailable"}),
    ] {
        let decoded: SampleValue =
            serde_json::from_slice(&serde_json::to_vec(&document).unwrap()).unwrap();
        assert_eq!(serde_json::to_value(decoded).unwrap(), document);
    }
}
