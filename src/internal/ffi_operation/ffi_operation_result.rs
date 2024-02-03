use crate::prelude::*;

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


impl From<FFIFileIOWriteResult> for FFIOperationResult {
    fn from(value: FFIFileIOWriteResult) -> Self {
        match value {
            FFIFileIOWriteResult::Success { value } => {
                Ok(FFIOperationOk::FileIOWrite  { response: value })
            }
            FFIFileIOWriteResult::Failure { error } => Err(error),
        }
    }
}

pub type FFIOperationResult = Result<FFIOperationOk, SwiftSideError>;
