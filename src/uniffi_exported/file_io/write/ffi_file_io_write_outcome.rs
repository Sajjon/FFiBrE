use crate::prelude::*;

#[derive(Enum, Clone, Debug)]
pub enum FFIFileIOWriteOutcome {
    Success { value: FFIFileIOWriteResponse },
    Failure { error: FFIFileIOWriteError },
}

impl Into<Result<FFIFileIOWriteResponse, FFIFileIOWriteError>> for FFIFileIOWriteOutcome {
    fn into(self) -> Result<FFIFileIOWriteResponse, FFIFileIOWriteError> {
        match self {
            Self::Success { value } => Ok(value),
            Self::Failure { error } => Err(error),
        }
    }
}
