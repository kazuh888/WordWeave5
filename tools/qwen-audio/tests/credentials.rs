use qwen_audio::{ApiKey, Connection, ErrorCode, SafeError, SendDisposition, TokyoHost};

const ORIGIN: &str = "https://fixture.ap-northeast-1.maas.aliyuncs.com";

#[test]
fn tokyo_origin_and_full_base_normalize_to_same_exact_endpoint() {
    for suffix in ["", "/", "/compatible-mode/v1", "/compatible-mode/v1/"] {
        assert_eq!(
            TokyoHost::parse(&format!("{ORIGIN}{suffix}"))
                .unwrap()
                .as_str(),
            format!("{ORIGIN}/compatible-mode/v1")
        );
    }
    let workspace = "a".repeat(63);
    assert!(TokyoHost::parse(&format!(
        "https://{workspace}.ap-northeast-1.maas.aliyuncs.com"
    ))
    .is_ok());
}

#[test]
fn arbitrary_urls_credentials_ports_encoded_and_lookalike_hosts_are_rejected() {
    let urls = [
        "http://fixture.ap-northeast-1.maas.aliyuncs.com",
        "https://user:secret@fixture.ap-northeast-1.maas.aliyuncs.com",
        "https://fixture.ap-northeast-1.maas.aliyuncs.com:443",
        "https://fixture.ap-northeast-1.maas.aliyuncs.com:444",
        "https://fixture.ap-northeast-1.maas.aliyuncs.com/?x=1",
        "https://fixture.ap-northeast-1.maas.aliyuncs.com/#x",
        "https://fixture.ap-northeast-1.maas.aliyuncs.com/compatible-mode/v1//",
        "https://fixture.ap-northeast-1.maas.aliyuncs.com/other",
        "https://fixture.ap-northeast-1.maas.aliyuncs.com/compatible-mode/%76%31",
        "https://fi%78ture.ap-northeast-1.maas.aliyuncs.com",
        "https://fixture.ap-northeast-1.maas.aliyuncs.com.attacker.invalid",
        "https://fixture.ap-southeast-1.maas.aliyuncs.com",
        "https://extra.fixture.ap-northeast-1.maas.aliyuncs.com",
        "https://-fixture.ap-northeast-1.maas.aliyuncs.com",
        "https://fixture-.ap-northeast-1.maas.aliyuncs.com",
        "https://Fixture.ap-northeast-1.maas.aliyuncs.com",
        "https://.ap-northeast-1.maas.aliyuncs.com",
        "https://fixture.ap-northeast-1.maas.aliyuncs.com.",
    ];
    for url in urls {
        assert_eq!(
            TokyoHost::parse(url).err().expect(url).code(),
            ErrorCode::InvalidHost,
            "{url}"
        );
    }
    assert!(TokyoHost::parse(&format!(
        "https://{}.ap-northeast-1.maas.aliyuncs.com",
        "a".repeat(64)
    ))
    .is_err());
}

#[test]
fn api_key_is_never_debuggable_and_control_input_is_rejected() {
    let key = ApiKey::new("synthetic-secret-unique-sentinel".to_owned()).unwrap();
    assert!(!format!("{key:?}").contains("synthetic-secret-unique-sentinel"));
    let connection = Connection::new(TokyoHost::parse(ORIGIN).unwrap(), key);
    assert!(!format!("{connection:?}").contains("synthetic-secret-unique-sentinel"));
    for value in ["", "secret\n", "secret\r", "secret\0", "secret\t"] {
        assert_eq!(
            ApiKey::new(value.to_owned()).err().unwrap().code(),
            ErrorCode::InvalidKey
        );
    }
}

#[test]
fn safe_error_roundtrip_rejects_injected_raw_message() {
    let error = SafeError::new(ErrorCode::Network, SendDisposition::MayHaveBeenSent);
    let mut json = serde_json::to_value(&error).unwrap();
    assert_eq!(
        serde_json::from_value::<SafeError>(json.clone()).unwrap(),
        error
    );
    assert!(std::error::Error::source(&error).is_none());
    json["message"] = serde_json::json!("synthetic-key-sentinel private-body");
    assert!(serde_json::from_value::<SafeError>(json).is_err());
}
