use crate::prelude::*;

impl From<FFIOperationResult> for Result<FFIOperationOk, SwiftSideError> {
    fn from(value: FFIOperationResult) -> Self {
        match value {
            FFIOperationResult::Success { value } => Ok(value),
            FFIOperationResult::Failure { error } => Err(error),
        }
    }
}
