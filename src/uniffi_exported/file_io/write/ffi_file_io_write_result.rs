use crate::prelude::*;

#[derive(Enum, Clone, Debug)]
pub enum FFIFileIOWriteResult {
    Success { value: FFIFileIOWriteResponse },
    Failure { error: FFIFileIOWriteError },
}

impl Into<Result<FFIFileIOWriteResponse, FFISideError>> for FFIFileIOWriteResult {
    fn into(self) -> Result<FFIFileIOWriteResponse, FFISideError> {
        match self {
            Self::Success { value } => Ok(value),
            Self::Failure { error } => Err(error.into()),
        }
    }
}
