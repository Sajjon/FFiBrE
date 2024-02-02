use crate::prelude::*;

#[derive(Enum, Clone, Debug)]
pub enum NetworkResult {
    Success { value: NetworkResponse },
    Failure { error: NetworkError },
}

impl From<NetworkResult> for Result<NetworkResponse, NetworkError> {
    fn from(value: NetworkResult) -> Self {
        match value {
            NetworkResult::Success { value } => Ok(value),
            NetworkResult::Failure { error } => Err(error),
        }
    }
}
