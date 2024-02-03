use crate::prelude::*;
use thiserror::Error as ThisError;

#[derive(Debug, PartialEq, Eq, Clone, Error, ThisError)]
pub enum FFIFileIOWriteError {
    #[error("UnknownError'")]
    Unknown { string: String },
}
