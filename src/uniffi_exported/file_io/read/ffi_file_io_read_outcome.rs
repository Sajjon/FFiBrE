use crate::prelude::*;

#[derive(Enum, Clone, Debug)]
pub enum FFIFileIOReadOutcome {
    Success { value: FFIFileIOReadResponse },
    Failure { error: FFIFileIOReadError },
}

impl Into<Result<FFIFileIOReadResponse, FFIFileIOReadError>> for FFIFileIOReadOutcome {
    fn into(self) -> Result<FFIFileIOReadResponse, FFIFileIOReadError> {
        match self {
            Self::Success { value } => Ok(value),
            Self::Failure { error } => Err(error),
        }
    }
}
