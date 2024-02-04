use crate::prelude::*;
use thiserror::Error as ThisError;

#[derive(Debug, PartialEq, Eq, Clone, Error, ThisError)]
pub enum FFIFileIOWriteError {
    #[error("Failed To Create New File For Writing")]
    FailedToCreateNewFile,

    #[error("Failed To Get Handle To File For Writing")]
    FailedToGetHandleToFileForWriting,

    #[error("Failed to write to file handle: '{underlying}'")]
    FailedToWriteToFileHandle { underlying: String },
}
