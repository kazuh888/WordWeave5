use qwen_audio::{ApiHost, ApiKey, Connection, ErrorCode, Region, TokyoHost};

fn regions() -> [(Region, &'static str, &'static str); 6] {
    [
        (Region::Tokyo, "ap-northeast-1", "東京"),
        (Region::Singapore, "ap-southeast-1", "シンガポール"),
        (Region::Beijing, "cn-beijing", "北京"),
        (Region::HongKong, "cn-hongkong", "香港"),
        (Region::Frankfurt, "eu-central-1", "フランクフルト"),
        (Region::Virginia, "us-east-1", "バージニア"),
    ]
}

fn origin(workspace: &str, id: &str) -> String {
    format!("https://{workspace}.{id}.maas.aliyuncs.com")
}

// QS-AC-007: only these six options are offered, in approved display order.
#[test]
fn region_options_have_exact_ids_labels_and_examples() {
    for (index, (region, id, label)) in regions().into_iter().enumerate() {
        assert_eq!(Region::ALL[index], region);
        assert_eq!(region.id(), id);
        assert_eq!(region.label(), label);
        assert!(region.example_url().contains(id));
        assert!(region.example_url().starts_with("https://"));
        assert!(region.example_url().ends_with("/compatible-mode/v1"));
    }
    assert_eq!(Region::ALL.len(), 6);
}

#[test]
fn all_six_regions_normalize_origin_and_base_path_to_same_destination() {
    for (region, id, _) in regions() {
        let origin = origin("fixture-a1", id);
        let expected = format!("{origin}/compatible-mode/v1");
        for suffix in ["", "/", "/compatible-mode/v1", "/compatible-mode/v1/"] {
            let host = ApiHost::parse_for_region(&format!("{origin}{suffix}"), region)
                .expect("supported workspace destination");
            assert_eq!(host.as_str(), expected);
            assert_eq!(host.region(), region);
            assert_eq!(ApiHost::parse(host.as_str()).unwrap().region(), region);
        }
    }
}

#[test]
fn selected_region_must_match_destination_region() {
    for (destination, id, _) in regions() {
        for selected in Region::ALL {
            let parsed = ApiHost::parse_for_region(&origin("fixture", id), selected);
            if selected == destination {
                assert!(parsed.is_ok());
            } else {
                assert_eq!(parsed.err().unwrap().code(), ErrorCode::InvalidHost);
            }
        }
    }
}

#[test]
fn workspace_label_accepts_one_and_sixty_three_characters() {
    for (_, id, _) in regions() {
        for label in [
            "a".to_owned(),
            "0".to_owned(),
            "a-b1".to_owned(),
            "a".repeat(63),
        ] {
            assert!(
                ApiHost::parse(&origin(&label, id)).is_ok(),
                "{label} in {id}"
            );
        }
        for label in [
            String::new(),
            "a".repeat(64),
            "-a".to_owned(),
            "a-".to_owned(),
            "A".to_owned(),
            "a_b".to_owned(),
            "あ".to_owned(),
            "extra.fixture".to_owned(),
        ] {
            assert_eq!(
                ApiHost::parse(&origin(&label, id)).err().unwrap().code(),
                ErrorCode::InvalidHost,
                "invalid workspace {label} in {id}"
            );
        }
    }
}

#[test]
fn every_region_rejects_unsafe_url_forms_and_arbitrary_paths() {
    for (_, id, _) in regions() {
        let origin = origin("fixture", id);
        let host = origin.strip_prefix("https://").unwrap();
        let forbidden = [
            format!("http://{host}"),
            format!("https://user:synthetic@{host}"),
            format!("{origin}:443"),
            format!("{origin}:444"),
            format!("{origin}?query=1"),
            format!("{origin}#fragment"),
            format!("{origin}/compatible-mode/%76%31"),
            format!("{origin}/compatible-mode/v1//"),
            format!("{origin}/chat/completions"),
            format!("{origin}/other"),
            format!("{origin}.attacker.invalid"),
            format!("{origin}."),
            format!(" {origin}"),
            format!("{origin} "),
            format!("https://fi%78ture.{id}.maas.aliyuncs.com"),
        ];
        for url in forbidden {
            assert_eq!(
                ApiHost::parse(&url).err().unwrap().code(),
                ErrorCode::InvalidHost,
                "{url}"
            );
        }
    }
    for url in [
        "https://example.invalid",
        "https://fixture.us-west-1.maas.aliyuncs.com",
        "https://dashscope.aliyuncs.com/compatible-mode/v1",
    ] {
        assert_eq!(
            ApiHost::parse(url).err().unwrap().code(),
            ErrorCode::InvalidHost
        );
    }
}

// QS-AC-009: the legacy type must retain its Tokyo-only meaning.
#[test]
fn legacy_tokyo_host_stays_strict_and_converts_without_changing_url() {
    let legacy = TokyoHost::parse(&origin("fixture", "ap-northeast-1")).unwrap();
    let expected = legacy.as_str().to_owned();
    let expanded: ApiHost = legacy.into();
    assert_eq!(expanded.region(), Region::Tokyo);
    assert_eq!(expanded.as_str(), expected);
    for (_, id, _) in regions().into_iter().skip(1) {
        assert_eq!(
            TokyoHost::parse(&origin("fixture", id))
                .err()
                .unwrap()
                .code(),
            ErrorCode::InvalidHost
        );
    }
}

// QS-AC-008: reject opaque key transfer; keep the source usable after failure.
#[test]
fn changed_workspace_or_region_requires_new_key_and_preserves_source() {
    let old = Connection::new(
        ApiHost::parse(&origin("old", "ap-northeast-1")).unwrap(),
        ApiKey::new("synthetic-host-bound-key".into()).unwrap(),
    );
    for (workspace, id) in [
        ("new", "ap-northeast-1"),
        ("old", "ap-southeast-1"),
        ("new", "cn-beijing"),
    ] {
        let changed = ApiHost::parse(&origin(workspace, id)).unwrap();
        assert!(old.with_host(changed).is_err());
        assert_eq!(
            old.host().as_str(),
            "https://old.ap-northeast-1.maas.aliyuncs.com/compatible-mode/v1"
        );
    }
    let same = old
        .with_host(ApiHost::parse(old.host().as_str()).unwrap())
        .unwrap();
    assert_eq!(same.host().as_str(), old.host().as_str());
    assert!(!format!("{old:?} {same:?}").contains("synthetic-host-bound-key"));
}

#[test]
fn normalized_same_host_can_keep_opaque_key_in_each_region() {
    for (region, id, _) in regions() {
        let origin = origin("fixture", id);
        let old = Connection::new(
            ApiHost::parse(&origin).unwrap(),
            ApiKey::new("synthetic-same-host-key".into()).unwrap(),
        );
        let same = old
            .with_host(ApiHost::parse(&format!("{origin}/compatible-mode/v1/")).unwrap())
            .expect("normalization does not change key destination");
        assert_eq!(same.host().as_str(), old.host().as_str());
        assert_eq!(same.host().region(), region);
    }
}

#[test]
fn new_connection_rejects_blank_and_header_invalid_key_inputs() {
    for key in [
        "", " ", "\t　", "secret\n", "secret\r", "secret\0", "secret\t",
    ] {
        assert_eq!(
            ApiKey::new(key.to_owned()).err().unwrap().code(),
            ErrorCode::InvalidKey
        );
    }
    assert!(ApiKey::new("synthetic-new-key".to_owned()).is_ok());
}
