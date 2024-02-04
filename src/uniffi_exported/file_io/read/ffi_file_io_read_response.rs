use crate::prelude::*;

#[derive(Enum, Clone, Debug, PartialEq, Eq)]
pub enum FFIFileIOReadResponse {
    Exists {
        file: FFIFileIOReadResponseFileExists,
    },
    DoesNotExist {
        absolute_path: String,
    },
}

impl From<FFIFileIOReadResponse> for Option<FFIFileIOReadResponseFileExists> {
    fn from(value: FFIFileIOReadResponse) -> Self {
        match value {
            FFIFileIOReadResponse::Exists { file } => Some(file),
            FFIFileIOReadResponse::DoesNotExist { absolute_path: _ } => None,
        }
    }
}

#[derive(Record, Clone, Debug, PartialEq, Eq)]
pub struct FFIFileIOReadResponseFileExists {
    pub absolute_path: String,
    pub contents: Vec<u8>,
}
