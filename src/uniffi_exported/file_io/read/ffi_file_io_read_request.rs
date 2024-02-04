use crate::prelude::*;

#[derive(Record, Clone, Debug, PartialEq, Eq)]
pub struct FFIFileIOReadRequest {
    pub absolute_path: String,
}

impl FFIFileIOReadRequest {
    pub fn new(absolute_path: String) -> Self {
        Self { absolute_path }
    }
}
