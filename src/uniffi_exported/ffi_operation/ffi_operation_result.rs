use crate::prelude::*;

/// UniFFI does not allow us to pass `[Swift]Result` <-> `[Rust]Result` (I think),
/// so this is an implementation of that, so that a [`FFIOperationHandler`] can pass
/// the result of an [`FFIOperation`] back to Rust land, i.e. either a `Failure`
/// or a `Success` (data).
#[derive(Enum, Clone, Debug)]
pub enum FFINetworkResult {
    Success { value: NetworkResponse },
    Failure { error: SwiftSideError },
}
impl From<FFINetworkResult> for FFIOperationResult {
    fn from(value: FFINetworkResult) -> Self {
        match value {
            FFINetworkResult::Success { value } => {
                Ok(FFIOperationOk::Networking { response: value })
            }
            FFINetworkResult::Failure { error } => Err(error),
        }
    }
}

pub type FFIOperationResult = Result<FFIOperationOk, SwiftSideError>;
