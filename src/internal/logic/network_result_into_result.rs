use crate::prelude::*;

impl From<NetworkResult> for Result<NetworkResponse, NetworkError> {
    fn from(value: NetworkResult) -> Self {
        match value {
            NetworkResult::Success { value } => Ok(value),
            NetworkResult::Failure { error } => Err(error),
        }
    }
}
