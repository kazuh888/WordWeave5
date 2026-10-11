//! Private, serde-transparent identifier and reference wrappers.

use serde::{Deserialize, Serialize};

use crate::{ContractError, ContractErrorCode};

fn canonical_hex(value: u8) -> bool {
    value.is_ascii_digit() || (b'a'..=b'f').contains(&value)
}

fn validate_uuid(value: &str, field: &'static str) -> Result<(), ContractError> {
    let valid = value.len() == 36
        && value.bytes().enumerate().all(|(index, byte)| {
            if matches!(index, 8 | 13 | 18 | 23) {
                byte == b'-'
            } else {
                canonical_hex(byte)
            }
        });
    if valid {
        Ok(())
    } else {
        Err(ContractError {
            code: ContractErrorCode::InvalidId,
            field,
        })
    }
}

fn validate_hash(value: &str, field: &'static str) -> Result<(), ContractError> {
    if value.len() == 64 && value.bytes().all(canonical_hex) {
        Ok(())
    } else {
        Err(ContractError {
            code: ContractErrorCode::InvalidHash,
            field,
        })
    }
}

pub(crate) fn validate_bytes(
    bytes: u64,
    maximum: u64,
    field: &'static str,
) -> Result<(), ContractError> {
    if bytes == 0 {
        Err(ContractError {
            code: ContractErrorCode::InvalidValue,
            field,
        })
    } else if bytes > maximum {
        Err(ContractError {
            code: ContractErrorCode::LimitExceeded,
            field,
        })
    } else {
        Ok(())
    }
}

macro_rules! string_id {
    ($name:ident, $validator:ident, $field:literal) => {
        #[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
        #[serde(transparent)]
        pub struct $name(String);

        impl $name {
            pub fn parse(value: &str) -> Result<Self, ContractError> {
                $validator(value, $field)?;
                Ok(Self(value.to_owned()))
            }

            pub fn as_str(&self) -> &str {
                &self.0
            }

            pub fn validate(&self) -> Result<(), ContractError> {
                $validator(&self.0, $field)
            }
        }
    };
}

string_id!(TrialId, validate_uuid, "trial_id");
string_id!(RunId, validate_uuid, "run_id");
string_id!(OperationId, validate_uuid, "operation_id");
string_id!(Sha256, validate_hash, "sha256");

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BlobRef {
    pub sha256: Sha256,
    pub bytes: u64,
}

impl BlobRef {
    pub fn validate(&self) -> Result<(), ContractError> {
        self.sha256.validate()?;
        validate_bytes(self.bytes, 12 * 1024 * 1024, "blob.bytes")
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct RootRef(BlobRef);

impl RootRef {
    /// Wraps a reference without validating it. Call `validate` at the boundary.
    pub fn new(blob: BlobRef) -> Self {
        Self(blob)
    }

    pub fn as_blob(&self) -> &BlobRef {
        &self.0
    }

    pub fn validate(&self) -> Result<(), ContractError> {
        self.0.sha256.validate()?;
        validate_bytes(self.0.bytes, 8 * 1024 * 1024, "root.bytes")
    }
}
