//! Shared practice-history contracts, independent of host storage and UI.
//!
//! Deserializing these DTOs does not establish validity or saved state.

pub mod ids;
pub mod model;
mod validate;

pub use ids::*;
pub use model::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ContractErrorCode {
    UnknownVersion,
    InvalidId,
    InvalidHash,
    InvalidValue,
    LimitExceeded,
    ReferenceMismatch,
    Conflict,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ContractError {
    pub code: ContractErrorCode,
    pub field: &'static str,
}
