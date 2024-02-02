use crate::prelude::*;

#[derive(Enum, Clone, Debug)]
pub enum FFIOperationResult {
    Success { value: Option<Vec<u8>> },
    Failure { error: SwiftSideError },
}
