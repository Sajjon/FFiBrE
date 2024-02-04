use crate::prelude::*;

#[derive(Record, Clone, Debug, PartialEq, Eq)]
pub struct FFIFileIOWriteResponse {
    pub already_existed: bool,
}
