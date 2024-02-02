use crate::prelude::*;

#[derive(Enum, Clone, Debug)]
pub enum NetworkResult {
    Success { value: NetworkResponse },
    Failure { error: NetworkError },
}