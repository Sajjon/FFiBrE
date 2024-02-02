use crate::prelude::*;

impl From<NetworkResult> for Result<NetworkResponse, SwiftSideError> {
    fn from(value: NetworkResult) -> Self {
        match value {
            NetworkResult::Success { value } => Ok(value),
            NetworkResult::Failure { error } => Err(error),
        }
    }
}
