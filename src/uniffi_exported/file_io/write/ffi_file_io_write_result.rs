use crate::prelude::*;

#[derive(Enum, Clone, Debug)]
pub enum FFIFileIOWriteResult {
    Success { value: FFIFileIOWriteResponse },
    Failure { error: SwiftSideError },
}

impl Into<Result<FFIFileIOWriteResponse, SwiftSideError>> for FFIFileIOWriteResult {
    fn into(self) -> Result<FFIFileIOWriteResponse, SwiftSideError> {
        match self {
            Self::Success { value } => Ok(value),
            Self::Failure { error } => Err(error),
        }
    }
}
