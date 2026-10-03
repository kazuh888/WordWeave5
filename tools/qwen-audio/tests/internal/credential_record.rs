//! Included only by credentials.rs cfg(all(test, windows)).
//! Synthetic owned memory; never calls Windows credential storage.

use super::{copy_and_wipe_blob, decode_record};
use crate::{ErrorCode, Region, SendDisposition};

#[test]
fn null_blob_with_zero_or_positive_length_is_rejected_before_access() {
    for len in [0, 1, 2560] {
        // NULL is supplied to exercise the documented early-validation path.
        let error = unsafe { copy_and_wipe_blob(std::ptr::null_mut(), len) }.unwrap_err();
        assert_eq!(error.code(), ErrorCode::CredentialRead);
        assert_eq!(error.disposition(), SendDisposition::NotSent);
    }
}

#[test]
fn nonnull_zero_length_is_rejected_without_writing_owned_memory() {
    let mut allocated = vec![0x7b_u8; 16];
    let before = allocated.clone();
    let error = unsafe { copy_and_wipe_blob(allocated.as_mut_ptr(), 0) }.unwrap_err();
    assert_eq!(error.code(), ErrorCode::CredentialRead);
    assert_eq!(allocated, before);
}

#[test]
fn oversized_allocated_blob_is_rejected_without_copying_or_wiping() {
    // Full allocation is deliberately present: a faulty implementation must
    // produce an assertion failure, not out-of-bounds memory access.
    let mut allocated = vec![0x6a_u8; 2561];
    let before = allocated.clone();
    let error = unsafe { copy_and_wipe_blob(allocated.as_mut_ptr(), allocated.len()) }.unwrap_err();
    assert_eq!(error.code(), ErrorCode::CredentialRead);
    assert_eq!(allocated, before);
}

#[test]
fn valid_blob_copy_matches_input_and_original_allocation_is_zeroed() {
    for len in [1, 32, 2560] {
        let mut allocated: Vec<u8> = (0..len).map(|n| (n % 251 + 1) as u8).collect();
        let before = allocated.clone();
        let copied =
            unsafe { copy_and_wipe_blob(allocated.as_mut_ptr(), allocated.len()) }.unwrap();
        assert_eq!(copied.as_slice(), before);
        assert!(allocated.iter().all(|byte| *byte == 0));
    }
}

// QS-AC-009: legacy persisted bytes are accepted without rewriting the record.
#[test]
fn legacy_tokyo_v1_record_decodes_without_changing_input_bytes() {
    let bytes = br#"{"version":1,"host":"https://legacy.ap-northeast-1.maas.aliyuncs.com/compatible-mode/v1","key":"synthetic-legacy-key","effort":"medium"}"#.to_vec();
    let before = bytes.clone();
    let connection = decode_record(&bytes).unwrap();
    assert_eq!(connection.host().region(), Region::Tokyo);
    assert_eq!(
        connection.host().as_str(),
        "https://legacy.ap-northeast-1.maas.aliyuncs.com/compatible-mode/v1"
    );
    assert_eq!(connection.key().value(), "synthetic-legacy-key");
    assert_eq!(connection.effort(), "medium");
    assert_eq!(bytes, before);
}

// QS-AC-007/009: each approved host derives its region from the persisted URL.
#[test]
fn v1_record_accepts_each_approved_region_without_store_access_or_migration() {
    for (region, id) in [
        (Region::Tokyo, "ap-northeast-1"),
        (Region::Singapore, "ap-southeast-1"),
        (Region::Beijing, "cn-beijing"),
        (Region::HongKong, "cn-hongkong"),
        (Region::Frankfurt, "eu-central-1"),
        (Region::Virginia, "us-east-1"),
    ] {
        let host = format!("https://fixture.{id}.maas.aliyuncs.com/compatible-mode/v1");
        let bytes = serde_json::to_vec(&serde_json::json!({
            "version": 1,
            "host": host,
            "key": "synthetic-six-region-key",
            "effort": "medium"
        }))
        .unwrap();
        let before = bytes.clone();
        let connection = decode_record(&bytes).unwrap();
        assert_eq!(connection.host().region(), region);
        assert_eq!(connection.host().as_str(), host);
        assert_eq!(connection.key().value(), "synthetic-six-region-key");
        assert_eq!(bytes, before);
    }
}

#[test]
fn unknown_record_version_effort_and_host_are_safe_read_errors() {
    for (field, replacement) in [
        ("version", serde_json::json!(0)),
        ("version", serde_json::json!(2)),
        ("effort", serde_json::json!("high")),
        ("effort", serde_json::json!("")),
        ("host", serde_json::json!("https://example.invalid")),
        (
            "host",
            serde_json::json!("https://fixture.us-west-1.maas.aliyuncs.com"),
        ),
        (
            "host",
            serde_json::json!("http://fixture.ap-northeast-1.maas.aliyuncs.com"),
        ),
    ] {
        let mut record = serde_json::json!({
            "version": 1,
            "host": "https://fixture.ap-northeast-1.maas.aliyuncs.com/compatible-mode/v1",
            "key": "synthetic-invalid-record-key",
            "effort": "medium"
        });
        record[field] = replacement;
        let bytes = serde_json::to_vec(&record).unwrap();
        let before = bytes.clone();
        let error = decode_record(&bytes).unwrap_err();
        assert_eq!(error.code(), ErrorCode::CredentialRead, "{field}");
        assert_eq!(error.disposition(), SendDisposition::NotSent);
        assert!(!format!("{error:?}").contains("synthetic-invalid-record-key"));
        assert_eq!(bytes, before);
    }
}

#[test]
fn invalid_key_and_malformed_v1_records_remain_safe_read_errors() {
    let valid = serde_json::json!({
        "version": 1,
        "host": "https://fixture.ap-northeast-1.maas.aliyuncs.com/compatible-mode/v1",
        "key": "synthetic-record-key",
        "effort": "medium"
    });
    let mut invalid = Vec::new();
    for key in ["", " \t", "synthetic-key\n"] {
        let mut record = valid.clone();
        record["key"] = serde_json::json!(key);
        invalid.push(serde_json::to_vec(&record).unwrap());
    }
    for field in ["version", "host", "key", "effort"] {
        let mut record = valid.clone();
        record.as_object_mut().unwrap().remove(field);
        invalid.push(serde_json::to_vec(&record).unwrap());
    }
    let mut unknown = valid;
    unknown["region"] = serde_json::json!("ap-northeast-1");
    invalid.push(serde_json::to_vec(&unknown).unwrap());
    invalid.extend([b"{malformed".to_vec(), Vec::new(), b"null".to_vec()]);
    for bytes in invalid {
        let error = decode_record(&bytes).unwrap_err();
        assert_eq!(error.code(), ErrorCode::CredentialRead);
        assert_eq!(error.disposition(), SendDisposition::NotSent);
    }
}
