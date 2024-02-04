use crate::prelude::*;

#[derive(Enum, Clone, Debug, PartialEq, Eq)]
pub enum FFIFileIOReadResponse {
    Exists { contents: Vec<u8> },
    DoesNotExist,
}

impl From<FFIFileIOReadResponse> for Option<Vec<u8>> {
    fn from(value: FFIFileIOReadResponse) -> Self {
        match value {
            FFIFileIOReadResponse::Exists { contents } => Some(contents),
            FFIFileIOReadResponse::DoesNotExist => None,
        }
    }
}
