use crate::prelude::*;

#[derive(Enum, Clone, Debug, PartialEq, Eq)]
pub enum FFIFileIOWriteResponse {
    OverwriteAborted,
    DidWrite { already_existed: bool }
}
