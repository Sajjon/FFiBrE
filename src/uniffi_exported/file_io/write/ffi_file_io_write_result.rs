use crate::prelude::*;

#[derive(Enum, Clone, Debug)]
pub enum FFIFileIOWriteResult {
    Success { value: FFIFileIOWriteResponse },
    Failure { error: SwiftSideError },
}
