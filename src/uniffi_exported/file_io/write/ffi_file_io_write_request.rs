use crate::prelude::*;

#[derive(Enum, Clone, Debug, PartialEq, Eq)]
pub enum FileAlreadyExistsStrategy {
    Overwrite,
    Abort,
}

#[derive(Record, Clone, Debug, PartialEq, Eq)]
pub struct FFIFileIOWriteRequest {
    pub absolute_path: String,
    pub contents: Vec<u8>,
    pub exists_strategy: FileAlreadyExistsStrategy,
}

impl FFIFileIOWriteRequest {
    pub fn new(
        absolute_path: String,
        contents: Vec<u8>,
        exists_strategy: FileAlreadyExistsStrategy,
    ) -> Self {
        Self {
            absolute_path,
            contents,
            exists_strategy,
        }
    }
}
