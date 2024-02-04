use crate::prelude::*;

#[derive(Enum, Clone, Debug)]
pub enum FFIFileIOReadResult {
    Success { value: FFIFileIOReadResponse },
    Failure { error: FFIFileIOReadError },
}

impl Into<Result<FFIFileIOReadResponse, FFISideError>> for FFIFileIOReadResult {
    fn into(self) -> Result<FFIFileIOReadResponse, FFISideError> {
        match self {
            Self::Success { value } => Ok(value),
            Self::Failure { error } => Err(error.into()),
        }
    }
}
