use qwen_audio::{ApiHost, ApiKey, Connection, TokyoHost};

#[test]
fn host_replacement_requires_new_key_and_preserves_source() {
    let old = Connection::new(
        TokyoHost::parse("https://old.ap-northeast-1.maas.aliyuncs.com").unwrap(),
        ApiKey::new("synthetic-namespace-secret".into()).unwrap(),
    );
    let next =
        old.with_host(ApiHost::parse("https://new.ap-northeast-1.maas.aliyuncs.com").unwrap());
    assert!(
        next.is_err(),
        "a different workspace must not inherit the old key"
    );
    assert!(old.host().as_str().contains("//old."));
    assert!(!format!("{old:?} {next:?}").contains("synthetic-namespace-secret"));
}

#[cfg(all(windows, feature = "windows-credentials"))]
mod windows_namespace {
    use qwen_audio::WindowsCredentialStore;
    #[test]
    fn legacy_constructor_keeps_target() {
        assert_eq!(
            WindowsCredentialStore::new().target(),
            "WordWeave5.QwenAudio.Connection.v1"
        );
        assert_eq!(
            WindowsCredentialStore::default().target(),
            "WordWeave5.QwenAudio.Connection.v1"
        );
    }
    #[test]
    fn namespaces_never_read_legacy_record() {
        // Constructor/accessor only: never load, save, enumerate or remove OS credentials.
        let reading =
            WindowsCredentialStore::for_target("WordWeave5.QwenReading.Connection.v1").unwrap();
        assert_eq!(reading.target(), "WordWeave5.QwenReading.Connection.v1");
        assert_ne!(reading.target(), WindowsCredentialStore::new().target());
    }
    #[test]
    fn invalid_namespace_is_rejected_without_os_access() {
        for value in ["", "bad\0target"] {
            assert!(WindowsCredentialStore::for_target(value).is_err());
        }
        let oversized: &'static str = Box::leak("x".repeat(40_000).into_boxed_str());
        assert!(WindowsCredentialStore::for_target(oversized).is_err());
    }
}
