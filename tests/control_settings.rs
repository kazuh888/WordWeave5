use wordweave5::store::{Progress, Settings};

#[test]
fn old_settings_keep_slow_speech_and_new_defaults_are_explicit() {
    let settings: Settings = serde_json::from_str(r#"{"slow_speech":true}"#).unwrap();
    let value = serde_json::to_value(settings).unwrap();
    assert_eq!(value["slow_speech"], true);
    assert_eq!(value.get("speech_rate"), Some(&serde_json::Value::Null));
    assert_eq!(value.get("codex_effort"), Some(&serde_json::json!("")));
}

#[test]
fn invalid_playback_rates_and_effort_text_are_rejected_before_save() {
    let base = serde_json::to_value(Progress::default()).unwrap();
    for rate in [0.49, 4.01, -1.0] {
        let mut value = base.clone(); value["settings"]["speech_rate"] = serde_json::json!(rate);
        assert!(serde_json::from_value::<Progress>(value).unwrap().validate().is_err());
    }
    let mut value = base;
    value["settings"]["codex_effort"] = serde_json::json!("high\nSECRET");
    assert!(serde_json::from_value::<Progress>(value).unwrap().validate().is_err());
}
