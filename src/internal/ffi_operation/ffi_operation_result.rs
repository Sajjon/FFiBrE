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

pub type FFIOperationResult = Result<FFIOperationOk, SwiftSideError>;
