use crate::prelude::*;

/// UniFFI does not allow us to pass `[Swift]Result` <-> `[Rust]Result` (I think),
/// so this is an implementation of that, so that a [`FFIOperationHandler`] can pass
/// the result of an [`FFIOperation`] back to Rust land, i.e. either a `Failure`
/// or a `Success` (data).
#[derive(Enum, Clone, Debug)]
pub enum FFINetworkingOutcome {
    Success { value: NetworkResponse },
    Failure { error: FFINetworkingError },
}

impl Into<Result<NetworkResponse, FFINetworkingError>> for FFINetworkingOutcome {
    fn into(self) -> Result<NetworkResponse, FFINetworkingError> {
        match self {
            Self::Success { value } => Ok(value),
            Self::Failure { error } => Err(error),
        }
    }
}
