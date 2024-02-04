use crate::prelude::*;

#[derive(Enum, Clone, Debug)]
pub enum FFINetworkingOutcome {
    Success { value: FFINetworkingResponse },
    Failure { error: FFINetworkingError },
}

impl Into<Result<FFINetworkingResponse, FFINetworkingError>> for FFINetworkingOutcome {
    fn into(self) -> Result<FFINetworkingResponse, FFINetworkingError> {
        match self {
            Self::Success { value } => Ok(value),
            Self::Failure { error } => Err(error),
        }
    }
}
